use crate::RsCraftError;
use serde::Serialize;

const SEGMENT_BITS: u8 = 0b0111_1111;
const CONTINUE_BIT: u8 = 0b1000_0000;

#[derive(Debug, Serialize)]
pub struct ListPingResponse {
    pub version: ListPingResponseVersion,
    pub players: ListPingResponsePlayers,
    pub description: ChatMessage,
}

#[derive(Debug, Serialize)]
pub struct ListPingResponseVersion {
    pub name: String,
    pub protocol: u32,
}

#[derive(Debug, Serialize)]
pub struct ListPingResponsePlayers {
    pub max: u32,
    pub online: u32,
    pub sample: Vec<ListPingResponsePlayerSample>,
}

#[derive(Debug, Serialize)]
pub struct ListPingResponsePlayerSample {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub text: String,
    #[serde(skip_serializing_if = "std::string::String::is_empty")]
    pub bold: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<ChatMessage>,
}

impl ChatMessage {
    pub fn text(text: String) -> Self {
        Self {
            text: text.replace("&", "§"),
            bold: String::new(),
            extra: vec![],
        }
    }
    pub fn set_bold(&mut self, value: bool) {
        self.bold = value.to_string();
    }
    pub fn str(text: &str) -> Self {
        ChatMessage::text(String::from(text).replace("&", "§"))
    }
}

/// FIXME /\

pub(super) trait PacketReader {
    fn get_u8(&mut self) -> Result<u8, RsCraftError>;
    fn get_u16(&mut self) -> Result<u16, RsCraftError>;
    fn get_i32(&mut self) -> Result<i32, RsCraftError>;
    fn get_varint(&mut self) -> Result<i32, RsCraftError>;
    fn get_string(&mut self) -> Result<String, RsCraftError>;
}

impl PacketReader for Vec<u8> {
    fn get_u8(&mut self) -> Result<u8, RsCraftError> {
        if self.len() == 0 {
            error!("Trying to pop item from empty vector");
            return Err(RsCraftError::PacketParsingError(
                "Trying to pop item from empty vector".to_string(),
            ));
        }
        Ok(self.remove(0))
    }

    fn get_u16(&mut self) -> Result<u16, RsCraftError> {
        Ok(u16::from_be_bytes([self.get_u8()?, self.get_u8()?]))
    }

    fn get_i32(&mut self) -> Result<i32, RsCraftError> {
        Ok(i32::from_be_bytes([
            self.get_u8()?,
            self.get_u8()?,
            self.get_u8()?,
            self.get_u8()?,
        ]))
    }

    fn get_varint(&mut self) -> Result<i32, RsCraftError> {
        let mut ans = 0;
        for i in 0..4 {
            let buf = self.get_u8()?;
            ans |= ((buf & SEGMENT_BITS) as i32) << 7 * i;
            if buf & CONTINUE_BIT == 0 {
                break;
            }
        }
        Ok(ans)
    }

    fn get_string(&mut self) -> Result<String, RsCraftError> {
        let len = self.get_varint()?;
        let mut buf = Vec::new();
        for _ in 0..len {
            buf.push(self.get_u8()?)
        }
        String::from_utf8(buf).map_err(|err| {
            error!("Failed to parse string: {:?}", err);
            RsCraftError::PacketParsingError(format!("Failed to parse string: {:?}", err))
        })
    }
}

pub(super) trait PacketWriter {
    fn write_u8(&mut self, value: u8);
    fn write_u128(&mut self, value: u128);
    fn write_i32(&mut self, value: i32);
    fn write_varint(&mut self, value: i32);
    fn write_string(&mut self, value: String);
    fn create_packet(&mut self, pid: i32) -> Vec<u8>;
}

impl PacketWriter for Vec<u8> {
    fn write_u8(&mut self, value: u8) {
        self.push(value)
    }

    fn write_u128(&mut self, value: u128) {
        self.extend_from_slice(&value.to_be_bytes());
    }

    fn write_i32(&mut self, value: i32) {
        self.extend_from_slice(&value.to_be_bytes());
    }

    fn write_varint(&mut self, mut value: i32) {
        let mut buf = vec![0u8; 1];
        let mut n = 0;
        loop {
            if value <= 127 || n >= 8 {
                break;
            }
            buf.insert(n, (0x80 | (value & 0x7F)) as u8); // FIXME reuse values 0x80 and 0x7F
            value >>= 7;
            value -= 1;
            n += 1;
        }
        buf.insert(n, value as u8);
        n += 1;
        self.extend_from_slice(&buf.as_slice()[..n])
    }

    fn write_string(&mut self, value: String) {
        let bytes = value.as_bytes();
        self.write_varint(bytes.len() as i32);
        self.extend_from_slice(bytes);
    }

    fn create_packet(&mut self, pid: i32) -> Vec<u8> {
        let mut packet = Vec::new();
        let mut len_bytes: Vec<u8> = Vec::new();
        len_bytes.write_varint(pid);
        packet.write_varint((self.len() + len_bytes.len()) as i32);
        packet.extend_from_slice(len_bytes.as_slice());
        drop(len_bytes);
        packet.extend_from_slice(self.as_slice());
        packet
    }
}
