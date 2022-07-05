use crate::block::Block;
use crate::difficulter::Difficulter;

pub struct BlockChain {
    pub blocks: Vec<Block>,
}

impl BlockChain {
    /// add a new block if valid
    pub fn add(&mut self, new: Block) {
        // TODO: return error
        if Block::is_valid_next_block(&new, &self.get_latest()) {
            self.blocks.push(new)
        }
    }

    /// get new_blocks and if valid completely change the self.blocks
    pub fn replace(&mut self, new_blocks: Vec<Block>) {
        let new_chain = BlockChain { blocks: new_blocks };

        if new_chain.is_valid()
            && Difficulter::get_accumulated_difficulty(&new_chain)
                > Difficulter::get_accumulated_difficulty(self)
        {
            self.blocks = new_chain.blocks;
        }
        // TODO: return error
    }

    /// return the latest block
    pub fn get_latest(&self) -> Block {
        self.blocks.last().unwrap().clone()
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
    fn is_valid(&self) -> bool {
        if *self.blocks.first().unwrap() != BlockChain::get_genesis() {
            return false;
        }

        for i in 1..self.blocks.len() {
            if !Block::is_valid_next_block(
                self.blocks.get(i).unwrap(),
                self.blocks.get(i - 1).unwrap(),
            ) {
                return false;
            }
        }

        true
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
