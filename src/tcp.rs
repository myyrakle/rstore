use chorba::{decode, encode};
use engine::KVEngine;
use protocol::{
    CLEAR, CLEAR_OK, DELETE, DELETE_OK, DeleteRequest, ERROR, GET, GET_OK, GetRequest, GetResponse,
    PACKET_INVALID, PING, PONG, PacketError, SET, SET_OK, SetRequest, generate_packet,
    read_all_from_stream,
};
use tokio::{io::AsyncWriteExt, net::TcpStream};

mod engine;
pub mod protocol;

#[tokio::main]
async fn main() {
    let engine = KVEngine::new();

    let address = "0.0.0.0:13535";
    log::debug!("Listening on {}", address);

    // 1. 서버 시작
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    loop {
        // 2. 클라이언트 연결 수신 (단일 세션)
        if let Ok((tcp_stream, socket_address)) = listener.accept().await {
            log::debug!("Accepted connection from {}", socket_address);

            let engine = engine.clone();

            tokio::spawn(async move {
                handle_stream(tcp_stream, engine).await;
            });
        } else {
            log::error!("Failed to accept connection");
        }
    }
}

async fn handle_stream(mut tcp_stream: TcpStream, mut engine: KVEngine) {
    loop {
        // 3. 클라이언트로부터 패킷 수신
        let (tag, bytes) = match read_all_from_stream(&mut tcp_stream).await {
            Ok(packet) => packet,
            Err(PacketError::NoDataReceived) => {
                log::debug!("No data received");
                return;
            }
            Err(error) => {
                log::error!("Failed to fetch packet: {}", error);
                let _ = tcp_stream.write_all(&[PACKET_INVALID]).await;
                continue;
            }
        };

        match tag {
            PING => {
                log::debug!("Received PING");

                if let Err(error) = tcp_stream.write_all(&[PONG]).await {
                    log::error!("Failed to send PONG: {}", error);
                    continue;
                }
            }
            SET => {
                log::debug!("Received SET");

                process_set(&mut tcp_stream, &mut engine, &bytes).await;
            }
            GET => {
                log::debug!("Received GET");

                process_get(&mut tcp_stream, &mut engine, &bytes).await;
            }
            DELETE => {
                log::debug!("Received DELETE");

                process_delete(&mut tcp_stream, &mut engine, &bytes).await;
            }
            CLEAR => {
                log::debug!("Received CLEAR");

                if let Err(error) = engine.clear_all() {
                    log::error!("Failed to clear all key-value pairs: {}", error);
                    let _ = tcp_stream.write_all(&[ERROR]).await;
                    continue;
                }

                // Send a response back to the client
                let _ = tcp_stream.write_all(&[CLEAR_OK]).await;
            }
            _ => {
                log::error!("Unknown command: {}", tag);

                let _ = tcp_stream.write_all(&[PACKET_INVALID]).await;
                continue;
            }
        }
    }
}

pub async fn process_set(stream: &mut TcpStream, engine: &mut KVEngine, bytes: &[u8]) {
    let decode_result = decode::<SetRequest>(bytes);

    let set_request = match decode_result {
        Ok(set_request) => set_request,
        Err(error) => {
            log::error!("Failed to decode SetRequest: {}", error);
            let _ = stream.write_all(&[PACKET_INVALID]).await;
            return;
        }
    };

    let key = set_request.key;
    let value = set_request.value;
    if let Err(error) = engine.set_key_value(key, value) {
        log::error!("Failed to set key-value pair: {}", error);
        let _ = stream.write_all(&[ERROR]).await;
    }

    // Send a response back to the client
    let _ = stream.write_all(&[SET_OK]).await;
}

pub async fn process_get(stream: &mut TcpStream, engine: &mut KVEngine, bytes: &[u8]) {
    let decode_result = decode::<GetRequest>(bytes);

    let get_request = match decode_result {
        Ok(get_request) => get_request,
        Err(error) => {
            log::error!("Failed to decode GetRequest: {}", error);
            let _ = stream.write_all(&[PACKET_INVALID]).await;
            return;
        }
    };

    let key = get_request.key;
    match engine.get_key_value(&key) {
        Ok(value) => {
            // Send the value back to the client
            let get_response = GetResponse { value };
            let response_bytes = encode(&get_response);

            let response = generate_packet(GET_OK, &response_bytes);
            let _ = stream.write_all(&response).await;
        }
        Err(error) => {
            log::error!("Failed to get key-value pair: {}", error);
            let _ = stream.write_all(&[ERROR]).await;
        }
    }
}

pub async fn process_delete(stream: &mut TcpStream, engine: &mut KVEngine, bytes: &[u8]) {
    let decode_result = decode::<DeleteRequest>(bytes);

    let get_request = match decode_result {
        Ok(get_request) => get_request,
        Err(error) => {
            log::error!("Failed to decode GetRequest: {}", error);
            let _ = stream.write_all(&[PACKET_INVALID]).await;
            return;
        }
    };

    let key = get_request.key;
    if let Err(error) = engine.delete_key_value(&key) {
        log::error!("Failed to delete key-value pair: {}", error);
        let _ = stream.write_all(&[ERROR]).await;
    }

    // Send a response back to the client
    let _ = stream.write_all(&[DELETE_OK]).await;
}
