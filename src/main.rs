use std::{io::Write, net::TcpStream};

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect("0.0.0.0:13535").unwrap();

    let mut buffer = [0; 1024];

    stream.write_all(buffer.as_slice()).unwrap();
}
