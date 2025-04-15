use chorba::Decode;

// Redis - 512MB (Key, Value)
// Memcached - 1MB (Key, Value)
pub const KEY_BYTE_LIMIT: u32 = 1024 * 1024; // 1MB
pub const VALUE_BYTE_LIMIT: u32 = 1024 * 1024 * 10; // 10MB
pub const PACKET_BYTE_LIMIT: u32 = 1024 * 1024 * 20; // 20MB

pub const PAYLOAD_CHUNK_SIZE: u32 = 1024; // 1KB
pub const PAYLOAD_HEAD_SIZE: u32 = 5; // Tag 1 Byte + Length 4 Bytes
pub const PAYLOAD_FIRST_MAX_VALUE_SIZE: u32 = PAYLOAD_CHUNK_SIZE - PAYLOAD_HEAD_SIZE; // 1KB - 5 Bytes

// Request Tag - Start Byte
pub const PING: u8 = 0x01;
pub const SET: u8 = 0x02;
pub const GET: u8 = 0x03;
pub const DELETE: u8 = 0x04;
pub const CLEAR: u8 = 0x05;

// Response Tag - Start Byte
pub const PONG: u8 = 0x01;
pub const SET_OK: u8 = 0x02;
pub const GET_OK: u8 = 0x03;
pub const DELETE_OK: u8 = 0x04;
pub const CLEAR_OK: u8 = 0x05;
pub const PACKET_INVALID: u8 = 0xFE;
pub const ERROR: u8 = 0xFF;

#[derive(Decode)]
pub struct SetRequest {
    pub key: String,
    pub value: String,
}

#[derive(Decode)]
pub struct GetRequest {
    pub key: String,
}
