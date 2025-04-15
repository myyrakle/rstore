use std::sync::Arc;

use tokio::net::TcpStream;

#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: u16,
}

impl ConnectionConfig {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }
}

#[derive(Debug, Clone)]
pub struct RStoreClient {
    connection_pool: Arc<ConnectionPool>,
    connection_config: ConnectionConfig,
}

impl RStoreClient {
    pub fn new(connection_config: ConnectionConfig) -> Self {
        let connection_pool = Arc::new(ConnectionPool {
            connections: Vec::new(),
        });

        RStoreClient {
            connection_pool,
            connection_config,
        }
    }
}

#[derive(Debug)]
pub struct ConnectionPool {
    connections: Vec<Connection>,
}

#[derive(Debug)]
pub struct Connection {
    using: bool,
    tcp_stream: TcpStream,
}
