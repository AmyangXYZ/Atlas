use crate::transaction::Transaction;

pub struct Block {
    pub transactions: Vec<Transaction>,
    pub merkle_root: [u8; 32],
    pub prev_block_hash: [u8; 32],
    pub timestamp: u64,
}
