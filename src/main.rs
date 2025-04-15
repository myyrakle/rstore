use std::{io::Write, net::TcpStream};

extern crate serde;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Foo {
    pub user_id: String,
    pub user_name: String,
}

fn main() {
    let mut stream = TcpStream::connect("0.0.0.0:13535").unwrap();

    let mut buffer = [0; 1024];

    stream.write_all(buffer.as_slice()).unwrap();
}
