use ring::digest::{digest, SHA256};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::transaction::Transaction;

pub const BLOCK_PERIOD: u64 = 10; // seconds

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub merkle_root: [u8; 32],
    pub prev_block_hash: [u8; 32],
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(transactions: Vec<Transaction>, prev_block_hash: [u8; 32]) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let transaction_hashes: Vec<[u8; 32]> = transactions.iter().map(|txn| txn.hash).collect();

        let merkle_root =
            Self::calculate_merkle_root(&transaction_hashes, timestamp, &prev_block_hash);

        Self {
            merkle_root,
            prev_block_hash,
            timestamp,
            transactions,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let transactions = bytes[72..]
            .chunks(109)
            .map(|txn| Transaction::from_bytes(txn))
            .collect();

        Self {
            merkle_root: bytes[0..32].try_into().unwrap(),
            prev_block_hash: bytes[32..64].try_into().unwrap(),
            timestamp: u64::from_le_bytes(bytes[64..72].try_into().unwrap()),
            transactions,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.merkle_root);
        bytes.extend_from_slice(&self.prev_block_hash);
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        for transaction in &self.transactions {
            bytes.extend_from_slice(&transaction.as_bytes());
        }
        bytes
    }

    fn calculate_merkle_root(
        hashes: &[[u8; 32]],
        timestamp: u64,
        prev_block_hash: &[u8; 32],
    ) -> [u8; 32] {
        if hashes.is_empty() {
            // For empty blocks, hash the timestamp and previous block hash
            let mut data = Vec::with_capacity(40); // 32 bytes for prev_hash + 8 bytes for timestamp
            data.extend_from_slice(prev_block_hash);
            data.extend_from_slice(&timestamp.to_be_bytes());

            let mut hash = [0u8; 32];
            hash.copy_from_slice(digest(&SHA256, &data).as_ref());
            return hash;
        }
        if hashes.len() == 1 {
            return hashes[0];
        }

        let mut next_level = Vec::new();

        for chunk in hashes.chunks(2) {
            let mut combined = Vec::with_capacity(64);
            combined.extend_from_slice(&chunk[0]);
            combined.extend_from_slice(chunk.get(1).unwrap_or(&chunk[0]));

            let mut hash = [0u8; 32];
            hash.copy_from_slice(digest(&SHA256, &combined).as_ref());
            next_level.push(hash);
        }

        Self::calculate_merkle_root(&next_level, timestamp, prev_block_hash)
    }
}
