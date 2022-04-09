use crate::error::RsCraftError;
use crate::packet::{
    ChatMessage, ListPingResponse, ListPingResponsePlayers, ListPingResponseVersion, PacketReader,
    PacketWriter,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

pub(super) struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn new(address: &str, port: u16) -> Result<Self, RsCraftError> {
        debug!("TCP server init on {}:{}", address, port);

        Ok(Self {
            listener: TcpListener::bind(format!("{}:{}", address, port))
                .await
                .map_err(|err| {
                    error!("Cannot init TcpListener: {:?}", err);
                    RsCraftError::TcpError(format!("Cannot init TcpListener: {:?}", err))
                })?,
        })
    }

    pub async fn accept(&self) -> Result<Connection, RsCraftError> {
        let (tcp_stream, address) = self.listener.accept().await.map_err(|err| {
            error!("Cannot accept connection: {:?}", err);
            RsCraftError::TcpError(format!("Cannot accept connection: {:?}", err))
        })?;

        debug!("Accepted new TCP connection {}", address);

        let stream = Stream::new(tcp_stream);
        Ok(Connection::new(stream))
    }
}

pub(super) struct Stream {
    stream: TcpStream,
}

impl Stream {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub async fn read(&mut self) -> Result<Vec<u8>, RsCraftError> {
        let mut buffer = [0u8; 4096];
        let packet_len = self.stream.read(&mut buffer).await.map_err(|err| {
            error!("Read socket error: {:?}", err);
            RsCraftError::TcpError(format!("Read socket error: {:?}", err))
        })?;
        if packet_len == 0 {
            debug!("Empty TCP body");
            return Err(RsCraftError::TcpError("Empty TCP body".to_string()));
        }
        Ok(buffer[..packet_len].to_vec())
    }

    pub async fn write_all(&mut self, value: Vec<u8>) -> Result<(), RsCraftError> {
        self.stream.write_all(&value).await.map_err(|err| {
            error!("Write to socket error: {:?}", err);
            RsCraftError::TcpError(format!("Write to socket error: {:?}", err))
        })
    }
}

pub(super) struct Connection {
    stream: Stream,
}

impl Connection {
    pub fn new(stream: Stream) -> Self {
        Self { stream }
    }

    pub async fn list(&mut self) -> Result<(), RsCraftError> {
        let mut packet = self.stream.read().await?;
        let len = packet.get_varint()?;
        let pid = packet.get_varint()?;

        debug!("LIST {}|{}", len, pid);

        let mut bytes = Vec::new();
        bytes.write_string(
            serde_json::to_string(&ListPingResponse {
                // FIXME
                version: ListPingResponseVersion {
                    name: String::from("ULE"),
                    protocol: 758,
                },
                players: ListPingResponsePlayers {
                    max: 88,
                    online: 14,
                    sample: vec![],
                },
                description: ChatMessage::str("o4ko"),
            })
            .unwrap(),
        );
        self.stream.write_all(bytes.create_packet(0x00)).await?;

        let ping = self.stream.read().await?;
        debug!("Ping");

        self.stream.write_all(ping).await?;
        std::thread::sleep_ms(500);
        debug!("Pong");

        Ok(())
    }

    pub async fn login(&mut self) -> Result<(), RsCraftError> {
        let mut packet = self.stream.read().await?;
        let len = packet.get_varint()?;
        let pid = packet.get_varint()?;
        let name = packet.get_string()?;
        debug!("Login start: {:?}|{:?}|{:?}", len, pid, name);

        let mut packet = Vec::new();
        let uuid = Uuid::new_v4().as_u128();
        PacketWriter::write_u128(&mut packet, uuid);
        packet.write_string(name);
        self.stream.write_all(packet.create_packet(0x02)).await?;

        debug!("Sent");

        let mut packet = Vec::new();
        PacketWriter::write_i32(&mut packet, 14881337);
        self.stream.write_all(packet.create_packet(0x30)).await?;
        debug!("ping");

        let mut packet = self.stream.read().await?;
        let len = packet.get_varint()?;
        let pid = packet.get_varint()?;
        let pong = packet.get_i32()?;
        debug!("{:?}|{:?}|{:?}", len, pid, pong);

        Ok(())
    }

    pub async fn handle(&mut self) -> Result<(), RsCraftError> {
        // Handshaking
        let mut packet = self.stream.read().await?;
        let len = packet.get_varint()?;
        let pid = packet.get_varint()?;
        let ver = packet.get_varint()?;
        let address = packet.get_string()?;
        let port = packet.get_u16()?;
        let next_state = packet.get_varint()?;
        debug!(
            "{}|{}|{}|{}:{}|{}",
            len, pid, ver, address, port, next_state
        );

        if next_state == 1 {
            self.list().await?;
        } else if next_state == 2 {
            self.login().await?;
        }

        Ok(())
    }
}
