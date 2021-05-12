use std::{collections::VecDeque, io::{self, Stdout, Write}, net::{SocketAddr, UdpSocket}, process::exit, str::FromStr, sync::{Arc, Mutex, MutexGuard}, thread, time::Duration, usize};

use crossterm::{
    cursor,
    event::{self, poll, Event, KeyCode, KeyEvent},
    queue,
    terminal::{self, ClearType, EnterAlternateScreen},
    QueueableCommand,
};

const ID_LEN: u32 = 20;
const MESSAGE_LEN: u32 = 20;

fn main() {
    let mut stdout = Arc::new(Mutex::new(io::stdout()));

    {
        let mut lock_stdout = stdout.lock().unwrap();

        lock_stdout.queue(EnterAlternateScreen).unwrap();
        lock_stdout.flush().unwrap();
    }

    let local_addr = get_local_addr().unwrap();
    let socket = Arc::new(UdpSocket::bind(local_addr).unwrap());

    let (id, ip) = initialize_iden(&local_addr, &mut stdout);

    {
        let mut lock_stdout = stdout.lock().unwrap();
        println!("Success! Will connect ip: {}", ip);
        println!("Stat connect.......");
        lock_stdout.flush().unwrap();
    }

    {
        let mut lock_stdout = stdout.lock().unwrap();
        match socket.connect(ip) {
            Ok(_) => {
                println!("Connect Success! Will enter communication view!");
                lock_stdout.flush().unwrap();
            }
            Err(e) => exit_process(-1, &mut lock_stdout, e),
        };
    }

    let mut messages = Arc::new(Mutex::new(VecDeque::<String>::new()));

    let arc_socket = socket.clone();
    let mut mut_messages = messages.clone();
    let imut_messages = messages.clone();
    let mut stdout_c = stdout.clone();
    thread::spawn(move || {
        receive_message(arc_socket, &mut mut_messages, &mut stdout_c);
    });

    let mut input = String::new();

    {
        let mut lock_stdout = stdout.lock().unwrap();
        queue!(*lock_stdout, terminal::Clear(ClearType::All),).unwrap();
    }
    print_input(&local_addr, &input, &mut stdout);

    loop {
        if poll(Duration::from_millis(10)).unwrap() {
            if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                match code {
                    KeyCode::Enter => {
                        let input_message = input.trim_end().to_string();
                        if input_message.eq("") {
                            continue;
                        }
                        if input_message.eq_ignore_ascii_case("exit") {
                            break;
                        }
                        send_meesage(format!("{}{}", id, input_message), &ip, socket.clone())
                            .unwrap();
                        push_message(format!("{}:\t{}", id, input_message), &mut messages);
                        input.clear();
                        print_message(imut_messages.clone(), &mut stdout);
                        print_input(&local_addr, &input, &mut stdout);
                    }
                    KeyCode::Char(c) => {
                        input.push(c);
                        print_input(&local_addr, &input, &mut stdout);
                    }
                    KeyCode::Backspace => {
                        input.pop();
                        print_input(&local_addr, &input, &mut stdout);
                    }
                    _ => {}
                };
            }
        }
    }

    {
        let mut lock_stdout = stdout.lock().unwrap();
        queue!(*lock_stdout, terminal::LeaveAlternateScreen).unwrap();
    }

    stdout.lock().unwrap().flush().unwrap();
}

