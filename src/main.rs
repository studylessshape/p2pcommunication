use std::{collections::VecDeque, io::stdin, sync::{Arc, Mutex}, thread, time::Duration};

use communication::prelude::*;

fn main() {
    buf::initialize();
    protocol::set_protocol(&"MOYU".to_string());

    let mut input = String::new();
    buf::print_input(&input);

    {
        for i in 0..15 {
            buf::push_message(&format!("Value: {}", i));
        }

        buf::push_message(&protocol::Message::parse(&"MOYU1230000000001Hello".to_string()).unwrap().to_string());
    }

    buf::print_message();
    stdin().read_line(&mut input);
    buf::print_input(&input);
    thread::sleep(Duration::from_millis(3000));
    buf::clear_buf();
}