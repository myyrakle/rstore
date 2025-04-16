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
pub const PONG: u8 = 0xf1;
pub const SET_OK: u8 = 0xf2;
pub const GET_OK: u8 = 0xf3;
pub const DELETE_OK: u8 = 0xf4;
pub const CLEAR_OK: u8 = 0xf5;
pub const PACKET_INVALID: u8 = 0xfe;
pub const ERROR: u8 = 0xff;

pub const NO_VALUE_TAGS: [u8; 8] = [
    PING,
    CLEAR,
    PONG,
    SET_OK,
    DELETE_OK,
    CLEAR_OK,
    PACKET_INVALID,
    ERROR,
];

#[derive(Decode, Encode, Debug, Clone)]
pub struct SetRequest {
    pub key: String,
    pub value: String,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct GetRequest {
    pub key: String,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct GetResponse {
    pub value: String,
}

#[derive(Decode, Encode, Debug, Clone)]
pub struct DeleteRequest {
    pub key: String,
}

#[derive(Debug, Clone)]
pub struct StartPacket<'a> {
    pub tag: u8,
    pub length: u32,
    pub value: &'a [u8],
}

#[allow(dead_code)]
pub(crate) fn parse_start_packet(packet: &[u8]) -> Option<StartPacket<'_>> {
    if packet.is_empty() {
        return None;
    }

    let tag = packet[0];

    if packet.len() < 5 {
        return Some(StartPacket {
            tag,
            length: 0,
            value: packet,
        });
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

#[allow(dead_code)]
pub(crate) fn generate_packet(packet_type: u8, payload: &[u8]) -> Vec<u8> {
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
