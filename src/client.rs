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
    net::{SocketAddr, UdpSocket},
    process::exit,
    str::FromStr,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const DEFAULT_PROTOCOL: &'static str = "MOYU";
const EXIT_COMMAND: &'static str = "exit";
const KEY_MAX_LEN: usize = 16;
const TIMEOUT_COUNT: usize = 30;

pub fn run() {
    buf::initialize();
    protocol::set_protocol(DEFAULT_PROTOCOL.to_string());
    input_identity();

    let identity = choose_owner();
    let mess_que = Arc::new(Mutex::new(VecDeque::<protocol::Message>::new()));
    let socket = Arc::new(UdpSocket::bind(server::get_local_addr().unwrap()).unwrap());

    {
        let copy_mess_que = mess_que.clone();
        let copy_socket = socket.clone();
        thread::spawn(move || {
            server::receive(copy_socket, copy_mess_que);
        })
    };

    let send_addr = if identity.is_room_joiner() {
        join_room(mess_que.clone(), socket.clone())
    } else {
        server::set_key(input_key());
        server::send_message_to(
            &protocol::Message::new(server::Code::Request as u8, &server::get_key()),
            &socket.local_addr().unwrap(),
            socket.clone(),
        );
        socket.local_addr().unwrap()
    };
    communication(socket, send_addr, mess_que);
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
                    let len = if protocol::ID_LEN > id.len() {
                        id.len()
                    } else {
                        protocol::ID_LEN
                    };
                    id = id[..len].trim_end().trim_start().to_string();
                }
                Err(e) => {
                    buf::print_error(&e);
                    exit(e.kind() as i32);
                }
            }
            if id == EXIT_COMMAND {
                exit_client(0);
            } else if id.len() <= 0 {
                id = "None".to_string();
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
            protocol::Message::parse_id(&protocol::get_id().unwrap())
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

        queue!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )
        .unwrap();
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
    stdout.flush().unwrap();

    loop {
        if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
            match code {
                KeyCode::Char('1') => {
                    server::set_identity(server::Identity::RoomOwner);
                    break;
                }
                KeyCode::Char('2') => {
                    server::set_identity(server::Identity::RoomJoiner);
                    break;
                }
                _ => {}
            };
        }
    }

    server::get_identity()
}

fn join_room(
    mess_que: Arc<Mutex<VecDeque<protocol::Message>>>,
    socket: Arc<UdpSocket>,
) -> SocketAddr {
    let mut request_message =
        protocol::Message::new(server::Code::Request as u8, &protocol::get_id().unwrap());
    let mut stdout = io::stdout();
    queue!(stdout, terminal::Clear(ClearType::All),).unwrap();

    loop {
        let room_addr = input_ip();

        let key = input_key();
        request_message.message = key;
        print!("Join");

        let mut loading_count = 0;
        let join_flag = loop {
            server::send_message_to(&request_message, &room_addr, socket.clone());
            print!(".");
            stdout.flush().unwrap();

            let mut lock_mess_que = mess_que.lock().unwrap();

            match lock_mess_que.pop_front() {
                Some(message) => {
                    if message.message == server::JOIN_SUCCESS.to_string() {
                        break true;
                    } else if message.message == server::JOIN_FAILED.to_string() {
                        break false;
                    }
                }
                None => {}
            };
            loading_count += 1;
            if loading_count >= TIMEOUT_COUNT {
                break false;
            }
            thread::sleep(Duration::from_secs_f32(1.5));
        };

        if !join_flag {
            print!("\nTime out or False key!\nJoin faild!");
        } else {
            return room_addr;
        }
        stdout.flush().unwrap();
        thread::sleep(Duration::from_secs_f32(2.0));
    }
}

