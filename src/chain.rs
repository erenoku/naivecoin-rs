use log::info;

use crate::block::{Block, UNSPENT_TX_OUTS};
use crate::difficulter::Difficulter;
use crate::transaction::{Transaction, UnspentTxOut};

pub struct BlockChain {
    pub blocks: Vec<Block>,
}

impl BlockChain {
    /// add a new block if valid
    pub fn add(&mut self, new: Block) {
        let mut unspent_tx_outs = UNSPENT_TX_OUTS.write().unwrap();

        // TODO: return error
        if Block::is_valid_next_block(&new, &self.get_latest().unwrap(), self) {
            if let Some(ret_val) =
                Transaction::process_transaction(&new.data, &unspent_tx_outs, &(new.index as u64))
            {
                self.blocks.push(new);
                *unspent_tx_outs = ret_val;
            }
        }
    }

    /// get new_blocks and if valid completely change the self.blocks
    pub fn replace(&mut self, new_blocks: Vec<Block>) {
        let new_chain = BlockChain { blocks: new_blocks };

        if let Some(new_unspent_tx_outs) = new_chain.is_valid() {
            if Difficulter::get_accumulated_difficulty(&new_chain)
                > Difficulter::get_accumulated_difficulty(self)
            {
                self.blocks = new_chain.blocks;
                *UNSPENT_TX_OUTS.write().unwrap() = new_unspent_tx_outs
            }
        }
        // TODO: return error
    }

    /// return the latest block
    pub fn get_latest(&self) -> Option<Block> {
        Some(self.blocks.last()?.clone())
    }

    /// return the genesis block
    pub fn get_genesis() -> Block {
        let mut g = Block {
            index: 0,
            previous_hash: String::from("0"),
            timestamp: 1465154705,
            data: vec![],
            hash: String::new(),
            difficulty: 0,
            nonce: 0,
        };

        let hash = g.calculate_hash();
        g.hash = hash;

        g
    }

