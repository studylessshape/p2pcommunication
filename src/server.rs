use std::{
    collections::VecDeque,
    io, mem,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
};

use crossterm::style::{Colorize, Styler};

use crate::protocol::Message;

use super::protocol;

#[derive(PartialEq, Clone, Copy)]
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
pub const JOIN_SUCCESS: &'static str = "Success join room";
pub const JOIN_FAILED: &'static str = "Error key";

pub fn set_identity(identity: Identity) {
    unsafe {
        IDENTITY = identity;
    }
}

pub fn get_identity() -> Identity{
    unsafe {
        IDENTITY
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

pub fn receive(socket: Arc<UdpSocket>, mess_que: Arc<Mutex<VecDeque<protocol::Message>>>) {
    let mut buf = [0u8; 512];
    let mut ips = Vec::<SocketAddr>::new();

    loop {
        if let Ok((_, addr)) = socket.recv_from(&mut buf) {
            let message = match protocol::Message::parse(&String::from_utf8_lossy(&buf).to_string()) {
                Ok(mes) => mes,
                Err(_) => continue,
            };
            let code = Code::from(message.code);
            match code {
                Code::Search => {
                    receive_search(addr, socket.clone());
                }
                Code::Request => {
                    receive_request(&message, mess_que.clone(), addr, &mut ips, socket.clone());
                }
                Code::Reply => {
                    receive_reply(&message, &mut ips, addr ,mess_que.clone());
                }
                Code::Message => {
                    receive_message(&message, mess_que.clone(), &ips, socket.clone());
                }
            };
        }
    }
}

fn receive_search(addr: SocketAddr, socket: Arc<UdpSocket>) {
    if is_room_owner() {
        send_message_to(
            &protocol::Message::new(Code::Reply as u8, &protocol::get_id().unwrap()),
            &addr,
            socket.clone(),
        );
    }
}

/// This function is only call back by room owner.
///
/// First compare the key recevied. If it is correct, client will join this ip to ip list and send new joiner to all ip.
///
/// If is not, client will send this ip a message to notice the key is error.
fn receive_request(
    message: &protocol::Message,
    mess_que: Arc<Mutex<VecDeque<protocol::Message>>>,
    addr: SocketAddr,
    ips: &mut Vec<SocketAddr>,
    socket: Arc<UdpSocket>,
) {
    if is_room_owner() {
        // Compare key
        if message.messaage == get_key() {
            // Send to this ip with join success message
            send_message_to(
                &protocol::Message::new(Code::Reply as u8, &String::from(JOIN_SUCCESS)),
                &addr,
                socket.clone(),
            );
            // Let this ip join the ip list
            ips.push(addr);
            let join_message = Message {
                code: Code::Message as u8,
                messaage: JOIN_SUCCESS.bold().red().to_string(),
                pro_id: protocol::ProtocolID {
                    id: message.pro_id.id.clone(),
                    protocol: protocol::get_protocol().unwrap(),
                },
            };
            push_to_message_queue(&join_message, mess_que);
            // Send the join message to all ip
            send_message_to_all(&join_message, ips, socket);
        } else {
            send_message_to(
                &protocol::Message::new(Code::Reply as u8, &String::from(JOIN_FAILED)),
                &addr,
                socket.clone(),
            );
        }
    }
}

fn receive_reply(message: &protocol::Message, ips: &mut Vec<SocketAddr>, addr: SocketAddr, mess_que: Arc<Mutex<VecDeque<protocol::Message>>>) {
    if !is_room_owner(){
        if message.messaage == JOIN_SUCCESS {
            ips.push(addr);
        }
        push_to_message_queue(message, mess_que);
    }
}

/// If room owner receive message by Code::Message, it will send this message to all ip
///
/// If not, it will push this message to message queue
fn receive_message(
    message: &protocol::Message,
    mess_que: Arc<Mutex<VecDeque<protocol::Message>>>,
    ips: &Vec<SocketAddr>,
    socket: Arc<UdpSocket>,
) {
    push_to_message_queue(message, mess_que);
    if is_room_owner() {
        send_message_to_all(message, ips, socket);
    }
}

fn push_to_message_queue(
    message: &protocol::Message,
    mess_que: Arc<Mutex<VecDeque<protocol::Message>>>,
) {
    let mut lock_messages = mess_que.lock().unwrap();
    lock_messages.push_back(message.clone());
}

fn send_message_to(message: &protocol::Message, addr: &SocketAddr, socket: Arc<UdpSocket>) {
    socket.send_to(&message.to_buf(), addr).unwrap();
}

fn send_message_to_all(message: &protocol::Message, ips: &Vec<SocketAddr>, socket: Arc<UdpSocket>) {
    for ip in ips.iter() {
        send_message_to(&message, ip, socket.clone());
    }
}

/// Get local address method from [@egmkang](https://github.com/egmkang/local_ipaddress)
pub fn get_local_addr() -> io::Result<SocketAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    socket.connect("8.8.8.8:80")?;

    socket.local_addr()
}
