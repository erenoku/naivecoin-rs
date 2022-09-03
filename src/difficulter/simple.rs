use log::info;

use crate::block::Block;
use crate::chain::BlockChain;
use crate::{BLOCK_GENERATION_INTERVAL, DIFFICULTY_ADJUSTMENT_INTERVAL};

use super::Difficulter;

pub struct SimpleDifficulter;

pub const START_DIFFICULTY: u32 = 1;

impl Difficulter for SimpleDifficulter {
    fn get_accumulated_difficulty(chain: &BlockChain) -> u64 {
        chain
            .blocks
            .iter()
            .map(|block| 2u64.pow(block.difficulty))
            .sum()
    }

    fn get_adjusted_difficulty(chain: &BlockChain, latest_block: &Block) -> u32 {
        let prev_adjustment_block: &Block =
            &chain.blocks[chain.blocks.len() - DIFFICULTY_ADJUSTMENT_INTERVAL as usize];
        let time_expected = (BLOCK_GENERATION_INTERVAL * DIFFICULTY_ADJUSTMENT_INTERVAL) as u64;
        let time_taken = latest_block.timestamp - prev_adjustment_block.timestamp;
        info!(
            "time taken: {} time expected: {}",
            time_taken, time_expected
        );

        if time_taken < time_expected / 2 {
            prev_adjustment_block.difficulty + 1
        } else if time_taken > time_expected * 2 {
            if prev_adjustment_block.difficulty > 0 {
                prev_adjustment_block.difficulty - 1
            } else {
                prev_adjustment_block.difficulty
            }
        } else {
            prev_adjustment_block.difficulty
        }
    }

    fn get_difficulty(chain: &BlockChain) -> u32 {
        if let Some(latest) = chain.get_latest() {
            if latest.index % DIFFICULTY_ADJUSTMENT_INTERVAL == 0 && latest.index != 0 {
                return SimpleDifficulter::get_adjusted_difficulty(chain, &latest);
            }
            latest.difficulty
        } else {
            START_DIFFICULTY
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_difficulty() {
        let mut block = Block {
            index: 10,
            previous_hash: String::new(),
            timestamp: BlockChain::get_genesis().timestamp + 1,
            data: vec![],
            hash: String::new(),
            nonce: 0,
            difficulty: 0,
        };
        // the blocks are not validated so we can easily make up blocks
        let mut chain = BlockChain {
            blocks: vec![
                BlockChain::get_genesis(),
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
            ],
        };

        assert_eq!(SimpleDifficulter::get_difficulty(&chain), 1);

        block = Block {
            index: 10,
            previous_hash: String::new(),
            timestamp: BlockChain::get_genesis().timestamp + 500,
            data: vec![],
            hash: String::new(),
            nonce: 0,
            difficulty: 0,
        };
        // the blocks are not validated so we can easily make up blocks
        chain = BlockChain {
            blocks: vec![
                BlockChain::get_genesis(),
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
            ],
        };

        assert_eq!(SimpleDifficulter::get_difficulty(&chain), 0);

        block = Block {
            index: 10,
            previous_hash: String::new(),
            timestamp: BlockChain::get_genesis().timestamp + 500,
            data: vec![],
            hash: String::new(),
            nonce: 0,
            difficulty: 5,
        };
        // the blocks are not validated so we can easily make up blocks
        chain = BlockChain {
            blocks: vec![
                Block {
                    index: 10,
                    previous_hash: String::new(),
                    timestamp: BlockChain::get_genesis().timestamp,
                    data: vec![],
                    hash: String::new(),
                    nonce: 0,
                    difficulty: 5,
                },
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
                block.clone(),
                block,
            ],
        };

        assert_eq!(SimpleDifficulter::get_difficulty(&chain), 4);
    }
}
