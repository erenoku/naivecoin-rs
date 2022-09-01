pub mod block;
pub mod chain;
pub mod crypto;
pub mod difficulter;
pub mod message;
pub mod p2p;
pub mod p2p_handler;
pub mod transaction;
pub mod transaction_pool;
pub mod validator;
pub mod wallet;

use once_cell::sync::{Lazy, OnceCell};
use std::sync::RwLock;

use crate::block::Block;
use crate::wallet::Wallet;
use chain::BlockChain;
use transaction_pool::TransactionPool;

// TODO: decouple library code from main executable code to put these global variables
// into the file main.rs

pub static BLOCK_CHAIN: Lazy<RwLock<BlockChain>> = Lazy::new(|| {
    RwLock::new(BlockChain {
        blocks: vec![BlockChain::get_genesis()],
    })
});

pub static TRANSACTIN_POOL: Lazy<RwLock<TransactionPool>> =
    Lazy::new(|| RwLock::new(TransactionPool::new()));

pub static WALLET: OnceCell<RwLock<Wallet>> = OnceCell::new();

// in seconds
pub const BLOCK_GENERATION_INTERVAL: u32 = 10;
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u32 = 10;

pub const COINBASE_AMOUNT: u64 = 50;
