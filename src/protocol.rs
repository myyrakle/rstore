use chorba::{Decode, Encode};
use tokio::{io::AsyncReadExt, net::TcpStream};

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

#[derive(Debug, thiserror::Error)]
pub enum PacketError {
    #[error("Tag is empty")]
    EmptyTag,
    #[error("Read failed")]
    ReadFailed(#[from] std::io::Error),
    #[error("No Data received")]
    NoDataReceived,
}

#[allow(dead_code)]
pub(crate) fn parse_start_packet(packet: &[u8]) -> Result<StartPacket<'_>, PacketError> {
    if packet.is_empty() {
        return Err(PacketError::EmptyTag);
    }

    let tag = packet[0];

    if packet.len() < 5 {
        return Ok(StartPacket {
            tag,
            length: 0,
            value: packet,
        });
    }

    let tag = packet[0];

    if NO_VALUE_TAGS.contains(&tag) {
        return Ok(StartPacket {
            tag,
            length: 0,
            value: packet,
        });
    }

    let length = u32::from_be_bytes([packet[1], packet[2], packet[3], packet[4]]);
    let value = &packet[5..];

    Ok(StartPacket { tag, length, value })
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

#[allow(dead_code)]
pub(crate) async fn read_all_from_stream(
    tcp_stream: &mut TcpStream,
) -> Result<(u8, Vec<u8>), PacketError> {
    let mut packet_buffer = [0; PAYLOAD_CHUNK_SIZE as usize];

    let first_read_count = tcp_stream.read(&mut packet_buffer).await?;

    if first_read_count == 0 {
        return Err(PacketError::NoDataReceived);
    }

    let (length, tag, mut all_bytes) = {
        let start_packet = parse_start_packet(&packet_buffer[..first_read_count])?;

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
