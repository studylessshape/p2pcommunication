use super::prelude::*;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    queue,
    terminal::{self, ClearType},
};
use std::{
    collections::VecDeque,
    io::Write,
    net::UdpSocket,
    process::exit,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const DEFAULT_PROTOCOL: &'static str = "MOYU";

pub fn run() {
    buf::initialize();
    protocol::set_protocol(DEFAULT_PROTOCOL.to_string());
    input_identity();

    let identity = choose_owner();

    let mess_que = Arc::new(Mutex::new(VecDeque::<protocol::Message>::new()));
    let socket = Arc::new(UdpSocket::bind(server::get_local_addr().unwrap()).unwrap());

    let receive_thread = {
        let copy_mess_que = mess_que.clone();
        let copy_socket = socket.clone();
        thread::spawn(move || {
            server::receive(copy_socket, copy_mess_que);
        })
    };

    thread::sleep(Duration::from_millis(2000));
    buf::reset();
}

fn input_identity() -> String {
    let mut stdout = io::stdout();
    let mut id = String::new();

    loop {
        print!("Input your id > ");
        stdout.flush().unwrap();
        loop {
            match io::stdin().read_line(&mut id) {
                Ok(_) => {
                    id = id.trim_end().trim_start().to_string();
                }
                Err(e) => {
                    buf::print_error(&e);
                    exit(e.kind() as i32);
                }
            }
            protocol::set_id(&id);

            if protocol::Message::parse_id(&protocol::get_id().unwrap()) == "" {
                println!("Please input valid id(not all space and zero): ");
                stdout.flush().unwrap();
            } else {
                break;
            }
        }
        println!(
            "Enter to ensure your id(ESC to cancel): [{}]",
            protocol::get_id().unwrap()
        );
        stdout.flush().unwrap();
        let is_ensure;
        loop {
            let event = crossterm::event::read().unwrap();

            if event == Event::Key(KeyCode::Enter.into()) {
                is_ensure = true;
                break;
            } else if event == Event::Key(KeyCode::Esc.into()) {
                is_ensure = false;
                break;
            }
        }

        queue!(stdout, terminal::Clear(ClearType::All)).unwrap();
        stdout.flush().unwrap();

        if is_ensure {
            return id;
        }

        id.clear();
    }
}

fn choose_owner() -> server::Identity {
    let mut stdout = io::stdout();

    queue!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::Hide,
        cursor::MoveTo(0, 0),
    )
    .unwrap();

    println!("Choose:");
    println!("1.Create room");
    println!("2.Join room");
    loop {
        if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
            match code {
                KeyCode::Char('1') => {
                    server::set_identity(server::Identity::RoomOwner);
                    break;
                }
                KeyCode::Char('2') => {
                    server::set_identity(server::Identity::RooomJoiner);
                    break;
                }
                _ => {}
            };
        }
    }

    server::get_identity()
}

fn join_room() {}
