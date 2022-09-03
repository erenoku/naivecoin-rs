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
    difficulter::{simple::SimpleDifficulter, Difficulter},
    transaction::{Transaction, UnspentTxOut},
    // validator::{pos::PosValidator as PowValidator, Validator},
    validator::{pow::PowValidator, Validator},
    wallet::Wallet,
    BLOCK_CHAIN,
    TRANSACTIN_POOL,
};

pub static UNSPENT_TX_OUTS: Lazy<RwLock<Vec<UnspentTxOut>>> = Lazy::new(|| RwLock::new(vec![]));

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
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
        if PowValidator::is_valid(prev, next, chain) {
            return true;
        }

        false
    }

    pub fn calculate_hash_from_data(
        index: &u32,
        previous_hash: &str,
        timestamp: &u64,
        data: &[Transaction],
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
        let tx = &*TRANSACTIN_POOL.read().unwrap();
        Self::generate_next_raw(vec![vec![coinbase_tx], tx.0.clone()].concat(), &chain)
    }

    /// generate the next block with given block_data
    pub fn generate_next_raw(block_data: Vec<Transaction>, chain: &BlockChain) -> Self {
        let prev_block = chain.get_latest().unwrap();
        let next_index = prev_block.index + 1;
        let next_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let difficulty = SimpleDifficulter::get_difficulty(&BLOCK_CHAIN.read().unwrap());

        PowValidator::find_block(&prev_block, block_data, difficulty)
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
            UNSPENT_TX_OUTS.read().unwrap().to_vec(),
        ) {
            return Some(Self::generate_next_raw(vec![coinbase_tx, tx], &chain));
        }
        None
    }
}
