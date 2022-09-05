use log::{error, info};
use primitive_types::U512;
use sha2::{Digest, Sha256};
use std::{
    ops::{Div, Mul},
    sync::{Arc, RwLock},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::{block::Block, crypto::KeyPair, transaction::UnspentTxOut, wallet::Wallet};

use super::Validator;

pub struct PosValidator {
    pub wallet: Arc<RwLock<Wallet>>,
    pub unspent_tx_outs: Arc<RwLock<Vec<UnspentTxOut>>>,
}

const ALLOW_WITHOUT_COIN_INDEX: u8 = 10;

/// does the calculation SHA256(prevhash + address + timestamp) <= 2^256 * balance / diff
/// Reference: https://blog.ethereum.org/2014/07/05/stake
fn check_special_hash(
    index: u32,
    prev_hash: &[u8],
    address: String,
    balance: u64,
    diff: u32,
) -> bool {
    let mut balance = balance;
    if index <= ALLOW_WITHOUT_COIN_INDEX as u32 {
        balance += 1;
    }

    let mut hasher = Sha256::new();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    hasher.update(prev_hash);
    hasher.update(&address);
    hasher.update(timestamp.to_be_bytes());

    let hash: [u8; 32] = *hasher.finalize().as_ref();

    let left_side = U512::from_big_endian(&hash);
    let right_side: U512 = U512::from(2).pow(256.into()).mul(balance).div(diff);

    let valid = left_side <= right_side;
    if !valid {
        error!("check_special_hash false");
    }
    valid
}

impl Validator for PosValidator {
    fn is_valid(
        &self,
        prev_block: &crate::block::Block,
        next_block: &crate::block::Block,
        chain: &crate::chain::BlockChain,
        unspent_tx_outs: &Vec<UnspentTxOut>,
    ) -> bool {
        let pub_key = &self.wallet.read().unwrap().get_public_key();
        let my_balance = Wallet::get_balance(KeyPair::public_key_to_hex(pub_key), unspent_tx_outs);
        check_special_hash(
            next_block.index,
            prev_block.hash.as_bytes(),
            KeyPair::public_key_to_hex(pub_key),
            my_balance,
            next_block.difficulty,
        ) && prev_block.index + 1 == next_block.index
            && prev_block.hash == next_block.previous_hash
            && next_block.calculate_hash() == next_block.hash
            && self.has_valid_difficulty(next_block, chain)
            && self.is_valid_timestamp(next_block, prev_block)
    }

    fn find_block(
        &self,
        prev_block: &crate::block::Block,
        data: Vec<crate::transaction::Transaction>,
        difficulty: u32,
    ) -> Block {
        let pub_key = &self.wallet.read().unwrap().get_public_key();
        info!("got wallet");
        let my_balance = Wallet::get_balance(
            KeyPair::public_key_to_hex(pub_key),
            &self.unspent_tx_outs.read().unwrap(),
        );

        loop {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let hash = Block::calculate_hash_from_data(
                &(prev_block.index + 1),
                &prev_block.hash,
                &timestamp,
                &data,
                &difficulty,
                &0,
            );

            if check_special_hash(
                prev_block.index + 1,
                prev_block.hash.as_bytes(),
                KeyPair::public_key_to_hex(pub_key),
                my_balance,
                difficulty,
            ) {
                return Block {
                    index: (prev_block.index + 1),
                    previous_hash: prev_block.hash.clone(),
                    timestamp,
                    data,
                    hash,
                    difficulty,
                    nonce: 0,
                };
            }

            thread::sleep(Duration::from_secs_f32(0.9)); // wait a little less than 1 second
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_check_special_hash() {
//         todo!()
//     }
// }
