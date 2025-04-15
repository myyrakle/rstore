use std::{
    sync::{Arc, Mutex, Weak},
    time::Duration,
};

use chorba::{decode, encode};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::protocol::{
    self, GetRequest, GetResponse, PACKET_BYTE_LIMIT, PAYLOAD_CHUNK_SIZE, parse_start_packet,
};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Connection error: {0}")]
    ConnectionError(#[from] std::io::Error),
    #[error("Send request error")]
    SendRequestError(std::io::Error),
}

pub type ClientResult<T> = std::result::Result<T, ClientError>;

#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: u16,
    pub min_connections: u32,         // 최소 유지 연결 수
    pub max_connections: u32,         // 최대 허용 연결 수
    pub connection_timeout: Duration, // 유휴 연결 타임아웃
    pub idle_timeout: Duration,       // 연결 최대 수명
}

const MIN_CONNECTION_DEFAULT: u32 = 1;
const MAX_CONNECTION_DEFAULT: u32 = 10;
const CONNECTION_TIMEOUT_DEFAULT: Duration = Duration::from_secs(30);
const IDLE_TIMEOUT_DEFAULT: Duration = Duration::from_secs(60);

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            min_connections: MIN_CONNECTION_DEFAULT,
            max_connections: MAX_CONNECTION_DEFAULT,
            connection_timeout: CONNECTION_TIMEOUT_DEFAULT,
            idle_timeout: IDLE_TIMEOUT_DEFAULT,
            host: "".into(),
            port: 0,
        }
    }
}

impl ConnectionConfig {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            min_connections: MIN_CONNECTION_DEFAULT,
            max_connections: MAX_CONNECTION_DEFAULT,
            connection_timeout: CONNECTION_TIMEOUT_DEFAULT,
            idle_timeout: IDLE_TIMEOUT_DEFAULT,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RStoreClient {
    connection_pool: Arc<Mutex<ConnectionPool>>,
    connection_config: ConnectionConfig,
}

impl RStoreClient {
    pub fn new(connection_config: ConnectionConfig) -> Self {
        let connection_pool = Arc::new(Mutex::new(ConnectionPool {
            connections: vec![],
            connection_count: 0,
        }));

        RStoreClient {
            connection_pool,
            connection_config,
        }
    }

    async fn create_connection(&self) -> ClientResult<PooledConnection> {
        let tcp_stream = TcpStream::connect(format!(
            "{}:{}",
            self.connection_config.host, self.connection_config.port
        ))
        .await?;

        {
            let mut pool = self.connection_pool.lock().unwrap();

            pool.connection_count += 1;
        }

        let pooled_connection =
            PooledConnection::new(tcp_stream, Arc::downgrade(&self.connection_pool));

        Ok(pooled_connection)
    }

    pub async fn connect(&self) -> ClientResult<()> {
        let connection_count = { self.connection_pool.lock().unwrap().connection_count };

        if connection_count == 0 {
            let new_connection = self.create_connection().await?;

            {
                let mut pool = self.connection_pool.lock().unwrap();
                pool.connections.push(new_connection);
            }
        }

        Ok(())
    }

    pub async fn get_connection_or_wait(&self) -> ClientResult<PooledConnection> {
        loop {
            let (connections_is_empty, connection_count) = {
                let pool = self.connection_pool.lock().unwrap();
                let connection_count = pool.connection_count;
                let connections_is_empty = pool.connections.is_empty();
                (connections_is_empty, connection_count)
            };

            if connections_is_empty && connection_count < self.connection_config.max_connections {
                let new_connection = self.create_connection().await?;
                return Ok(new_connection);
            }

            // Remove the connection from the pool if it's not valid
            {
                let mut pool = self.connection_pool.lock().unwrap();
                pool.connections
                    .retain(|c| c.tcp_stream.peer_addr().is_ok());
                pool.connection_count = pool.connections.len() as u32;

                if let Some(connection) = pool.connections.pop() {
                    return Ok(connection);
                }
            }

            std::thread::sleep(Duration::from_millis(100));

            // TODO: cancel with timeout
        }
    }

    pub async fn ping(&self) -> ClientResult<()> {
        let mut connection = self.get_connection_or_wait().await?;

        let _ = request_ping(&mut connection.tcp_stream).await?;

        connection.release_to_pool();

        Ok(())
    }

    pub async fn get(&self, request: GetRequest) -> ClientResult<GetResponse> {
        let mut connection = self.get_connection_or_wait().await?;

        let response = request_get(&mut connection.tcp_stream, request).await?;

        connection.release_to_pool();

        Ok(response)
    }

    pub async fn set(&self, request: protocol::SetRequest) -> ClientResult<()> {
        let mut connection = self.get_connection_or_wait().await?;

        let _ = request_set(&mut connection.tcp_stream, request).await?;

        connection.release_to_pool();

        Ok(())
    }

    pub async fn delete(&self, request: protocol::DeleteRequest) -> ClientResult<()> {
        let mut connection = self.get_connection_or_wait().await?;

        let _ = request_delete(&mut connection.tcp_stream, request).await?;

        connection.release_to_pool();

        Ok(())
    }

