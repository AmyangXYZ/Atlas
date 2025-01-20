use std::time::{SystemTime, UNIX_EPOCH};

use crate::transaction::Transaction;

#[derive(Debug, Clone)]
pub struct Block {
    pub transactions: Vec<Transaction>,
    pub merkle_root: [u8; 32],
    pub prev_block_hash: [u8; 32],
    pub timestamp: u64,
}

impl Block {
    pub fn new(transactions: Vec<Transaction>, prev_block_hash: [u8; 32]) -> Self {
        Self {
            transactions,
            merkle_root: [0; 32],
            prev_block_hash,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}
