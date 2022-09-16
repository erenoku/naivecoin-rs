use log::error;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{block::Block, chain::BlockChain, transaction::UnspentTxOut, validator::Validator};

#[derive(Debug)]
pub struct PowValidator {}

impl PowValidator {
    fn has_valid_hash(hash: &str, difficulty: &u32, is_validate: bool) -> bool {
        let end = difficulty / 4 + 1;
        // end = 2

        for i in 0..end {
            // i = {0, 1}
            let a = match hash.as_bytes()[i as usize] as char {
                '0' => "0000",
                '1' => "0001",
                '2' => "0010",
                '3' => "0011",
                '4' => "0100",
                '5' => "0101",
                '6' => "0110",
                '7' => "0111",
                '8' => "1000",
                '9' => "1001",
                'a' | 'A' => "1010",
                'b' | 'B' => "1011",
                'c' | 'C' => "1100",
                'd' | 'D' => "1101",
                'e' | 'E' => "1110",
                'f' | 'F' => "1111",
                e => panic!("sha256 hash contains invalid character: {}", e),
            };

            // assert_eq(difficulty, 10)
            // difficulty % 4 = 2
            // difficulty / 4

            // it is the last iteration not all bytes have to be 0
            if i == end - 1 {
                if !a.starts_with("0".repeat(*difficulty as usize % 4).as_str()) {
                    if is_validate {
                        error!("hash doesn't match difficulty hash: {a}, difficulty: {difficulty}");
                    }
                    return false;
                }
            } else if a != "0000" {
                if is_validate {
                    error!("hash doesn't match difficulty hash: {a}, difficulty: {difficulty}");
                }
                return false;
            }
        }

        true
    }
}

impl Validator for PowValidator {
    fn is_valid(
        &self,
        prev_block: &Block,
        next_block: &Block,
        chain: &BlockChain,
        _: &[UnspentTxOut],
    ) -> bool {
        prev_block.index + 1 == next_block.index
            && prev_block.hash == next_block.previous_hash
            && next_block.calculate_hash() == next_block.hash
            && self.has_valid_difficulty(next_block, chain)
            && Self::has_valid_hash(&next_block.hash, &next_block.difficulty, true)
            && self.is_valid_timestamp(next_block, prev_block)
    }

    fn find_block(
        &self,
        prev_block: &Block,
        data: Vec<crate::transaction::Transaction>,
        difficulty: u32,
    ) -> Block {
        let index = prev_block.index + 1;
        let previous_hash = prev_block.hash.clone();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut nonce = 0;
        loop {
            let hash = Block::calculate_hash_from_data(
                &index,
                &previous_hash,
                &timestamp,
                &data,
                &difficulty,
                &nonce,
                &String::new(),
            );

            if PowValidator::has_valid_hash(&hash, &difficulty, false) {
                return Block {
                    index,
                    previous_hash,
                    timestamp,
                    data,
                    hash,
                    difficulty,
                    nonce,
                    miner_balance: 0,
                    miner_address: String::new(),
                };
            }
            nonce += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // no need to test validation logic as they are tested in chain.rs

    #[test]
    fn test_hash_matches_difficulty() {
        assert!(PowValidator::has_valid_hash(
            &String::from("0fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            &4,
            false
        ));

        assert!(PowValidator::has_valid_hash(
            &String::from("0fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            &0,
            false
        ));

        assert!(!PowValidator::has_valid_hash(
            &String::from("0fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            &5,
            false
        ));
    }
}
