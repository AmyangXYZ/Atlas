use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cache::CacheOperation;
use crate::utils::serialize_hash;
use ring::digest::{digest, SHA256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub node_id: u16,
    pub client_id: u16,
    pub data_name: String,
    pub operation: CacheOperation,
    pub timestamp: u64,
    #[serde(serialize_with = "serialize_hash")]
    pub hash: [u8; 32],
}

impl Transaction {
    pub fn new(node_id: u16, client_id: u16, data_name: String, operation: CacheOperation) -> Self {
        assert!(data_name.len() <= 64, "Data name too long");
        let mut hash = [0; 32];
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        hash.copy_from_slice(
            digest(
                &SHA256,
                format!(
                    "{:?}.{:?}.{:?}.{:?}.{:?}",
                    node_id, client_id, data_name, operation, timestamp
                )
                .as_bytes(),
            )
            .as_ref(),
        );
        Self {
            node_id,
            client_id,
            data_name,
            operation,
            timestamp,
            hash,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(
            bytes.len() == 109,
            "Invalid transaction length, expected 109, got {:?}",
            bytes.len()
        );
        Self {
            node_id: u16::from_le_bytes(bytes[0..2].try_into().unwrap()),
            client_id: u16::from_le_bytes(bytes[2..4].try_into().unwrap()),
            data_name: String::from_utf8(
                bytes[4..68]
                    .to_vec()
                    .into_iter()
                    .take_while(|&b| b != 0)
                    .collect(),
            )
            .unwrap(),
            operation: CacheOperation::from(bytes[68]),
            timestamp: u64::from_le_bytes(bytes[69..77].try_into().unwrap()),
            hash: bytes[77..109].try_into().unwrap(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.node_id.to_le_bytes());
        bytes.extend_from_slice(&self.client_id.to_le_bytes());
        let mut name_bytes = [0u8; 64];
        name_bytes[..self.data_name.len()].copy_from_slice(self.data_name.as_bytes());
        bytes.extend_from_slice(&name_bytes);
        bytes.push(self.operation as u8);
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.hash);
        bytes
    }
}
