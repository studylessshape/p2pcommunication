use std::{
    collections::VecDeque,
    io::{self, Write},
    sync::Mutex,
};

use crossterm::{
    cursor, queue,
    terminal::{self, ClearType},
};

const MESSAGE_BUF: u16 = 20;

lazy_static! {
    static ref MESSAGES: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
}

pub fn initialize() {
    enter_alternate_screen();
}

pub fn reset() {
    clear_buf();
    leave_alternate_screen();
}

pub fn clear_buf() {
    let mut lock_messages = MESSAGES.lock().unwrap();
    lock_messages.clear();
}

pub fn enter_alternate_screen() {
    let mut stdout = io::stdout();
    queue!(stdout, terminal::EnterAlternateScreen,).unwrap();
    stdout.flush().unwrap();
}

pub fn leave_alternate_screen() {
    let mut stdout = io::stdout();
    queue!(stdout, terminal::LeaveAlternateScreen,).unwrap();
    stdout.flush().unwrap();
}

pub fn print_message() {
    let mut stdout = io::stdout();
    queue!(
        stdout,
        cursor::MoveTo(0, MESSAGE_BUF),
        terminal::Clear(ClearType::FromCursorUp),
        terminal::Clear(ClearType::CurrentLine),
    )
    .unwrap();
    queue!(stdout, cursor::MoveTo(0, 0)).unwrap();

    let lock_messages = MESSAGES.lock().unwrap();

    for message in lock_messages.iter() {
        println!("{}", message);
    }

    stdout.flush().unwrap();
}

pub fn println(s: &String, line: u16) {
    let mut stdout = io::stdout();
    queue!(
        stdout,
        cursor::MoveTo(0, line),
        terminal::Clear(ClearType::CurrentLine),
    )
    .unwrap();
    println!("{}", s);
    stdout.flush().unwrap();
}

pub fn print_input(head: &String, input: &String, line: u16) {
    let mut stdout = io::stdout();
    queue!(
        stdout,
        cursor::MoveTo(0, line),
        terminal::Clear(ClearType::CurrentLine),
    )
    .unwrap();

    println!("{}{}", head, input);
    stdout.flush().unwrap();
}

pub fn push_message(message: &String) {
    let mut lock_message = MESSAGES.lock().unwrap();
    if lock_message.len() > MESSAGE_BUF as usize {
        lock_message.pop_front();
    }
    lock_message.push_back(message.clone());
}

pub fn print_error<E: ToString>(err: &E) {
    reset();
    let mut stdout = io::stdout();
    println!("{}", err.to_string());
    stdout.flush().unwrap();
}

pub fn clear_all() {
    let mut stdout = io::stdout();
    queue!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .unwrap();
}
