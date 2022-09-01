// TODO: find a better filename
pub mod simple;

use crate::block::Block;
use crate::chain::BlockChain;

pub trait Difficulter {
    fn get_accumulated_difficulty(chain: &BlockChain) -> u64;
    fn get_adjusted_difficulty(chain: &BlockChain, latest_block: &Block) -> u32;
    fn get_difficulty(chain: &BlockChain) -> u32;
}
