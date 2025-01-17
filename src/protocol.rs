use ring::rand::{SecureRandom, SystemRandom};

pub const MAGIC_NUMBER: u32 = 0xA71A5;
pub const PACKET_BUFFER_SIZE: usize = 1024;
pub const MAX_RETRIES: u8 = 3;
pub const ACK_TIMEOUT: u64 = 500; // milliseconds

pub const ORCHESTRATOR_ID: u16 = 0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PacketType {
    JoinRequest,
    JoinResponse,
    KeyRequest,
    KeyResponse,
    Data,
    Ack,
}

impl From<u8> for PacketType {
    fn from(value: u8) -> Self {
        match value {
            0 => PacketType::JoinRequest,
            1 => PacketType::JoinResponse,
            2 => PacketType::KeyRequest,
            3 => PacketType::KeyResponse,
            4 => PacketType::Data,
            5 => PacketType::Ack,
            _ => panic!("Invalid packet type"),
        }
    }
}

pub struct Packet {
    pub magic_number: u32,
    pub packet_id: u32,
    pub src: u16,
    pub dst: u16,
    pub packet_type: u8,
    pub timestamp: u64,
    pub payload: Vec<u8>,
}

impl Packet {
    pub fn new(src: u16, dst: u16, packet_type: PacketType, payload: Vec<u8>) -> Self {
        let mut random_bytes = [0u8; 8];
        SystemRandom::new().fill(&mut random_bytes).unwrap();
        Self {
            magic_number: MAGIC_NUMBER,
            packet_id: u32::from_le_bytes(random_bytes[0..4].try_into().unwrap()),
            src,
            dst,
            packet_type: packet_type as u8,
            timestamp: 0,
            payload,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 21 {
            return None;
        }
        let magic_number = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        if magic_number != MAGIC_NUMBER {
            return None;
        }
        let packet_id = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let src = u16::from_le_bytes(bytes[8..10].try_into().unwrap());
        let dst = u16::from_le_bytes(bytes[10..12].try_into().unwrap());
        let packet_type = bytes[12];
        let timestamp = u64::from_le_bytes(bytes[13..21].try_into().unwrap());
        let payload = bytes[21..].to_vec();
        Some(Self {
            magic_number: MAGIC_NUMBER,
            packet_id,
            src,
            dst,
            packet_type,
            timestamp,
            payload,
        })
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.magic_number.to_le_bytes());
        bytes.extend_from_slice(&self.packet_id.to_le_bytes());
        bytes.extend_from_slice(&self.src.to_le_bytes());
        bytes.extend_from_slice(&self.dst.to_le_bytes());
        bytes.extend_from_slice(&self.packet_type.to_le_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    pub fn clone(&self) -> Self {
        Self {
            magic_number: self.magic_number,
            packet_id: self.packet_id,
            src: self.src,
            dst: self.dst,
            packet_type: self.packet_type,
            timestamp: self.timestamp,
            payload: self.payload.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AckPayload {
    pub packet_id: u32,
}

impl AckPayload {
    pub fn new(packet_id: u32) -> Self {
        Self { packet_id }
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            packet_id: u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
        }
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        self.packet_id.to_le_bytes().to_vec()
    }
}

#[derive(Debug, Clone)]
pub struct JoinRequestPayload {
    pub node_id: u16,
    pub key: Vec<u8>,
}

impl JoinRequestPayload {
    pub fn new(node_id: u16, key: Vec<u8>) -> Self {
        Self { node_id, key }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            node_id: u16::from_le_bytes(bytes[0..2].try_into().unwrap()),
            key: bytes[2..].to_vec(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.node_id.to_le_bytes());
        bytes.extend_from_slice(&self.key);
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct JoinResponsePayload {
    pub permission: bool,
}

impl JoinResponsePayload {
    pub fn new(permission: bool) -> Self {
        Self { permission }
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            permission: bytes[0] == 1,
        }
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        vec![self.permission as u8]
    }
}

#[derive(Debug, Clone)]
pub struct KeyPayload {
    pub node_id: u16,
    pub key: Vec<u8>,
}

impl KeyPayload {
    pub fn new(node_id: u16, key: Vec<u8>) -> Self {
        Self { node_id, key }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            node_id: u16::from_le_bytes(bytes[0..2].try_into().unwrap()),
            key: bytes[2..].to_vec(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.node_id.to_le_bytes());
        bytes.extend_from_slice(&self.key);
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct DataPayload {
    pub signature: Vec<u8>,
    pub data: Vec<u8>,
}

impl DataPayload {
    pub fn new(signature: Vec<u8>, data: Vec<u8>) -> Self {
        Self { signature, data }
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            signature: bytes[0..64].to_vec(),
            data: bytes[64..].to_vec(),
        }
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.signature);
        bytes.extend_from_slice(&self.data);
        bytes
    }
}
