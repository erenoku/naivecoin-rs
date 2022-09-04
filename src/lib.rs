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

// in seconds
pub const BLOCK_GENERATION_INTERVAL: u32 = 10;
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u32 = 10;

pub const COINBASE_AMOUNT: u64 = 50;
