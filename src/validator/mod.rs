pub mod pos;
pub mod pow;

use log::error;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::block::Block;
use crate::chain::BlockChain;
use crate::difficulter::{simple::SimpleDifficulter, Difficulter};
use crate::transaction::{Transaction, UnspentTxOut};

pub trait Validator {
    fn is_valid(
        &self,
        prev_block: &Block,
        next_block: &Block,
        chain: &BlockChain,
        unspent_tx_outs: &[UnspentTxOut],
    ) -> bool;
    fn find_block(&self, prev_block: &Block, data: Vec<Transaction>, difficulty: u32) -> Block;
    // fn has_valid_hash(hash: &str, difficulty: &u32, is_validate: bool) -> bool;

    fn has_valid_difficulty(&self, block: &Block, chain: &BlockChain) -> bool {
        let r = block.difficulty >= SimpleDifficulter::get_difficulty(chain);
        if !r {
            error!("block doesn't have valid difficulty")
        }

        r
    }

    fn is_valid_timestamp(&self, next_block: &Block, prev_block: &Block) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let r = prev_block.timestamp - 60 < next_block.timestamp || now + 60 < next_block.timestamp;
        if !r {
            error!("block doesn't have a valid timestamp");
        }

        r
    }
}