    pub async fn clear(&self) -> ClientResult<()> {
        let mut connection = self.get_connection_or_wait().await?;

        let _ = request_clear(&mut connection.tcp_stream).await?;

        connection.release_to_pool();

        Ok(())
    }
}

#[derive(Debug)]
pub struct ConnectionPool {
    connections: Vec<PooledConnection>,
    connection_count: u32,
}

#[derive(Debug)]
pub struct PooledConnection {
    tcp_stream: TcpStream,
    pool: Weak<Mutex<ConnectionPool>>,
}

impl PooledConnection {
    pub fn new(tcp_stream: TcpStream, pool: Weak<Mutex<ConnectionPool>>) -> Self {
        PooledConnection {
            tcp_stream,
            pool: pool,
        }
    }

    pub fn release_to_pool(self) {
        if let Some(pool) = self.pool.upgrade() {
            let mut pool = pool.lock().unwrap();
            pool.connections.push(self);
        }
    }
}

async fn request_ping(tcp_stream: &mut TcpStream) -> ClientResult<()> {
    // empty bytes array
    let request_bytes = vec![];

    let request_packet = generate_packet(protocol::PING, &request_bytes);

    tcp_stream.write(&request_packet).await?;

    let (response_tag, _) = fetch_all_packet(tcp_stream).await?;

    // if response_tag != protocol::PONG {
    //     return Err(ClientError::ConnectionError(std::io::Error::new(
    //         std::io::ErrorKind::InvalidData,
    //         "Ping Failed",
    //     )));
    // }

    Ok(())
}

async fn request_get(tcp_stream: &mut TcpStream, request: GetRequest) -> ClientResult<GetResponse> {
    let request_bytes = encode(&request);

    let request_packet = generate_packet(protocol::GET, &request_bytes);

    tcp_stream.write(&request_packet).await?;

    let (response_tag, response_bytes) = fetch_all_packet(tcp_stream).await?;

    if response_tag != protocol::GET_OK {
        return Err(ClientError::ConnectionError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid response tag",
        )));
    }

    let decoded = decode::<GetResponse>(&response_bytes).map_err(|_| {
        ClientError::ConnectionError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed to decode response",
        ))
    })?;

    Ok(decoded)
}

async fn request_set(
    tcp_stream: &mut TcpStream,
    request: protocol::SetRequest,
) -> ClientResult<()> {
    let request_bytes = encode(&request);

    let request_packet = generate_packet(protocol::SET, &request_bytes);

    tcp_stream.write(&request_packet).await?;

    let (response_tag, _) = fetch_all_packet(tcp_stream).await?;

    if response_tag != protocol::SET_OK {
        return Err(ClientError::ConnectionError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid response tag",
        )));
    }

    Ok(())
}

async fn request_delete(
    tcp_stream: &mut TcpStream,
    request: protocol::DeleteRequest,
) -> ClientResult<()> {
    let request_bytes = encode(&request);

    let request_packet = generate_packet(protocol::DELETE, &request_bytes);

    tcp_stream.write(&request_packet).await?;

    let (response_tag, _) = fetch_all_packet(tcp_stream).await?;

    if response_tag != protocol::DELETE_OK {
        return Err(ClientError::ConnectionError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid response tag",
        )));
    }

    Ok(())
}

async fn request_clear(tcp_stream: &mut TcpStream) -> ClientResult<()> {
    let request_bytes = vec![];

    let request_packet = generate_packet(protocol::CLEAR, &request_bytes);

    tcp_stream.write(&request_packet).await?;

    let (response_tag, _) = fetch_all_packet(tcp_stream).await?;

    if response_tag != protocol::CLEAR_OK {
        return Err(ClientError::ConnectionError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid response tag",
        )));
    }

    Ok(())
}

fn generate_packet(packet_type: u8, payload: &[u8]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(1 + 4 + payload.len());

    // Add the packet type
    packet.push(packet_type);

    // Add the length tag
    let length = payload.len() as u32;
    packet.extend_from_slice(&length.to_be_bytes());

    // Add the payload
    packet.extend_from_slice(payload);

    packet
}

async fn fetch_all_packet(tcp_stream: &mut TcpStream) -> ClientResult<(u8, Vec<u8>)> {
    let mut packet_buffer = [0; PAYLOAD_CHUNK_SIZE as usize];

    let _ = tcp_stream.read(&mut packet_buffer).await?;

    let (length, tag, mut all_bytes) = {
        let start_packet = parse_start_packet(&packet_buffer).ok_or(
            ClientError::ConnectionError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to parse start packet",
            )),
        )?;

        let length = start_packet.length;
        let tag = start_packet.tag;

        let mut all_bytes = start_packet.value.to_vec();
        all_bytes.reserve(start_packet.length as usize);

        (length, tag, all_bytes)
    };

    while length > all_bytes.len() as u32 {
        let bytes_read = tcp_stream.read(&mut packet_buffer).await?;

        if bytes_read == 0 {
            break;
        }

        all_bytes.extend_from_slice(&packet_buffer[..bytes_read]);
    }

    Ok((tag, all_bytes))
}
