use std::io;

struct ProtocolID {
    protocol: String,
    id: String,
}

static mut PROTOCOL: ProtocolID = ProtocolID {
    protocol: String::new(),
    id: String::new(),
};

const ID_LEN: u32 = 12;
const PROTOCOL_LEN: u32 = 4;
const CODE_LEN: u32 = 1;

pub struct Message {
    code: u8,
    messaage: String,
    pro_id: ProtocolID,
}
// MOYU000000000000o1231323
// 0..4..17..18..
impl Message {
    pub fn parse(mes: &String) -> Result<Message, io::Error> {
        let message = Message {
            code: mes[(ID_LEN + PROTOCOL_LEN) as usize..(ID_LEN + PROTOCOL_LEN + CODE_LEN) as usize].as_bytes()[0],
            messaage: String::from(&mes[(ID_LEN+PROTOCOL_LEN+CODE_LEN) as usize..]),
            pro_id: ProtocolID { 
                protocol: String::from(&mes[..PROTOCOL_LEN as usize]),
                id: Message::parse_id(
                    &String::from(&mes[PROTOCOL_LEN as usize..(PROTOCOL_LEN + ID_LEN) as usize])
                )
             },
        };
        let de_protocol = get_protocol().unwrap();
        if message.pro_id.protocol != de_protocol {
            Err(io::Error::new(
                io::ErrorKind::InvalidData, 
                "Protocol invalid"))
        }else{
            Ok(message)
        }
    }

    fn parse_id(raw_id: &String) -> String {
        super::buf::push_message(raw_id);
        let mut id = String::new();
        let raw_id: Vec<char> = raw_id.chars().collect();
        let mut index = 0;
        for i in raw_id.len()-1..=0 {
            if raw_id[i] != '0' {
                index = i;
            }
        };
        
        for i in 0..=index {
            id.push(raw_id[i]);
        }
        id
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
            for _ in 0..(ID_LEN as usize-len) {
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