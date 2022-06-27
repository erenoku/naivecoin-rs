use crate::block::Block;

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

        if new_chain.is_valid() && new_chain.blocks.len() > self.blocks.len() {
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
        Block {
            index: 0,
            previous_hash: String::from("0"),
            timestamp: 1465154705,
            data: String::from("my genesis block!!"),
            hash: String::from("816534932c2b7154836da6afc367695e6337db8a921823784c14378abed4f7d7"),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut chain = BlockChain {
            blocks: vec![BlockChain::get_genesis()],
        };

        let next_block = Block {
            index: 1,
            previous_hash: BlockChain::get_genesis().hash,
            timestamp: BlockChain::get_genesis().timestamp + 1,
            data: String::from("ekmfwlkm"),
            hash: String::from("d7e7efcbda3fb07db3aa416ceaefa830f3a5e19e77f5231d16de76c32abc39b2"),
        };

        chain.add(next_block.clone());

        assert_eq!(chain.blocks.len(), 2);

        chain.add(next_block.clone());

        assert_eq!(chain.blocks.len(), 2);
    }

    #[test]
    fn test_is_valid() {
        let mut chain = BlockChain {
            blocks: vec![
                BlockChain::get_genesis(),
                Block {
                    index: 1,
                    previous_hash: BlockChain::get_genesis().hash,
                    timestamp: BlockChain::get_genesis().timestamp + 1,
                    data: String::from("ekmfwlkm"),
                    hash: String::from(
                        "d7e7efcbda3fb07db3aa416ceaefa830f3a5e19e77f5231d16de76c32abc39b2",
                    ),
                },
            ],
        };

        assert!(chain.is_valid());

        chain = BlockChain {
            blocks: vec![
                BlockChain::get_genesis(),
                Block {
                    index: 1,
                    previous_hash: String::from(
                        "e9e7efcbda3fb07db3aa416ceaefa830f3a5e19e77f5231d16de76c32abc39b2",
                    ),
                    timestamp: BlockChain::get_genesis().timestamp + 1,
                    data: String::from("ekmfwlkm"),
                    hash: String::from(
                        "d7e7efcbda3fb07db3aa416ceaefa830f3a5e19e77f5231d16de76c32abc39b2",
                    ),
                },
            ],
        };

        assert!(!chain.is_valid());

        chain = BlockChain {
            blocks: vec![
                BlockChain::get_genesis(),
                Block {
                    index: 1,
                    previous_hash: BlockChain::get_genesis().hash,
                    timestamp: BlockChain::get_genesis().timestamp - 1,
                    data: String::from("ekmfwlkm"),
                    hash: String::from(
                        "d7e7efcbda3fb07db3aa416ceaefa830f3a5e19e77f5231d16de76c32abc39b2",
                    ),
                },
            ],
        };

        assert!(!chain.is_valid());

        chain = BlockChain {
            blocks: vec![
                BlockChain::get_genesis(),
                Block {
                    index: 1,
                    previous_hash: BlockChain::get_genesis().hash,
                    timestamp: BlockChain::get_genesis().timestamp,
                    data: String::from("ekmfwlkm"),
                    hash: String::from(
                        "d7e7efcbda3fb07db3aa416ceaefa830f3a5e19e77f5231d16de76c32abc39b2",
                    ),
                },
            ],
        };

        assert!(!chain.is_valid());

        chain = BlockChain {
            blocks: vec![
                BlockChain::get_genesis(),
                Block {
                    index: 1,
                    previous_hash: BlockChain::get_genesis().hash,
                    timestamp: BlockChain::get_genesis().timestamp + 1,
                    data: String::from("ekmfwlkm"),
                    hash: String::from(
                        "d7e7efcbda3fb07db3aa416ceaefa831f3a5e19e77f5231d16de76c32abc39b2",
                    ),
                },
            ],
        };

        assert!(!chain.is_valid());
    }
}
