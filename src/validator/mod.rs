pub mod pow;

use crate::block::Block;
use crate::chain::BlockChain;

pub trait Validator {
    fn is_valid(prev_block: &Block, next_block: &Block, chain: &BlockChain) -> bool;
    fn hash_matches_difficulty(hash: &str, difficulty: &u32, is_validate: bool) -> bool;
}
