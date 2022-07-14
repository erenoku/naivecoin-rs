use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    sync::RwLock,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    chain::BlockChain,
    crypto::KeyPair,
    difficulter::Difficulter,
    transaction::{Transaction, UnspentTxOut},
    validator::Validator,
    wallet::Wallet,
    BLOCK_CHAIN,
};

pub static UNSPENT_TX_OUTS: Lazy<RwLock<Vec<UnspentTxOut>>> = Lazy::new(|| RwLock::new(vec![]));

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
    pub fn is_valid_next_block(next: &Block, prev: &Block, chain: &BlockChain) -> bool {
        if Validator::is_valid(prev, next, chain) {
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

    pub fn generate_next() -> Self {
        let chain = BLOCK_CHAIN.read().unwrap();
        let public_key = &Wallet::global().read().unwrap().get_public_key();

        let coinbase_tx = Transaction::get_coinbase_tx(
            KeyPair::public_key_to_hex(public_key),
            (chain.get_latest().unwrap().index + 1) as u64,
        );
        Self::generate_next_raw(vec![coinbase_tx], &chain)
    }

    /// generate the next block with given block_data
    pub fn generate_next_raw(block_data: Vec<Transaction>, chain: &BlockChain) -> Self {
        let prev_block = chain.get_latest().unwrap();
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

    pub fn generate_next_with_transaction(receiver_addr: String, amount: u64) -> Option<Self> {
        let chain = BLOCK_CHAIN.read().unwrap();
        let public_key = &Wallet::global().read().unwrap().get_public_key();
        let private_key = &Wallet::global().read().unwrap().get_private_key();

        let coinbase_tx = Transaction::get_coinbase_tx(
            KeyPair::public_key_to_hex(public_key),
            (chain.get_latest().unwrap().index + 1) as u64,
        );
        if let Some(tx) = Wallet::create_transaction(
            receiver_addr,
            amount,
            private_key,
            &UNSPENT_TX_OUTS.read().unwrap(),
        ) {
            return Some(Self::generate_next_raw(vec![coinbase_tx, tx], &chain));
        }
        None
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

            if Validator::hash_matches_difficulty(&hash, &difficulty, false) {
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