/// Return `(id, target_ip)`
fn initialize_iden(local_ip: &SocketAddr, stdout: &mut Arc<Mutex<Stdout>>) -> (String, SocketAddr) {
    let mut id = String::new();
    let mut target_ip = String::new();
    let mut ip;

    let mut lock_stdout = stdout.lock().unwrap();

    let stdin = io::stdin();
    loop {
        queue!(
            *lock_stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0,0),
            cursor::Show
        ).unwrap();
        println!("Your local ip: {}", local_ip.to_string());
        print!("Input your id > ");
        lock_stdout.flush().unwrap();

        match stdin.read_line(&mut id) {
            Ok(_) => id = id.trim_end().to_string(),
            Err(e) => exit_process(-1, &mut lock_stdout.into(), e),
        };

        let mut count = id.len() as i32 - ID_LEN as i32;

        if count < 0 {
            while count < 0 {
                id.push('\0');
                count += 1;
            }
        } else if count > 0 {
            while count > 0 {
                id.pop();
                count -= 1;
            }
        }

        print!("Enter ip to conncect > ");
        (*lock_stdout).flush().unwrap();

        match stdin.read_line(&mut target_ip) {
            Ok(_) => {
                ip = match SocketAddr::from_str(target_ip.trim()) {
                    Ok(ipaddr) => ipaddr,
                    Err(e) => exit_process(-1, &mut lock_stdout.into(), e),
                };
            }
            Err(e) => exit_process(-1, &mut lock_stdout.into(), e),
        };

        queue!(
            *lock_stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Hide,
        )
        .unwrap();
        println!("///////////Press \'Enter\' to ensure your profile, And \'ESC\' quit//////////");
        println!("\t\t\t\tYour id: {}", id);
        println!("\t\t\tTarget ip: {}", target_ip);
        println!("\t\tYour local ip: {}", local_ip.to_string());

        lock_stdout.flush().unwrap();

        let is_ensure;
        loop {
            let event = crossterm::event::read().unwrap();

            if event == Event::Key(KeyCode::Enter.into()) {
                is_ensure = true;
                break;
            }else if event == Event::Key(KeyCode::Esc.into()){
                is_ensure = false;
                break;
            }
        }

        queue!(
            *lock_stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0),
        )
        .unwrap();
        lock_stdout.flush().unwrap();

        if is_ensure {
            break;
        }
    }

    (id, ip)
}

fn print_message(messages: Arc<Mutex<VecDeque<String>>>, stdout: &mut Arc<Mutex<Stdout>>) {
    let mut lock_stdout = stdout.lock().unwrap();
    queue!(
        *lock_stdout,
        cursor::MoveTo(0, 20),
        terminal::Clear(ClearType::FromCursorUp),
        cursor::MoveTo(0, 0)
    )
    .unwrap();

    for message in (*messages.lock().unwrap()).iter() {
        println!("{}", message);
    }

    lock_stdout.flush().unwrap();
}

fn print_input(local_ip: &SocketAddr ,input: &String, stdout: &mut Arc<Mutex<Stdout>>) {
    let mut lock_stdout = stdout.lock().unwrap();
    queue!(
        *lock_stdout,
        cursor::MoveTo(0, 22),
        terminal::Clear(ClearType::CurrentLine),
    )
    .unwrap();
    println!("Input message: {}", input);
    println!("\n(Your ip: {})", local_ip.to_string());
    lock_stdout.flush().unwrap();
}

fn send_meesage(s: String, addr: &SocketAddr, socket: Arc<UdpSocket>) -> io::Result<usize> {
    socket.send_to(s.as_bytes(), addr)
}

fn receive_message(
    socket: Arc<UdpSocket>,
    messages: &mut Arc<Mutex<VecDeque<String>>>,
    stdout: &mut Arc<Mutex<Stdout>>,
)  {
    let mut buf = [0u8; 1500];
    loop {
        if let Ok(_) = socket.recv(&mut buf) {
            let message = String::from_utf8_lossy(&buf).to_string();
            let id = &message[..20];
            let message = &message[20..];
            push_message(format!("{}:\t{}", id, message), messages);
            let mut stdout_clone = stdout.clone();
            print_message(messages.clone(), &mut stdout_clone);
        }
    }
}

fn push_message(message: String, messages: &mut Arc<Mutex<VecDeque<String>>>) {
    let mut mut_mess = match messages.lock() {
        Ok(lock) => lock,
        Err(_) => return,
    };
    (*mut_mess).push_back(message);
    if mut_mess.len() > MESSAGE_LEN as usize {
        mut_mess.pop_front();
    }
}

fn get_local_addr() -> io::Result<SocketAddr> {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(e) => return Err(e),
    };

    match socket.connect("8.8.8.8:80") {
        Ok(()) => (),
        Err(e) => return Err(e),
    };

    socket.local_addr()
}

fn exit_process<E: ToString>(code: i32, stdout: &mut MutexGuard<Stdout>, error: E) -> ! {
    queue!(*stdout, terminal::LeaveAlternateScreen).unwrap();
    stdout.flush().unwrap();
    io::stderr()
        .write(format!("Err: {}\n", error.to_string()).as_bytes())
        .unwrap();
    exit(code)
}
