use log::info;

use crate::block::Block;
use crate::difficulter::simple::{SimpleDifficulter, START_DIFFICULTY};
use crate::difficulter::Difficulter;
use crate::transaction::{Transaction, UnspentTxOut};
use crate::transaction_pool::TransactionPool;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct BlockChain {
    pub blocks: Vec<Block>,
}

impl Default for BlockChain {
    fn default() -> Self {
        Self {
            blocks: vec![Self::get_genesis()],
        }
    }
}

impl BlockChain {
    /// add a new block if valid
    pub fn add(
        &mut self,
        new: Block,
        pool: &mut TransactionPool,
        unspent_tx_outs: &mut Vec<UnspentTxOut>,
        validator: &impl Validator,
    ) {
        // TODO: return error
        if Block::is_valid_next_block(
            &new,
            &self.get_latest().unwrap(),
            self,
            validator,
            unspent_tx_outs,
        ) {
            if let Some(ret_val) =
                Transaction::process_transaction(&new.data, unspent_tx_outs, &(new.index as u64))
            {
                self.blocks.push(new);
                *unspent_tx_outs = ret_val;
                pool.update(unspent_tx_outs);
            }
        }
    }

    /// get new_blocks and if valid completely change the self.blocks
    pub fn replace(
        &mut self,
        new_blocks: Vec<Block>,
        unspent_tx_outs: &mut Vec<UnspentTxOut>,
        transaction_pool: &mut TransactionPool,
        validator: &impl Validator,
    ) {
        let new_chain = BlockChain { blocks: new_blocks };

        if let Some(new_unspent_tx_outs) = new_chain.is_valid(validator, unspent_tx_outs) {
            if SimpleDifficulter::get_accumulated_difficulty(&new_chain)
                > SimpleDifficulter::get_accumulated_difficulty(self)
            {
                self.blocks = new_chain.blocks;
                *unspent_tx_outs = new_unspent_tx_outs;
                transaction_pool.update(unspent_tx_outs);
            }
        }
        // TODO: return error
    }

    /// return the latest block
    pub fn get_latest(&self) -> Option<Block> {
        info!("get latest");
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
            difficulty: START_DIFFICULTY,
            nonce: 0,
        };

        let hash = g.calculate_hash();
        g.hash = hash;

        g
    }

    /// check if the complete chain is valid
    // TODO: return result
    fn is_valid(
        &self,
        validator: &impl Validator,
        old_u_tx_outs: &[UnspentTxOut],
    ) -> Option<Vec<UnspentTxOut>> {
        if *self.blocks.first().unwrap() != BlockChain::get_genesis() {
            return None;
        }

        let mut new_unspent_tx_outs: Vec<UnspentTxOut> = vec![];

        let mut current_chain = BlockChain { blocks: vec![] };

        for i in 1..self.blocks.len() {
            if !Block::is_valid_next_block(
                self.blocks.get(i).unwrap(),
                self.blocks.get(i - 1).unwrap(),
                &current_chain,
                validator,
                old_u_tx_outs,
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

            current_chain
                .blocks
                .push(self.blocks.get(i).unwrap().to_owned());
        }
        Some(new_unspent_tx_outs)
    }
}

#[cfg(test)]
mod tests {
    use crate::validator::pow::PowValidator;

    use super::*;

    #[test]
    fn test_is_valid() {
        let validator = PowValidator {};
        let unspent_tx_outs: Vec<UnspentTxOut> = vec![];

        let mut chain = BlockChain {
            blocks: vec![BlockChain::get_genesis()],
        };

        let mut second = Block {
            index: 1,
            previous_hash: BlockChain::get_genesis().hash,
            timestamp: BlockChain::get_genesis().timestamp + 1,
            data: vec![],
            hash: String::new(),
            difficulty: 0,
            nonce: 0,
        };
        second.hash = second.calculate_hash();
        // Don't add new blocks like this there is a dedicated function for this called add
        chain.blocks.push(second.clone());
        assert!(chain.is_valid(&validator, &unspent_tx_outs).is_some());

        let mut third = Block {
            index: 2,
            previous_hash: BlockChain::get_genesis().hash,
            timestamp: BlockChain::get_genesis().timestamp + 2,
            data: vec![],
            hash: String::new(),
            difficulty: 0,
            nonce: 0,
        };
        third.hash = third.calculate_hash();
        let mut c1 = chain.clone();
        c1.blocks.push(third);
        assert!(c1.is_valid(&validator, &unspent_tx_outs).is_none());

        let mut forth = Block {
            index: 2,
            previous_hash: second.hash,
            timestamp: BlockChain::get_genesis().timestamp - 300,
            data: vec![],
            hash: String::new(),
            difficulty: 0,
            nonce: 0,
        };
        forth.hash = forth.calculate_hash();
        chain.blocks.push(forth);
        assert!(chain.is_valid(&validator, &unspent_tx_outs).is_none());
    }

    #[test]
    fn test_replace() {
        let validator = PowValidator {};
        let mut pool: TransactionPool = Default::default();
        let mut unspent_tx_outs: Vec<UnspentTxOut> = Default::default();

        let mut original = BlockChain {
            blocks: vec![
                BlockChain::get_genesis(),
                Block {
                    index: 1,
                    previous_hash: BlockChain::get_genesis().hash,
                    timestamp: BlockChain::get_genesis().timestamp + 1,
                    data: vec![],
                    hash: String::new(),
                    difficulty: 0,
                    nonce: 0,
                },
            ],
        };

        original.blocks[1].hash = original.blocks[1].calculate_hash();

        let mut new_block = Block {
            index: 2,
            previous_hash: original.blocks[1].hash.clone(),
            timestamp: BlockChain::get_genesis().timestamp + 2,
            data: vec![],
            hash: String::new(),
            difficulty: 0,
            nonce: 0,
        };
        new_block.hash = new_block.calculate_hash();
        let mut new_chain = original.clone();
        new_chain.add(new_block, &mut pool, &mut unspent_tx_outs, &validator);

        original.replace(
            new_chain.blocks,
            &mut unspent_tx_outs,
            &mut pool,
            &validator,
        );
        assert_eq!(original.blocks.len(), 3);
        assert_eq!(original.blocks[2].index, 2);
    }
}
