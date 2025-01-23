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
    SetData,
    GetData,
    Data,
    GetChain,
    Chain,
    Block,
    Transaction,
    Ack,
}

impl From<u8> for PacketType {
    fn from(value: u8) -> Self {
        match value {
            0 => PacketType::Probe,
            1 => PacketType::Sync,
            2 => PacketType::SetData,
            3 => PacketType::GetData,
            4 => PacketType::Data,
            5 => PacketType::GetChain,
            6 => PacketType::Chain,
            7 => PacketType::Block,
            8 => PacketType::Transaction,
            9 => PacketType::Ack,
            _ => panic!("Invalid packet type"),
        }
    }
}

#[derive(Debug, Clone)]
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
    pub public_key: [u8; 32],
}

impl ProbePayload {
    pub fn new(node_id: u16, public_key: [u8; 32]) -> Self {
        Self {
            node_id,
            public_key,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            node_id: u16::from_le_bytes(bytes[0..2].try_into().unwrap()),
            public_key: bytes[2..34].try_into().unwrap(),
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
    pub public_key: [u8; 32],
    pub chain_height: u32,
    pub last_block_hash: [u8; 32],
    pub last_block_timestamp: u64,
}

impl SyncPayload {
    pub fn new(
        node_id: u16,
        public_key: [u8; 32],
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
            public_key: bytes[2..34].try_into().unwrap(),
            chain_height: u32::from_le_bytes(bytes[34..38].try_into().unwrap()),
            last_block_hash: bytes[38..70].try_into().unwrap(),
            last_block_timestamp: u64::from_le_bytes(bytes[70..78].try_into().unwrap()),
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
    pub name: String, // max 64 bytes
    pub data: Vec<u8>,
}

impl DataPayload {
    pub fn new(name: String, data: Vec<u8>) -> Self {
        assert!(name.len() <= 64, "Name must not exceed 64 bytes");
        Self { name, data }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let name = String::from_utf8(
            bytes[0..64]
                .to_vec()
                .into_iter()
                .take_while(|&b| b != 0)
                .collect(),
        )
        .unwrap();

        let data = bytes[64..].to_vec();

        Self { name, data }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(128 + self.data.len());

        let mut name_bytes = [0u8; 64];
        name_bytes[..self.name.len()].copy_from_slice(self.name.as_bytes());
        bytes.extend_from_slice(&name_bytes);
        bytes.extend_from_slice(&self.data);
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct TransactionPayload {
    pub transaction: Transaction,
    pub signature: [u8; 64],
}

impl TransactionPayload {
    pub fn new(transaction: Transaction, signature: [u8; 64]) -> Self {
        Self {
            transaction,
            signature,
        }
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(
            bytes.len() == 173,
            "Invalid transaction payload length, expected 173, got {:?}",
            bytes.len()
        );
        Self {
            transaction: Transaction::from_bytes(&bytes[0..109]),
            signature: bytes[109..173].try_into().unwrap(),
        }
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.transaction.as_bytes());
        bytes.extend_from_slice(&self.signature);
        bytes
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
            block: Block::from_bytes(&bytes),
        }
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.block.as_bytes());
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct ChainPayload {
    pub chain: Vec<Block>,
}

impl ChainPayload {
    pub fn new(chain: Vec<Block>) -> Self {
        Self { chain }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut blocks = Vec::new();
        let mut offset = 0;

        while offset + 2 <= bytes.len() {
            let size = u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap()) as usize;
            if offset + 2 + size > bytes.len() {
                break;
            }
            let block = Block::from_bytes(&bytes[offset + 2..offset + 2 + size]);
            blocks.push(block);

            offset += 2 + size;
        }

        Self { chain: blocks }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for block in &self.chain {
            let block_bytes = block.as_bytes();
            bytes.extend_from_slice(&(block_bytes.len() as u16).to_le_bytes());
            bytes.extend_from_slice(&block_bytes);
        }
        bytes
    }
}
