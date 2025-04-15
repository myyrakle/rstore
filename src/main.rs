use std::{io::Write, net::TcpStream};

extern crate serde;
use serde::Serialize;

#[derive(Serialize)]
pub struct Foo {
    pub user_id: String,
    pub user_name: String,
}

fn main() {
    let mut stream = TcpStream::connect("0.0.0.0:13535").unwrap();

    let buffer = [0; 1024];

    stream.write_all(buffer.as_slice()).unwrap();
}
