use std::{
    collections::VecDeque,
    io, mem,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
};

use super::protocol;

#[derive(PartialEq)]
pub enum Identity {
    None,
    RoomOwner,
    RooomJoiner,
}

#[repr(u8)]
enum Code {
    Search = 0,
    Request,
    Reply,
    Message,
}

impl From<u8> for Code {
    fn from(code: u8) -> Self {
        unsafe { mem::transmute(code) }
    }
}

static mut IDENTITY: Identity = Identity::None;
static mut KEY: String = String::new();

pub fn set_identity(identity: Identity) {
    unsafe {
        IDENTITY = identity;
    }
}

pub fn get_key() -> String {
    unsafe { KEY.clone() }
}

pub fn is_room_owner() -> bool {
    unsafe { IDENTITY == Identity::RoomOwner }
}

pub fn set_key(key: String) {
    unsafe {
        KEY = key;
    }
}

pub fn initialize() {}

pub fn receive(socket: Arc<UdpSocket>, mess_que: Arc<Mutex<VecDeque<protocol::Message>>>) {
    let is_room_owner = is_room_owner();

    let mut buf = [0u8; 512];
    let mut ips = Vec::<SocketAddr>::new();

    loop {
        if let Ok((_, addr)) = socket.recv_from(&mut buf) {
            let message =
                protocol::Message::parse(&String::from_utf8_lossy(&buf).to_string()).unwrap();
            let code = Code::from(message.code);
            match code {
                Code::Search => {
                    receive_search(
                        &addr,
                        socket.clone()
                    );
                }
                Code::Request => {
                    receive_request(
                        &message,
                        &addr,
                        &mut ips,
                        socket.clone(),
                    );
                    
                }
                Code::Reply => {
                }
                Code::Message => {
                }
            };
        }
    }
}

fn receive_search(addr: &SocketAddr, socket: Arc<UdpSocket>) {
    if is_room_owner() {
        send_message_to(
            &protocol::Message::new(Code::Reply as u8,
                &protocol::get_id().unwrap()),
            &addr,
            socket.clone(),
        );
    }
}

fn receive_request(message: &protocol::Message, addr: &SocketAddr, ips: &mut Vec<SocketAddr>, socket: Arc<UdpSocket>) {
    if is_room_owner() {
        if message.messaage == get_key() {
            send_message_to(
                &protocol::Message::new(Code::Reply as u8,
                    &String::from("Success join room")),
                &addr,
                socket.clone(),
            );
            ips.push(addr.clone());
        } else {
            send_message_to(
                &protocol::Message::new(Code::Reply as u8,
                    &String::from("Error key")),
                &addr,
                socket.clone(),
            );
        }
    }
}

fn receive_reply() {

}

fn receive_message() {
    if is_room_owner() {
        
    }
}

fn push_to_message_queue(
    message: protocol::Message,
    mess_que: Arc<Mutex<VecDeque<protocol::Message>>>,
) {
    let mut lock_messages = mess_que.lock().unwrap();
    lock_messages.push_back(message);
}

fn send_message_to(message: &protocol::Message, addr: &SocketAddr, socket: Arc<UdpSocket>) {
    socket
        .send_to(&message.to_buf(), addr)
        .unwrap();
}

fn send_message_to_all(message: protocol::Message, ips: &Vec<SocketAddr>, socket: Arc<UdpSocket>) {
    for ip in ips.iter() {
        send_message_to(&message, ip, socket.clone());
    }
}

pub fn get_local_addr() -> io::Result<SocketAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    socket.connect("8.8.8.8:80")?;

    socket.local_addr()
}