fn input_ip() -> SocketAddr {
    let out_head = String::from("Enter ip to join room > ");
    let mut input = String::new();
    let mut room_addr;
    let mut stdout = io::stdout();
    buf::clear_all();
    buf::print_input(&out_head, &input, 0);
    loop {
        if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
            match code {
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Enter => {
                    if input.len() <= 0 {
                        continue;
                    }
                    if input == EXIT_COMMAND {
                        exit_client(0);
                    }
                    room_addr = SocketAddr::from_str(&input);
                    input.clear();

                    if let Err(_) = room_addr {
                        println!("Please input valid ip!");
                        stdout.flush().unwrap();
                        thread::sleep(Duration::from_secs_f32(2.5));
                        queue!(stdout, terminal::Clear(ClearType::All)).unwrap();
                        stdout.flush().unwrap();
                        buf::print_input(&out_head, &input, 0);
                        continue;
                    }
                    break;
                }
                KeyCode::Char(c) => {
                    if c.to_string().trim() != "" {
                        input.push(c);
                    }
                }
                _ => {}
            };
            match code {
                KeyCode::Backspace | KeyCode::Char(_) | KeyCode::Enter => {
                    buf::print_input(&out_head, &input, 0);
                }
                _ => {}
            };
        }
    }
    room_addr.unwrap()
}

fn input_key() -> String {
    let head = String::from("Enter key > ");
    let mut input = String::new();
    buf::clear_all();
    buf::print_input(&head, &input, 0);
    loop {
        if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
            match code {
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Enter => {
                    if input.len() <= 0 {
                        continue;
                    }
                    break;
                }
                KeyCode::Char(c) => {
                    if c.to_string().trim() != "" {
                        if input.len() < KEY_MAX_LEN {
                            input.push(c);
                        }
                    }
                }
                _ => {}
            };

            match code {
                KeyCode::Backspace | KeyCode::Enter | KeyCode::Char(_) => {
                    buf::print_input(&head, &input, 0);
                }
                _ => {}
            };
        }
    }
    input.trim().to_string()
}

fn communication(
    socket: Arc<UdpSocket>,
    send_addr: SocketAddr,
    mess_que: Arc<Mutex<VecDeque<protocol::Message>>>,
) {
    buf::clear_all();

    let input_head = String::from("Input message > ");
    let ip_head = if server::is_room_owner() {
        format!(
            "(Your ip: {}, Key: {})",
            socket.local_addr().unwrap().to_string(),
            server::get_key()
        )
    } else {
        format!("(Your ip: {})", socket.local_addr().unwrap().to_string())
    };
    let mut input = String::new();
    let input_line = 22;
    let ter_size = terminal::size().unwrap().0 as usize;
    let input_size = if input_head.len() > ter_size {
        0
    }else{
        ter_size - input_head.len()
    };
    buf::print_input(&input_head, &input, input_line);
    buf::println(&ip_head, 24);
    loop {
        if let Some(message) = get_new_message(mess_que.clone()) {
            if let server::Code::Message = server::Code::from(message.code) {
                buf::push_message(&message.to_string());
                buf::print_message();
            }
        }
        if let Ok(true) = event::poll(Duration::from_millis(100)) {
            if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                match code {
                    KeyCode::Enter => {
                        input = input.trim_end().trim_start().to_string();
                        if input.len() > 0 {
                            if input == EXIT_COMMAND {
                                server::send_message_to(
                                    &protocol::Message::new(
                                        server::Code::Request as u8,
                                        &server::EXIT_ROOM.to_string(),
                                    ),
                                    &send_addr,
                                    socket.clone(),
                                );
                                exit_client(0);
                            }
                            server::send_message_to(
                                &protocol::Message::new(server::Code::Message as u8, &input),
                                &send_addr,
                                socket.clone(),
                            );
                            input.clear();
                        }
                    }
                    KeyCode::Backspace | KeyCode::Delete => {
                        input.pop();
                    }
                    KeyCode::Char(c) => {
                        if input.len() < input_size {
                            input.push(c);
                        }
                    }
                    _ => {}
                };

                buf::print_input(&input_head, &input, input_line);
            }
        }
    }
}

fn get_new_message(mess_que: Arc<Mutex<VecDeque<protocol::Message>>>) -> Option<protocol::Message> {
    let mut mess_lock = mess_que.lock().unwrap();
    mess_lock.pop_front()
}

fn exit_client(code: i32) {
    buf::reset();
    let mut stdout = io::stdout();
    queue!(stdout, cursor::Show).unwrap();
    exit(code);
}