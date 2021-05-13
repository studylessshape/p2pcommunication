use std::{io, usize};

#[derive(Clone)]
pub struct ProtocolID {
    pub protocol: String,
    pub id: String,
}

static mut PROTOCOL: ProtocolID = ProtocolID {
    protocol: String::new(),
    id: String::new(),
};

pub const ID_LEN: usize = 12;
pub const PROTOCOL_LEN: usize = 4;
pub const CODE_LEN: usize = 1;

pub struct Message {
    pub code: u8,
    pub messaage: String,
    pub pro_id: ProtocolID,
}

impl Message {
    pub fn new(
        code: u8,
        messa: &String
    ) -> Message {
        Message {
            code,
            messaage: messa.clone(),
            pro_id: unsafe {PROTOCOL.clone()}
        }
    }

    pub fn parse(mes: &String) -> Result<Message, io::Error> {
        let message = Message {
            code: mes[(ID_LEN + PROTOCOL_LEN)..(ID_LEN + PROTOCOL_LEN + CODE_LEN)].as_bytes()[0],
            messaage: String::from(&mes[(ID_LEN + PROTOCOL_LEN + CODE_LEN)..]),
            pro_id: ProtocolID {
                protocol: String::from(&mes[..PROTOCOL_LEN]),
                id: Message::parse_id(&String::from(&mes[PROTOCOL_LEN..(PROTOCOL_LEN + ID_LEN)])),
            },
        };
        let de_protocol = get_protocol().unwrap();
        if message.pro_id.protocol != de_protocol {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Protocol invalid",
            ))
        } else {
            Ok(message)
        }
    }

    fn parse_id(raw_id: &String) -> String {
        let mut id = String::new();
        let raw_id: Vec<char> = raw_id.chars().collect();
        super::buf::push_message(&format!("{:?}", raw_id));
        let mut index = (raw_id.len() - 1) as i32;
        while index >= 0 {
            if raw_id[index as usize] != '0' {
                break;
            }
            index -= 1;
        }

        super::buf::push_message(&format!("index: {}", index));
        if index >= 0 {
            for i in 0..=index as usize {
                id.push(raw_id[i]);
            }
        }
        id
    }

    pub fn to_buf(&self) -> Vec<u8> {
        format!("{}{}{}{}", self.pro_id.protocol, self.pro_id.id, self.code, self.messaage).bytes().collect()
    }
}

impl ToString for Message {
    fn to_string(&self) -> String {
        format!("{}:\t{}", self.pro_id.id, self.messaage)
    }
}

pub fn set_protocol(protocol: &String) {
    unsafe {
        PROTOCOL.protocol = protocol.clone();
    }
}

pub fn set_id(id: &String) {
    unsafe {
        let mut id = id.clone();
        if id.len() < ID_LEN as usize {
            let len = id.len();
            for _ in 0..(ID_LEN as usize - len) {
                id.push('0');
            }
        }
        PROTOCOL.id = String::from(&id[..ID_LEN as usize]);
    }
}

pub fn get_protocol() -> Option<String> {
    unsafe {
        if PROTOCOL.protocol != "" {
            Some(PROTOCOL.protocol.to_string())
        } else {
            None
        }
    }
}

pub fn get_id() -> Option<String> {
    unsafe {
        if PROTOCOL.id != "" {
            Some(PROTOCOL.id.to_string())
        } else {
            None
        }
    }
}
