use chorba::{Decode, Encode};

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

pub const NO_VALUE_TAGS: [u8; 9] = [
    PING,
    SET,
    CLEAR,
    PONG,
    SET_OK,
    DELETE_OK,
    CLEAR_OK,
    PACKET_INVALID,
    ERROR,
];

#[derive(Decode, Encode)]
pub struct SetRequest {
    pub key: String,
    pub value: String,
}

#[derive(Decode, Encode)]
pub struct GetRequest {
    pub key: String,
}

#[derive(Decode, Encode)]
pub struct GetResponse {
    pub value: String,
}

#[derive(Decode, Encode)]
pub struct DeleteRequest {
    pub key: String,
}

pub struct StartPacket<'a> {
    pub tag: u8,
    pub length: u32,
    pub value: &'a [u8],
}

pub(crate) fn parse_start_packet(packet: &[u8]) -> Option<StartPacket<'_>> {
    if packet.len() < 5 {
        return None;
    }

    let tag = packet[0];

    if NO_VALUE_TAGS.contains(&tag) {
        return Some(StartPacket {
            tag,
            length: 0,
            value: packet,
        });
    }

    let length = u32::from_be_bytes([packet[1], packet[2], packet[3], packet[4]]);
    let value = &packet[5..];

    Some(StartPacket { tag, length, value })
}
