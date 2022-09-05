use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::RwLockWriteGuard;

use crate::{
    chain::BlockChain,
    crypto::KeyPair,
    difficulter::{simple::SimpleDifficulter, Difficulter},
    transaction::{Transaction, UnspentTxOut},
    transaction_pool::TransactionPool,
    validator::Validator,
    // validator::{pow::PowValidator, Validator},
    wallet::Wallet,
};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Block {
    pub index: u32,
    pub previous_hash: String,
    pub timestamp: u64,
    pub data: Vec<Transaction>,
    pub hash: String,
    pub nonce: u32,
    pub difficulty: u32,
    // pub creator_address: String,
    // pub creator_balance
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
    pub fn is_valid_next_block(
        next: &Block,
        prev: &Block,
        chain: &BlockChain,
        validator: &impl Validator,
        unspent_tx_outs: &[UnspentTxOut],
    ) -> bool {
        if validator.is_valid(prev, next, chain, unspent_tx_outs) {
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

    pub fn generate_next(
        chain: &BlockChain,
        wallet: &Wallet,
        tx_pool: &TransactionPool,
        validator: &impl Validator,
    ) -> Self {
        let public_key = wallet.get_public_key();

        let coinbase_tx = Transaction::get_coinbase_tx(
            KeyPair::public_key_to_hex(&public_key),
            (chain.get_latest().unwrap().index + 1) as u64,
        );
        Self::generate_next_raw(
            vec![vec![coinbase_tx], tx_pool.0.clone()].concat(),
            chain,
            validator,
        )
    }

    /// generate the next block with given block_data
    pub fn generate_next_raw(
        block_data: Vec<Transaction>,
        chain: &BlockChain,
        validator: &impl Validator,
    ) -> Self {
        let prev_block = chain.get_latest().unwrap();
        let difficulty = SimpleDifficulter::get_difficulty(chain);

        validator.find_block(&prev_block, block_data, difficulty)
    }

    pub fn generate_next_with_transaction(
        receiver_addr: String,
        amount: u64,
        chain: &BlockChain,
        wallet: &Wallet,
        pool: &TransactionPool,
        unspent_tx_outs: RwLockWriteGuard<Vec<UnspentTxOut>>, // TODO: remove state from validator
        validator: &impl Validator,
    ) -> Option<Self> {
        let public_key = wallet.get_public_key();
        let private_key = wallet.get_private_key();

        let coinbase_tx = Transaction::get_coinbase_tx(
            KeyPair::public_key_to_hex(&public_key),
            (chain.get_latest().unwrap().index + 1) as u64,
        );
        if let Some(tx) =
            Wallet::create_transaction(receiver_addr, amount, &private_key, &unspent_tx_outs, pool)
        {
            drop(unspent_tx_outs);
            return Some(Self::generate_next_raw(
                vec![coinbase_tx, tx],
                chain,
                validator,
            ));
        }
        None
    }
}
