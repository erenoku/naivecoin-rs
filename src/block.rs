use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{chain::BlockChain, BLOCK_CHAIN};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Block {
    pub index: u32,
    pub previous_hash: String,
    pub timestamp: u64,
    pub data: String,
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
        if prev.index + 1 != next.index
            || prev.hash != next.previous_hash
            || next.calculate_hash() != next.hash
            || !Block::hash_matches_difficulty(&next.hash, &next.difficulty)
            || !Block::is_valid_timestamp(next, prev)
        {
            return false;
        }

        true
    }

    fn is_valid_timestamp(next: &Block, prev: &Block) -> bool {
        // TODO: timezone??
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        prev.timestamp - 60 < next.timestamp || now + 60 < next.timestamp
    }

    fn calculate_hash_from_data(
        index: &u32,
        previous_hash: &str,
        timestamp: &u64,
        data: &str,
        difficulty: &u32,
        nonce: &u32,
    ) -> String {
        let mut hasher = Sha256::new();

        hasher.update(index.to_be_bytes());
        hasher.update(previous_hash);
        hasher.update(timestamp.to_be_bytes());
        hasher.update(data);
        hasher.update(difficulty.to_be_bytes());
        hasher.update(nonce.to_be_bytes());

        format!("{:x}", hasher.finalize())
    }

    /// generate the next block with given block_data
    pub fn generate_next(block_data: String, chain: &BlockChain) -> Block {
        let prev_block = chain.get_latest();
        let next_index = prev_block.index + 1;
        let next_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let difficulty = BLOCK_CHAIN.read().unwrap().get_difficulty();

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
        data: String,
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

            if Block::hash_matches_difficulty(&hash, &difficulty) {
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

    fn hash_matches_difficulty(hash: &str, difficulty: &u32) -> bool {
        let bin = Block::hex_to_bin(hash);

        bin.starts_with("0".repeat(*difficulty as usize).as_str())
    }

    fn hex_to_bin(hex: &str) -> String {
        let mut bin = String::new();

        for char in hex.chars() {
            let a = match char {
                '0' => "0000",
                '1' => "0001",
                '2' => "0010",
                '3' => "0011",
                '4' => "0100",
                '5' => "0101",
                '6' => "0110",
                '7' => "0111",
                '8' => "1000",
                '9' => "1001",
                'a' => "1010",
                'b' => "1011",
                'c' => "1100",
                'd' => "1101",
                'e' => "1110",
                'f' => "1111",
                'A' => "1010",
                'B' => "1011",
                'C' => "1100",
                'D' => "1101",
                'E' => "1110",
                'F' => "1111",
                e => panic!("sha256 hash contains invalid character: {}", e),
            };
            bin.push_str(a);
        }

        bin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_matches_difficulty() {
        assert!(Block::hash_matches_difficulty(
            &String::from("0fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            &4
        ));

        assert!(Block::hash_matches_difficulty(
            &String::from("0fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            &0
        ));

        assert!(!Block::hash_matches_difficulty(
            &String::from("0fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            &5
        ));
    }
}
