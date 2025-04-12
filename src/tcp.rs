use std::sync::Arc;

use engine::KVEngine;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, tcp},
};

mod engine;

// Redis - 512MB (Key, Value)
// Memcached - 1MB (Key, Value)
const KEY_BYTE_LIMIT: u32 = 1024 * 1024; // 1MB
const VALUE_BYTE_LIMIT: u32 = 1024 * 1024 * 10; // 10MB
const PACKET_BYTE_LIMIT: u32 = 1024 * 1024 * 20; // 20MB

const PAYLOAD_CHUNK_SIZE: u32 = 1024; // 1KB
const PAYLOAD_HEAD_SIZE: u32 = 5; // Tag 1 Byte + Length 4 Bytes
const PAYLOAD_FIRST_MAX_VALUE_SIZE: u32 = PAYLOAD_CHUNK_SIZE - PAYLOAD_HEAD_SIZE; // 1KB - 5 Bytes

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
const PACKET_INVALID: u8 = 0xFE;
const ERROR: u8 = 0xFF;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StreamStatus {
    NONE,
    SET(u32),
    GET(u32),
    DELETE(u32),
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
    let mut read_buffer: [u8; PAYLOAD_CHUNK_SIZE as usize] = [0; PAYLOAD_CHUNK_SIZE as usize];

    let mut state = StreamStatus::default();

    let mut context_buffer: Vec<u8> = Vec::new();

    loop {
        // 3. 클라이언트로부터 패킷 수신
        let stream_result = tcp_stream.read(&mut read_buffer).await;

        // 4. ... 패킷을 읽어서 분석

        let size = match stream_result {
            Ok(size) => size,
            Err(error) => {
                eprintln!("Failed to read from socket: {}", error);
                break;
            }
        };

        let read_buffer = &read_buffer[..size];

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

                        let start_packet = parse_start_packet(&read_buffer);

                        match start_packet {
                            Some(packet) => {
                                if packet.length > PACKET_BYTE_LIMIT {
                                    eprintln!("packet size exceeds limit");
                                    let _ = tcp_stream.write(&[PACKET_INVALID]).await;
                                    continue;
                                }

                                if packet.length > PAYLOAD_FIRST_MAX_VALUE_SIZE {
                                    context_buffer.extend_from_slice(packet.value);
                                    state = StreamStatus::SET(packet.length);

                                    continue;
                                }

                                process_set(&mut tcp_stream, packet.value);
                            }
                            None => {
                                let _ = tcp_stream.write(&[PACKET_INVALID]).await;
                            }
                        }
                    }
                    GET => {
                        println!("Received GET");

                        let start_packet = parse_start_packet(&read_buffer);

                        match start_packet {
                            Some(packet) => {
                                if packet.length > PACKET_BYTE_LIMIT {
                                    eprintln!("packet size exceeds limit");
                                    let _ = tcp_stream.write(&[PACKET_INVALID]).await;
                                    continue;
                                }

                                if packet.length > PAYLOAD_FIRST_MAX_VALUE_SIZE {
                                    context_buffer.extend_from_slice(packet.value);
                                    state = StreamStatus::GET(packet.length);

                                    continue;
                                }

                                process_get(&mut tcp_stream, packet.value);
                            }
                            None => {
                                let _ = tcp_stream.write(&[PACKET_INVALID]).await;
                            }
                        }
                    }
                    DELETE => {
                        println!("Received DELETE");

                        let start_packet = parse_start_packet(&read_buffer);

                        match start_packet {
                            Some(packet) => {
                                if packet.length > PACKET_BYTE_LIMIT {
                                    eprintln!("packet size exceeds limit");
                                    let _ = tcp_stream.write(&[PACKET_INVALID]).await;
                                    continue;
                                }

                                if packet.length > PAYLOAD_FIRST_MAX_VALUE_SIZE {
                                    context_buffer.extend_from_slice(packet.value);
                                    state = StreamStatus::DELETE(packet.length);

                                    continue;
                                }

                                // Process the value
                                process_delete(&mut tcp_stream, packet.value);
                            }
                            None => {
                                let _ = tcp_stream.write(&[PACKET_INVALID]).await;
                            }
                        }
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
            StreamStatus::SET(length) => {
                println!("Stream status: SET");

                if read_buffer.len() == 0 {
                    println!("No data received");
                    state = StreamStatus::NONE;

                    continue;
                }

                context_buffer.extend_from_slice(read_buffer);

                if context_buffer.len() as u32 >= length {
                    // Process the value
                    process_set(&mut tcp_stream, &context_buffer);

                    // Reset state
                    state = StreamStatus::NONE;
                } else {
                    println!("Waiting for more data...");
                }
            }
            StreamStatus::GET(_) => {
                println!("Stream status: GET");

                if read_buffer.len() == 0 {
                    println!("No data received");
                    state = StreamStatus::NONE;

                    continue;
                }

                context_buffer.extend_from_slice(read_buffer);

                if context_buffer.len() as u32 >= PAYLOAD_FIRST_MAX_VALUE_SIZE {
                    // Process the value
                    process_get(&mut tcp_stream, &context_buffer);

                    // Reset state
                    state = StreamStatus::NONE;
                } else {
                    println!("Waiting for more data...");
                }
            }
            StreamStatus::DELETE(_) => {
                println!("Stream status: DELETE");

                if read_buffer.len() == 0 {
                    println!("No data received");
                    state = StreamStatus::NONE;

                    continue;
                }

                context_buffer.extend_from_slice(read_buffer);

                if context_buffer.len() as u32 >= PAYLOAD_FIRST_MAX_VALUE_SIZE {
                    // Process the value
                    process_delete(&mut tcp_stream, &context_buffer);

                    // Reset state
                    state = StreamStatus::NONE;
                } else {
                    println!("Waiting for more data...");
                }
            }
        }
    }
}

pub struct StartPacket<'a> {
    pub tag: u8,
    pub length: u32,
    pub value: &'a [u8],
}

pub fn parse_start_packet<'a>(packet: &'a [u8]) -> Option<StartPacket<'a>> {
    if packet.len() < 5 {
        return None;
    }

    let tag = packet[0];
    let length = u32::from_be_bytes([packet[1], packet[2], packet[3], packet[4]]);
    let value = &packet[5..];

    Some(StartPacket { tag, length, value })
}

pub fn process_set(stream: &mut TcpStream, bytes: &[u8]) {
    // TODO implement set
}

pub fn process_get(stream: &mut TcpStream, bytes: &[u8]) {
    // TODO implement get
}

pub fn process_delete(stream: &mut TcpStream, bytes: &[u8]) {
    // TODO implement get
}
