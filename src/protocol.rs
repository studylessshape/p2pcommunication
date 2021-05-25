use std::{io, usize};

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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
            code: Message::parse_code(String::from(&mes[(ID_LEN + PROTOCOL_LEN)..(ID_LEN + PROTOCOL_LEN + CODE_LEN)])),
            messaage: String::from(&mes[(ID_LEN + PROTOCOL_LEN + CODE_LEN)..]).trim_end().trim_start().to_string(),
            pro_id: ProtocolID {
                protocol: String::from(&mes[..PROTOCOL_LEN]),
                id: String::from(&mes[PROTOCOL_LEN..(PROTOCOL_LEN + ID_LEN)]),
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

    pub fn parse_id(raw_id: &String) -> String {
        let mut id = String::new();
        let raw_id: Vec<char> = raw_id.chars().collect();
        let mut index = (raw_id.len() - 1) as i32;
        while index >= 0 {
            if raw_id[index as usize] != '0' {
                break;
            }
            index -= 1;
        }

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

    fn parse_code(s_code: String) -> u8 {
        if s_code.len() > 2 {
            return 0;
        }
        match s_code.chars().next() {
            Some(c) => {
                if c >= '0' || c <= '9' {
                    match c {
                        '0' => return 0,
                        '1' => return 1,
                        '2' => return 2,
                        '3' => return 3,
                        '4' => return 4,
                        '5' => return 5,
                        '6' => return 6,
                        '7' => return 7,
                        '8' => return 8,
                        '9' => return 9,
                        _=> return 0
                    }
                }
                return 0;
            },
            None => return 0,
        }
    }
}

impl ToString for Message {
    fn to_string(&self) -> String {
        format!("{}:\t{}", Message::parse_id(&self.pro_id.id), self.messaage)
    }
}

pub fn set_protocol(protocol: String) {
    unsafe {
        if protocol.len() >= PROTOCOL_LEN {
            PROTOCOL.protocol = protocol[..PROTOCOL_LEN].to_string();
        }else{
            PROTOCOL.protocol = protocol;
            for _ in 0..PROTOCOL_LEN-PROTOCOL.protocol.len() {
                PROTOCOL.protocol.push('0');
            }
        }
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
