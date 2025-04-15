use chorba::decode;
use engine::KVEngine;
use protocol::{
    CLEAR, DELETE, DeleteRequest, ERROR, GET, GET_OK, GetRequest, PACKET_BYTE_LIMIT,
    PACKET_INVALID, PAYLOAD_CHUNK_SIZE, PAYLOAD_FIRST_MAX_VALUE_SIZE, PING, PONG, SET, SetRequest,
    parse_start_packet,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

mod engine;
mod protocol;

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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum StreamStatus {
    #[default]
    NONE,
    SET(u32),
    GET(u32),
    DELETE(u32),
}

async fn handle_stream(mut tcp_stream: TcpStream, mut engine: KVEngine) {
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

                        let start_packet = parse_start_packet(read_buffer);

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

                                process_set(&mut tcp_stream, &mut engine, packet.value).await;
                            }
                            None => {
                                let _ = tcp_stream.write(&[PACKET_INVALID]).await;
                            }
                        }
                    }
                    GET => {
                        println!("Received GET");

                        let start_packet = parse_start_packet(read_buffer);

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

                                process_get(&mut tcp_stream, &mut engine, packet.value).await;
                            }
                            None => {
                                let _ = tcp_stream.write(&[PACKET_INVALID]).await;
                            }
                        }
                    }
                    DELETE => {
                        println!("Received DELETE");

                        let start_packet = parse_start_packet(read_buffer);

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
                                process_delete(&mut tcp_stream, &mut engine, packet.value).await;
                            }
                            None => {
                                let _ = tcp_stream.write(&[PACKET_INVALID]).await;
                            }
                        }
                    }
                    CLEAR => {
                        println!("Received CLEAR");

                        if let Err(error) = engine.clear_all() {
                            eprintln!("Failed to clear all key-value pairs: {}", error);
                            let _ = tcp_stream.write(&[ERROR]).await;
                            continue;
                        }
                    }
                    _ => {
                        eprintln!("Unknown command: {}", first_byte);
                    }
                }
            }
            StreamStatus::SET(length) => {
                println!("Stream status: SET");

                if read_buffer.is_empty() {
                    println!("No data received");
                    state = StreamStatus::NONE;

                    continue;
                }

                context_buffer.extend_from_slice(read_buffer);

                if context_buffer.len() as u32 >= length {
                    // Process the value
                    process_set(&mut tcp_stream, &mut engine, &context_buffer).await;

                    // Reset state
                    state = StreamStatus::NONE;
                } else {
                    println!("Waiting for more data...");
                }
            }
            StreamStatus::GET(_) => {
                println!("Stream status: GET");

                if read_buffer.is_empty() {
                    println!("No data received");
                    state = StreamStatus::NONE;

                    continue;
                }

                context_buffer.extend_from_slice(read_buffer);

                if context_buffer.len() as u32 >= PAYLOAD_FIRST_MAX_VALUE_SIZE {
                    // Process the value
                    process_get(&mut tcp_stream, &mut engine, &context_buffer).await;

                    // Reset state
                    state = StreamStatus::NONE;
                } else {
                    println!("Waiting for more data...");
                }
            }
            StreamStatus::DELETE(_) => {
                println!("Stream status: DELETE");

                if read_buffer.is_empty() {
                    println!("No data received");
                    state = StreamStatus::NONE;

                    continue;
                }

                context_buffer.extend_from_slice(read_buffer);

                if context_buffer.len() as u32 >= PAYLOAD_FIRST_MAX_VALUE_SIZE {
                    // Process the value
                    process_delete(&mut tcp_stream, &mut engine, &context_buffer).await;

                    // Reset state
                    state = StreamStatus::NONE;
                } else {
                    println!("Waiting for more data...");
                }
            }
        }
    }
}

pub async fn process_set(stream: &mut TcpStream, engine: &mut KVEngine, bytes: &[u8]) {
    let decode_result = decode::<SetRequest>(bytes);

    let set_request = match decode_result {
        Ok(set_request) => set_request,
        Err(error) => {
            eprintln!("Failed to decode SetRequest: {}", error);
            let _ = stream.write(&[PACKET_INVALID]).await;
            return;
        }
    };

    let key = set_request.key;
    let value = set_request.value;
    if let Err(error) = engine.set_key_value(key, value) {
        eprintln!("Failed to set key-value pair: {}", error);
        let _ = stream.write(&[ERROR]).await;
    }
}

pub async fn process_get(stream: &mut TcpStream, engine: &mut KVEngine, bytes: &[u8]) {
    let decode_result = decode::<GetRequest>(bytes);

    let get_request = match decode_result {
        Ok(get_request) => get_request,
        Err(error) => {
            eprintln!("Failed to decode GetRequest: {}", error);
            let _ = stream.write(&[PACKET_INVALID]).await;
            return;
        }
    };

    let key = get_request.key;
    match engine.get_key_value(&key) {
        Ok(value) => {
            // Send the value back to the client
            let response = [GET_OK, value.len() as u8].to_vec();
            stream.write_all(&response).await.unwrap();
        }
        Err(error) => {
            eprintln!("Failed to get key-value pair: {}", error);
            let _ = stream.write(&[ERROR]).await;
        }
    }
}

pub async fn process_delete(stream: &mut TcpStream, engine: &mut KVEngine, bytes: &[u8]) {
    let decode_result = decode::<DeleteRequest>(bytes);

    let get_request = match decode_result {
        Ok(get_request) => get_request,
        Err(error) => {
            eprintln!("Failed to decode GetRequest: {}", error);
            let _ = stream.write(&[PACKET_INVALID]).await;
            return;
        }
    };

    let key = get_request.key;
    if let Err(error) = engine.delete_key_value(&key) {
        eprintln!("Failed to delete key-value pair: {}", error);
        let _ = stream.write(&[ERROR]).await;
    }
}
