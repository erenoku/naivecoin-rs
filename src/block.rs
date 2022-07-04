use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    chain::BlockChain, difficulter::Difficulter, transaction::Transaction, validator::Validator,
    BLOCK_CHAIN,
};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Block {
    pub index: u32,
    pub previous_hash: String,
    pub timestamp: u64,
    pub data: Vec<Transaction>,
    pub hash: String,
    pub nonce: u32,
    pub difficulty: u32,
}

impl Block {
    /// calculate hash of the whole block
    pub fn calculate_hash(&self) -> String {
        Block::calculate_hash_from_data(
            &self.index,
            &self.previous_hash,
            &self.timestamp,
            &self.data,
            &self.difficulty,
            &self.nonce,
        )
    }

    /// check if the next block is valid for the given previous block
    pub fn is_valid_next_block(next: &Block, prev: &Block) -> bool {
        if Validator::is_valid(prev, next) {
            return true;
        }

        false
    }

    fn calculate_hash_from_data(
        index: &u32,
        previous_hash: &str,
        timestamp: &u64,
        data: &Vec<Transaction>,
        difficulty: &u32,
        nonce: &u32,
    ) -> String {
        let mut hasher = Sha256::new();

        hasher.update(index.to_be_bytes());
        hasher.update(previous_hash);
        hasher.update(timestamp.to_be_bytes());
        for t in data.iter() {
            hasher.update(t.id.as_str())
        }
        hasher.update(difficulty.to_be_bytes());
        hasher.update(nonce.to_be_bytes());

        format!("{:x}", hasher.finalize())
    }

    /// generate the next block with given block_data
    pub fn generate_next(block_data: Vec<Transaction>, chain: &BlockChain) -> Block {
        let prev_block = chain.get_latest();
        let next_index = prev_block.index + 1;
        let next_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let difficulty = Difficulter::get_difficulty(&BLOCK_CHAIN.read().unwrap());

        Block::find_block(
            next_index,
            prev_block.hash,
            next_timestamp,
            block_data,
            difficulty,
        )
    }

    pub fn find_block(
        index: u32,
        previous_hash: String,
        timestamp: u64,
        data: Vec<Transaction>,
        difficulty: u32,
    ) -> Self {
        let mut nonce = 0;
        loop {
            let hash = Block::calculate_hash_from_data(
                &index,
                &previous_hash,
                &timestamp,
                &data,
                &difficulty,
                &nonce,
            );

            if Validator::hash_matches_difficulty(&hash, &difficulty) {
                return Self {
                    index,
                    previous_hash,
                    timestamp,
                    data,
                    hash,
                    difficulty,
                    nonce,
                };
            }
            nonce += 1;
        }
    }
}
