use std::time::{SystemTime, UNIX_EPOCH};

use crate::block::Block;

pub struct Validator;

impl Validator {
    pub fn is_valid(prev_block: &Block, next_block: &Block) -> bool {
        prev_block.index + 1 == next_block.index
            && prev_block.hash == next_block.previous_hash
            && next_block.calculate_hash() == next_block.hash
            && Self::hash_matches_difficulty(&next_block.hash, &next_block.difficulty)
            && Self::is_valid_timestamp(next_block, prev_block)
    }

    fn is_valid_timestamp(next_block: &Block, prev_block: &Block) -> bool {
        // TODO: timezone??
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        prev_block.timestamp - 60 < next_block.timestamp || now + 60 < next_block.timestamp
    }

    pub fn hash_matches_difficulty(hash: &str, difficulty: &u32) -> bool {
        let bin = Self::hex_to_bin(hash);

        bin.starts_with("0".repeat(*difficulty as usize).as_str())
    }

    fn hex_to_bin(hex: &str) -> String {
        let mut bin = String::new();

        for char in hex.chars() {
            let a = match char {
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
            bin.push_str(a);
        }

        bin
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
