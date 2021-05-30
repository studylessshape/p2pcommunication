use std::{
    collections::VecDeque,
    io, mem,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
};

use crossterm::style::{Colorize, Styler};

use crate::prelude::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Identity {
    None,
    RoomOwner,
    RooomJoiner,
}

impl Identity {
    pub fn is_room_owner(&self) -> bool {
        *self == Identity::RoomOwner
    }

    pub fn is_room_joiner(&self) -> bool {
        *self == Identity::RooomJoiner
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum Code {
    Request = 0,
    Reply,
    Message,
    Exit,
    None,
}

impl From<u8> for Code {
    fn from(code: u8) -> Self {
        if code <= Code::None as u8 {
            unsafe { mem::transmute(code) }
        } else {
            Code::None
        }
    }
}

static mut IDENTITY: Identity = Identity::None;
static mut KEY: String = String::new();
static mut ROOM_OWNER_IP: String = String::new();
pub const JOIN_SUCCESS: &'static str = "Success join room";
pub const JOIN_FAILED: &'static str = "Error key";
pub const EXIT_ROOM: &'static str = "Exit room";

pub fn set_identity(identity: Identity) {
    unsafe {
        IDENTITY = identity;
    }
}

pub fn get_identity() -> Identity {
    unsafe { IDENTITY }
}

pub fn get_key() -> String {
    unsafe { KEY.clone() }
}

pub fn is_room_owner() -> bool {
    unsafe { IDENTITY == Identity::RoomOwner }
}

pub fn set_key(key: String) {
    unsafe {
        KEY = key.clone();
    }
}

pub fn receive(socket: Arc<UdpSocket>, mess_que: Arc<Mutex<VecDeque<protocol::Message>>>) {
    let mut buf = [0u8; 512];
    let mut ips = Vec::<SocketAddr>::new();

    loop {
        if let Ok((size, addr)) = socket.recv_from(&mut buf) {
            let message = match protocol::Message::parse(&buf[..size]) {
                Ok(mes) => mes,
                Err(_) => continue,
            };
            let code = Code::from(message.code);
            match code {
                Code::Request => {
                    receive_request(&message, mess_que.clone(), addr, &mut ips, socket.clone());
                }
                Code::Reply => {
                    receive_reply(&message, &mut ips, addr, mess_que.clone());
                }
                Code::Message => {
                    receive_message(&message, mess_que.clone(), &ips, socket.clone());
                }
                Code::Exit => {
                    receive_exit(&message, addr, mess_que.clone(), &mut ips, socket.clone());
                }
                _ => {}
            };
            // buf.fill_with(Default::default);
            buf.fill(Default::default());
        }
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
        // buf::println(&format!("{}:{}", get_key().len(), get_key()), 26);
        // if compare_string(&message.message, &get_key()) {
        if message.message == get_key() {
            // Send to this ip with join success message
            send_message_to(
                &protocol::Message::new(Code::Reply as u8, &String::from(JOIN_SUCCESS)),
                &addr,
                socket.clone(),
            );
            // Let this ip join the ip list
            if !is_joined_room(&addr, ips) {
                let join_message = protocol::Message {
                    code: Code::Message as u8,
                    message: JOIN_SUCCESS.green().bold().to_string(),
                    pro_id: protocol::ProtocolID {
                        id: message.pro_id.id.clone(),
                        protocol: protocol::get_protocol().unwrap(),
                    },
                };
                push_into_ips(&addr, ips);
                push_to_message_queue(&join_message, mess_que);
                send_message_to_all(&join_message, ips, socket);
            }
            // Send the join message to all ip
        // } else if compare_string(&message.message, &EXIT_ROOM.to_string()) {
        } else if message.message == EXIT_ROOM.to_string() {
            receive_exit(message, addr, mess_que, ips, socket);
        } else {
            send_message_to(
                &protocol::Message::new(Code::Reply as u8, &String::from(JOIN_FAILED)),
                &addr,
                socket.clone(),
            );
        }
    }
}

fn receive_reply(
    message: &protocol::Message,
    ips: &mut Vec<SocketAddr>,
    addr: SocketAddr,
    mess_que: Arc<Mutex<VecDeque<protocol::Message>>>,
) {
    if !is_room_owner() {
        if message.message == JOIN_SUCCESS {
            push_into_ips(&addr, ips);
            unsafe {
                ROOM_OWNER_IP = addr.to_string();
            }
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

fn receive_exit(
    message: &protocol::Message,
    addr: SocketAddr,
    mess_que: Arc<Mutex<VecDeque<protocol::Message>>>,
    ips: &mut Vec<SocketAddr>,
    socket: Arc<UdpSocket>,
) {
    let mut message = message.clone();
    message.code = Code::Message as u8;
    message.message = EXIT_ROOM.clone().red().bold().to_string();
    push_to_message_queue(&message, mess_que);
    if is_room_owner() && addr != socket.local_addr().unwrap() {
        ips.remove(find_ip(&addr, &ips).unwrap());
        send_message_to_all(&message, ips, socket);
    }
}

fn push_to_message_queue(
    message: &protocol::Message,
    mess_que: Arc<Mutex<VecDeque<protocol::Message>>>,
) {
    let mut lock_messages = mess_que.lock().unwrap();
    lock_messages.push_back(message.clone());
}

pub fn send_message_to(message: &protocol::Message, addr: &SocketAddr, socket: Arc<UdpSocket>) {
    socket.send_to(&message.to_buf(), addr).unwrap();
}

fn send_message_to_all(message: &protocol::Message, ips: &Vec<SocketAddr>, socket: Arc<UdpSocket>) {
    for ip in ips.iter() {
        if socket.local_addr().unwrap().to_string() != ip.to_string() {
            send_message_to(&message, ip, socket.clone());
        }
    }
}

/// Get local address method from [@egmkang](https://github.com/egmkang/local_ipaddress)
pub fn get_local_addr() -> io::Result<SocketAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    socket.connect("8.8.8.8:80")?;

    socket.local_addr()
}

fn push_into_ips(ip: &SocketAddr, ips: &mut Vec<SocketAddr>) -> bool {
    if is_joined_room(ip, ips) {
        return false;
    }
    ips.push(ip.clone());
    true
}

fn is_joined_room(ip: &SocketAddr, ips: &Vec<SocketAddr>) -> bool {
    if !is_room_owner() {
        unsafe {
            return ROOM_OWNER_IP == ip.to_string();
        }
    }
    for i_ip in ips.iter() {
        if ip.to_string() == i_ip.to_string() {
            return true;
        }
    }
    false
}

fn find_ip(ip: &SocketAddr, ips: &Vec<SocketAddr>) -> Option<usize> {
    for index in 0..ips.len() {
        if ip.to_string() == ips[index].to_string() {
            return Some(index);
        }
    }
    None
}
