use ring::rand::{SecureRandom, SystemRandom};

use crate::{block::Block, transaction::Transaction};

pub const MAGIC_NUMBER: u32 = 0xA71A5001;
pub const PACKET_BUFFER_SIZE: usize = 1024;
pub const MAX_RETRIES: u8 = 3;
pub const ACK_TIMEOUT: u64 = 500; // milliseconds

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PacketType {
    Probe,
    Sync,
    Transaction,
    Block,
    Data,
    Ack,
}

impl From<u8> for PacketType {
    fn from(value: u8) -> Self {
        match value {
            0 => PacketType::Probe,
            1 => PacketType::Sync,
            2 => PacketType::Transaction,
            3 => PacketType::Block,
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
pub struct ProbePayload {
    pub node_id: u16,
    pub public_key: Vec<u8>,
}

impl ProbePayload {
    pub fn new(node_id: u16, public_key: Vec<u8>) -> Self {
        Self {
            node_id,
            public_key,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            node_id: u16::from_le_bytes(bytes[0..2].try_into().unwrap()),
            public_key: bytes[2..].to_vec(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.node_id.to_le_bytes());
        bytes.extend_from_slice(&self.public_key);
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct SyncPayload {
    pub node_id: u16,
    pub public_key: Vec<u8>,
    pub chain_height: u32,
    pub last_block_hash: [u8; 32],
    pub last_block_timestamp: u64,
}

impl SyncPayload {
    pub fn new(
        node_id: u16,
        public_key: Vec<u8>,
        chain_height: u32,
        last_block_hash: [u8; 32],
        last_block_timestamp: u64,
    ) -> Self {
        Self {
            node_id,
            public_key,
            chain_height,
            last_block_hash,
            last_block_timestamp,
        }
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            node_id: u16::from_le_bytes(bytes[0..2].try_into().unwrap()),
            public_key: bytes[2..].to_vec(),
            chain_height: u32::from_le_bytes(bytes[2..6].try_into().unwrap()),
            last_block_hash: bytes[6..38].try_into().unwrap(),
            last_block_timestamp: u64::from_le_bytes(bytes[38..46].try_into().unwrap()),
        }
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.node_id.to_le_bytes());
        bytes.extend_from_slice(&self.public_key);
        bytes.extend_from_slice(&self.chain_height.to_le_bytes());
        bytes.extend_from_slice(&self.last_block_hash);
        bytes.extend_from_slice(&self.last_block_timestamp.to_le_bytes());
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

#[derive(Debug, Clone)]
pub struct TransactionPayload {
    pub transaction: Transaction,
}

impl TransactionPayload {
    pub fn new(transaction: Transaction) -> Self {
        Self { transaction }
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            transaction: Transaction::from_bytes(bytes),
        }
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        self.transaction.as_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct BlockPayload {
    pub block: Block,
}

impl BlockPayload {
    pub fn new(block: Block) -> Self {
        Self { block }
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            block: Block::new(vec![], bytes[32..64].try_into().unwrap()),
        }
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.block.prev_block_hash);
        for transaction in &self.block.transactions {
            bytes.extend_from_slice(&transaction.as_bytes());
        }
        bytes
    }
}
