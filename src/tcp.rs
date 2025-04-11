use std::sync::Arc;

use engine::KVEngine;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, tcp},
};

mod engine;

// Redis - 512MB (Key, Value)
// Memcached - 1MB (Key, Value)
const KEY_BYTE_LIMIT: usize = 1024 * 1024; // 1MB
const VALUE_BYTE_LIMIT: usize = 1024 * 1024 * 10; // 10MB

const PAYLOAD_CHUNK_SIZE: usize = 1024; // 1KB

// Request Tag - Start Byte
const PING: u8 = 0x01;
const SET: u8 = 0x02;
const GET: u8 = 0x03;
const DELETE: u8 = 0x04;
const CLEAR: u8 = 0x05;

// Response Tag - Start Byte
const PONG: u8 = 0x01;
const SET_OK: u8 = 0x02;
const GET_OK: u8 = 0x03;
const DELETE_OK: u8 = 0x04;
const CLEAR_OK: u8 = 0x05;
const ERROR: u8 = 0xFF;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StreamStatus {
    NONE,
    SET,
    GET,
    DELETE,
}

impl Default for StreamStatus {
    fn default() -> Self {
        StreamStatus::NONE
    }
}

#[tokio::main]
async fn main() {
    let engine = KVEngine::new();

    let address = "0.0.0.0:13535";
    println!("Listening on {}", address);

    // 1. 서버 시작
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    loop {
        // 2. 클라이언트 연결 수신 (단일 세션)
        if let Ok((tcp_stream, socket_address)) = listener.accept().await {
            println!("Accepted connection from {}", socket_address);

            let engine = engine.clone();

            tokio::spawn(async move {
                handle_stream(tcp_stream, engine).await;
            });
        } else {
            eprintln!("Failed to accept connection");
        }
    }
}

async fn handle_stream(mut tcp_stream: TcpStream, engine: KVEngine) {
    let mut read_buffer: [u8; PAYLOAD_CHUNK_SIZE] = [0; PAYLOAD_CHUNK_SIZE];

    let mut state = StreamStatus::default();

    loop {
        // 3. 클라이언트로부터 패킷 수신
        let stream_result = tcp_stream.read(&mut read_buffer).await;

        // 4. ... 패킷을 읽어서 분석

        // 5. 클라이언트에게 응답을 보냄
        let result = "OK";
        tcp_stream.write_all(result.as_bytes()).await.unwrap();

        let size = match stream_result {
            Ok(size) => size,
            Err(error) => {
                eprintln!("Failed to read from socket: {}", error);
                break;
            }
        };

        match state {
            StreamStatus::NONE => {
                if size == 0 {
                    println!("No data received");
                    continue;
                }

                println!("Received {:?} bytes", read_buffer);

                let first_byte = read_buffer[0];

                match first_byte {
                    PING => {
                        println!("Received PING");

                        if let Err(error) = tcp_stream.write(&[PONG]).await {
                            eprintln!("Failed to send PONG: {}", error);
                            continue;
                        }
                    }
                    SET => {
                        println!("Received SET");
                    }
                    GET => {
                        println!("Received GET");
                        // Handle GET
                        // Parse key from buffer
                    }
                    DELETE => {
                        println!("Received DELETE");
                        // Handle DELETE
                        // Parse key from buffer
                    }
                    CLEAR => {
                        println!("Received CLEAR");
                        // Handle CLEAR
                    }
                    _ => {
                        eprintln!("Unknown command: {}", first_byte);
                    }
                }
            }
            StreamStatus::SET => {
                println!("Stream status: SET");
            }
            StreamStatus::GET => {
                println!("Stream status: GET");
            }
            StreamStatus::DELETE => {
                println!("Stream status: DELETE");
            }
        }
    }
}
