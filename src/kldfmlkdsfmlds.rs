use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;

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

fn connection(mut server: TcpStream) {
    let mut buf = vec![0; 4096];
    let len = server.read(&mut buf).unwrap();
    let b = &buf[..len];
    let mut b_iter = b.iter();

    let len = {
        let mut ans = 0;
        for i in 0..4 {
            let buf = b_iter.next().unwrap();
            ans |= ((buf & 0b0111_1111) as i32) << 7 * i;
            if buf & 0b1000_0000 == 0 {
                break;
            }
        }
        ans
    };

    let pid = {
        let mut ans = 0;
        for i in 0..4 {
            let buf = b_iter.next().unwrap();
            ans |= ((buf & 0b0111_1111) as i32) << 7 * i;
            if buf & 0b1000_0000 == 0 {
                break;
            }
        }
        ans
    };

    let name = {
        let len = {
            let mut ans = 0;
            for i in 0..4 {
                let buf = b_iter.next().unwrap();
                ans |= ((buf & 0b0111_1111) as i32) << 7 * i;
                if buf & 0b1000_0000 == 0 {
                    break;
                }
            }
            ans
        };
        let mut buf = Vec::new();
        for _ in 0..len {
            buf.push(b_iter.next().unwrap().clone())
        }
        String::from_utf8(buf).unwrap()
    };

    println!("C -> S: Login Start - {:?}|{:?}|{:?}", len, pid, name);
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

fn list(mut server: TcpStream) {
    let mut buf = vec![0; 4096];
    let len = server.read(&mut buf).unwrap();
    let b = &buf[..len];
    let mut b_iter = b.iter();

    let len = {
        let mut ans = 0;
        for i in 0..4 {
            let buf = b_iter.next().unwrap();
            ans |= ((buf & 0b0111_1111) as i32) << 7 * i;
            if buf & 0b1000_0000 == 0 {
                break;
            }
        }
        ans
    };

    let pid = {
        let mut ans = 0;
        for i in 0..4 {
            let buf = b_iter.next().unwrap();
            ans |= ((buf & 0b0111_1111) as i32) << 7 * i;
            if buf & 0b1000_0000 == 0 {
                break;
            }
        }
        ans
    };

    println!("C -> S: Status {:?}|{:?}", len, pid);

    let mut json_response = Vec::new();

    let json = serde_json::to_string(&ListPingResponse {
        version: ListPingResponseVersion {
            name: String::from("ULE"),
            protocol: 758,
        },
        players: ListPingResponsePlayers {
            max: 88,
            online: 14,
            sample: vec![],
        },
        description: ChatMessage::str("&a&lJOPPA!"),
    })
    .unwrap();

    {
        let bytes = json.as_bytes();
        {
            let mut value = bytes.len() as i32;
            let mut buf = vec![0u8; 1];
            let mut n = 0;
            loop {
                if value <= 127 || n >= 8 {
                    break;
                }
                buf.insert(n, (0x80 | (value & 0x7F)) as u8);
                value >>= 7;
                value -= 1;
                n += 1;
            }
            buf.insert(n, value as u8);
            n += 1;
            json_response.extend_from_slice(&buf.as_slice()[..n])
        }
        json_response.extend_from_slice(bytes);
    }

    let mut packet = Vec::new();
    let mut len_bytes: Vec<u8> = Vec::new();
    {
        let mut value = 0x00;
        let mut buf = vec![0u8; 1];
        let mut n = 0;
        loop {
            if value <= 127 || n >= 8 {
                break;
            }
            buf.insert(n, (0x80 | (value & 0x7F)) as u8);
            value >>= 7;
            value -= 1;
            n += 1;
        }
        buf.insert(n, value as u8);
        n += 1;
        len_bytes.extend_from_slice(&buf.as_slice()[..n])
    }
    {
        let mut value = (json_response.len() + len_bytes.len()) as i32;
        let mut buf = vec![0u8; 1];
        let mut n = 0;
        loop {
            if value <= 127 || n >= 8 {
                break;
            }
            buf.insert(n, (0x80 | (value & 0x7F)) as u8);
            value >>= 7;
            value -= 1;
            n += 1;
        }
        buf.insert(n, value as u8);
        n += 1;
        packet.extend_from_slice(&buf.as_slice()[..n])
    }
    packet.extend_from_slice(len_bytes.as_slice());
    drop(len_bytes);
    packet.extend_from_slice(json_response.as_slice());
    server.write_all(&packet);

    println!("S -> C: List response");

    loop {
        let mut buf = vec![0; 4096];
        let len = server.read(&mut buf).unwrap();

        println!("C -> S: Ping");

        server.write_all(&buf);

        std::thread::sleep_ms(1000);

        println!("S -> C: Pong");
    }
}

fn main() {
    let listener = TcpListener::bind("192.168.1.8:25565").unwrap();
    loop {
        let (mut server, _) = listener.accept().unwrap();

        let mut buf = vec![0; 4096];
        let len = server.read(&mut buf).unwrap();
        let b = &buf[..len];
        let mut b_iter = b.iter();

        let len = {
            let mut ans = 0;
            for i in 0..4 {
                let buf = b_iter.next().unwrap();
                ans |= ((buf & 0b0111_1111) as i32) << 7 * i;
                if buf & 0b1000_0000 == 0 {
                    break;
                }
            }
            ans
        };

        let pid = {
            let mut ans = 0;
            for i in 0..4 {
                let buf = b_iter.next().unwrap();
                ans |= ((buf & 0b0111_1111) as i32) << 7 * i;
                if buf & 0b1000_0000 == 0 {
                    break;
                }
            }
            ans
        };

        let ver = {
            let mut ans = 0;
            for i in 0..4 {
                let buf = b_iter.next().unwrap();
                ans |= ((buf & 0b0111_1111) as i32) << 7 * i;
                if buf & 0b1000_0000 == 0 {
                    break;
                }
            }
            ans
        };

        let address = {
            let len = {
                let mut ans = 0;
                for i in 0..4 {
                    let buf = b_iter.next().unwrap();
                    ans |= ((buf & 0b0111_1111) as i32) << 7 * i;
                    if buf & 0b1000_0000 == 0 {
                        break;
                    }
                }
                ans
            };
            let mut buf = Vec::new();
            for _ in 0..len {
                buf.push(b_iter.next().unwrap().clone())
            }
            String::from_utf8(buf).unwrap()
        };

        let port = {
            u16::from_be_bytes([
                b_iter.next().unwrap().clone(),
                b_iter.next().unwrap().clone(),
            ])
        };

        let next_state = {
            let mut ans = 0;
            for i in 0..4 {
                let buf = b_iter.next().unwrap();
                ans |= ((buf & 0b0111_1111) as i32) << 7 * i;
                if buf & 0b1000_0000 == 0 {
                    break;
                }
            }
            ans
        };

        println!(
            "C -> S: Handshake - {:?}|{:?}|{:?}|{:?}:{:?}|{:?}",
            len, pid, ver, address, port, next_state
        );

        if next_state == 1 {
            list(server);
        } else if next_state == 2 {
            connection(server);
        }
    }
}