    /// check if the complete chain is valid
    // TODO: return result
    fn is_valid(&self) -> Option<Vec<UnspentTxOut>> {
        if *self.blocks.first().unwrap() != BlockChain::get_genesis() {
            return None;
        }

        let mut new_unspent_tx_outs: Vec<UnspentTxOut> = vec![];

        for i in 1..self.blocks.len() {
            let current_blocks: Vec<Block> = self.blocks[0..i - 1].to_vec();

            if !Block::is_valid_next_block(
                self.blocks.get(i).unwrap(),
                self.blocks.get(i - 1).unwrap(),
                &BlockChain {
                    blocks: current_blocks,
                },
            ) {
                return None;
            }

            if let Some(x) = Transaction::process_transaction(
                &self.blocks.get(i).unwrap().data,
                &new_unspent_tx_outs,
                &(self.blocks.get(i).unwrap().index as u64),
            ) {
                new_unspent_tx_outs = x;
            } else {
                return None;
            }
        }
        Some(new_unspent_tx_outs)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_replace() {
//         let mut current_chain = BlockChain {
//             blocks: vec![
//                 BlockChain::get_genesis(),
//                 Block {
//                     index: 1,
//                     previous_hash: BlockChain::get_genesis().hash,
//                     timestamp: BlockChain::get_genesis().timestamp + 1,
//                     data: vec![],
//                     hash: String::from(
//                         "5cc9096cfe838a7ea0c5d986c5b6072eac518858938033295381a45daa72cb6e",
//                     ),
//                     difficulty: 0,
//                     nonce: 0,
//                 },
//             ],
//         };

//         let next_blocks = vec![
//             BlockChain::get_genesis(),
//             Block {
//                 index: 1,
//                 previous_hash: BlockChain::get_genesis().hash,
//                 timestamp: BlockChain::get_genesis().timestamp + 1,
//                 data: vec![],
//                 hash: String::from(
//                     "5cc9096cfe838a7ea0c5d986c5b6072eac518858938033295381a45daa72cb6e",
//                 ),
//                 difficulty: 0,
//                 nonce: 0,
//             },
//             Block {
//                 index: 2,
//                 previous_hash: String::from(
//                     "5cc9096cfe838a7ea0c5d986c5b6072eac518858938033295381a45daa72cb6e",
//                 ),
//                 timestamp: BlockChain::get_genesis().timestamp + 2,
//                 data: vec![],
//                 hash: String::from(
//                     "13543d261672ae2f0cb9f54ded0b5eca74f9d3bf85a80fae21d40e4362ffbb40",
//                 ),
//                 difficulty: 0,
//                 nonce: 0,
//             },
//         ];

//         current_chain.replace(next_blocks);

//         assert_eq!(current_chain.blocks.len(), 3)
//     }

//     #[test]
//     fn test_add() {
//         let mut chain = BlockChain {
//             blocks: vec![BlockChain::get_genesis()],
//         };

//         let next_block = Block {
//             index: 1,
//             previous_hash: BlockChain::get_genesis().hash,
//             timestamp: BlockChain::get_genesis().timestamp + 1,
//             data: vec![],
//             hash: String::from("5cc9096cfe838a7ea0c5d986c5b6072eac518858938033295381a45daa72cb6e"),
//             difficulty: 0,
//             nonce: 0,
//         };

//         chain.add(next_block.clone());

//         assert_eq!(chain.blocks.len(), 2);

//         chain.add(next_block.clone());

//         assert_eq!(chain.blocks.len(), 2);
//     }

//     #[test]
//     fn test_is_valid() {
//         let mut chain = BlockChain {
//             blocks: vec![
//                 BlockChain::get_genesis(),
//                 Block {
//                     index: 1,
//                     previous_hash: BlockChain::get_genesis().hash,
//                     timestamp: BlockChain::get_genesis().timestamp + 1,
//                     data: vec![],
//                     hash: String::from(
//                         "5cc9096cfe838a7ea0c5d986c5b6072eac518858938033295381a45daa72cb6e",
//                     ),
//                     difficulty: 0,
//                     nonce: 0,
//                 },
//             ],
//         };

//         assert!(chain.is_valid());

//         chain = BlockChain {
//             blocks: vec![
//                 BlockChain::get_genesis(),
//                 Block {
//                     index: 1,
//                     previous_hash: String::from(
//                         "e9e7efcbda3fb07db3aa416ceaefa830f3a5e19e77f5231d16de76c32abc39b2",
//                     ),
//                     timestamp: BlockChain::get_genesis().timestamp + 1,
//                     data: vec![],
//                     hash: String::from(
//                         "d7e7efcbda3fb07db3aa416ceaefa830f3a5e19e77f5231d16de76c32abc39b2",
//                     ),
//                     difficulty: 0,
//                     nonce: 0,
//                 },
//             ],
//         };

//         assert!(!chain.is_valid());

//         chain = BlockChain {
//             blocks: vec![
//                 BlockChain::get_genesis(),
//                 Block {
//                     index: 1,
//                     previous_hash: BlockChain::get_genesis().hash,
//                     timestamp: BlockChain::get_genesis().timestamp - 1,
//                     data: vec![],
//                     hash: String::from(
//                         "d7e7efcbda3fb07db3aa416ceaefa830f3a5e19e77f5231d16de76c32abc39b2",
//                     ),
//                     difficulty: 0,
//                     nonce: 0,
//                 },
//             ],
//         };

//         assert!(!chain.is_valid());

//         chain = BlockChain {
//             blocks: vec![
//                 BlockChain::get_genesis(),
//                 Block {
//                     index: 1,
//                     previous_hash: BlockChain::get_genesis().hash,
//                     timestamp: BlockChain::get_genesis().timestamp,
//                     data: vec![],
//                     hash: String::from(
//                         "d7e7efcbda3fb07db3aa416ceaefa830f3a5e19e77f5231d16de76c32abc39b2",
//                     ),
//                     difficulty: 0,
//                     nonce: 0,
//                 },
//             ],
//         };

//         assert!(!chain.is_valid());

//         chain = BlockChain {
//             blocks: vec![
//                 BlockChain::get_genesis(),
//                 Block {
//                     index: 1,
//                     previous_hash: BlockChain::get_genesis().hash,
//                     timestamp: BlockChain::get_genesis().timestamp + 1,
//                     data: vec![],
//                     hash: String::from(
//                         "d7e7efcbda3fb07db3aa416ceaefa831f3a5e19e77f5231d16de76c32abc39b2",
//                     ),
//                     difficulty: 0,
//                     nonce: 0,
//                 },
//             ],
//         };

//         assert!(!chain.is_valid());
//     }
// }
