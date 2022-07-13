use log::error;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{block::Block, chain::BlockChain, difficulter::Difficulter, BLOCK_CHAIN};

pub struct Validator;

impl Validator {
    pub fn is_valid(prev_block: &Block, next_block: &Block, chain: &BlockChain) -> bool {
        prev_block.index + 1 == next_block.index
            && prev_block.hash == next_block.previous_hash
            && next_block.calculate_hash() == next_block.hash
            && Self::has_valid_difficulty(next_block, chain)
            && Self::hash_matches_difficulty(&next_block.hash, &next_block.difficulty)
            && Self::is_valid_timestamp(next_block, prev_block)
    }

    fn is_valid_timestamp(next_block: &Block, prev_block: &Block) -> bool {
        // TODO: timezone??
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

    pub fn has_valid_difficulty(block: &Block, chain: &BlockChain) -> bool {
        let r = block.difficulty >= Difficulter::get_difficulty(chain);
        if !r {
            error!("block doesn't have valid difficulty")
        }

        r
    }

    pub fn hash_matches_difficulty(hash: &str, difficulty: &u32) -> bool {
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
                'a' => "1010",
                'b' => "1011",
                'c' => "1100",
                'd' => "1101",
                'e' => "1110",
                'f' => "1111",
                'A' => "1010",
                'B' => "1011",
                'C' => "1100",
                'D' => "1101",
                'E' => "1110",
                'F' => "1111",
                e => panic!("sha256 hash contains invalid character: {}", e),
            };

            // assert_eq(difficulty, 10)
            // difficulty % 4 = 2
            // difficulty / 4

            // it is the last iteration not all bytes have to be 0
            if i == end - 1 {
                if !a.starts_with("0".repeat(*difficulty as usize % 4).as_str()) {
                    error!("hash doesn't match difficulty");
                    return false;
                }
            } else if a != "0000" {
                error!("hash doesn't match difficulty");
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // no need to test validation logic as they are tested in chain.rs

    #[test]
    fn test_hash_matches_difficulty() {
        assert!(Validator::hash_matches_difficulty(
            &String::from("0fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            &4
        ));

        assert!(Validator::hash_matches_difficulty(
            &String::from("0fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            &0
        ));

        assert!(!Validator::hash_matches_difficulty(
            &String::from("0fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
            &5
        ));
    }
}
