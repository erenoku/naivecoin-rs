use log::{error, warn};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::crypto::{KeyPair, PrivateKey, Signature};
use crate::COINBASE_AMOUNT;

#[derive(Clone, Debug)]
pub struct UnspentTxOut {
    pub tx_out_id: String,
    pub tx_out_index: u64,
    pub address: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxIn {
    pub tx_out_id: String,
    pub tx_out_index: u64,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxOut {
    pub address: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub id: String,

    pub tx_ins: Vec<TxIn>,
    pub tx_outs: Vec<TxOut>,
}

impl UnspentTxOut {
    pub fn to_unsigned_tx_in(&self) -> TxIn {
        TxIn {
            tx_out_id: self.tx_out_id.clone(),
            tx_out_index: self.tx_out_index,
            signature: String::new(),
        }
    }
}

impl Transaction {
    pub fn get_transaction_id(&self) -> String {
        let tx_in_content = self
            .tx_ins
            .iter()
            .map(|tx_in| tx_in.tx_out_id.clone() + &tx_in.tx_out_index.to_string())
            .reduce(|a, b| a + &b)
            .unwrap_or_default();

        let tx_out_content = self
            .tx_outs
            .iter()
            .map(|tx_out| tx_out.address.clone() + &tx_out.amount.to_string())
            .reduce(|a, b| a + &b)
            .unwrap_or_default();

        let mut hasher = Sha256::new();

        hasher.update(tx_in_content);
        hasher.update(tx_out_content);

        format!("{:x}", hasher.finalize())
    }

    pub fn validate(&self, new_unspent_tx_outs: &Vec<UnspentTxOut>) -> bool {
        if self.get_transaction_id() != self.id {
            warn!("invalid tx id: {}", self.id);
            return false;
        }

        let has_valid_tx_ins = self
            .tx_ins
            .iter()
            .map(|tx_in| tx_in.validate(self, &new_unspent_tx_outs))
            .all(|x| x);

        if !has_valid_tx_ins {
            warn!("some of the txIns are invalid in tx: {}", self.id);
            warn!("{:?}", self);
            return false;
        }

        let total_tx_in_values = self
            .tx_ins
            .iter()
            .map(|tx_in| tx_in.get_amount(&new_unspent_tx_outs))
            .reduce(|a, b| a + b)
            .unwrap_or(0);

        let total_tx_out_values = self
            .tx_outs
            .iter()
            .map(|tx_out| tx_out.amount)
            .reduce(|a, b| a + b)
            .unwrap_or(0);

        if total_tx_in_values != total_tx_out_values {
            warn!("totalTxOutValues != totalTxInValues in tx: {}", self.id);
            return false;
        }

        true
    }

    pub fn validate_block_transactions(
        new_transactions: &Vec<Self>,
        new_unspent_tx_outs: &Vec<UnspentTxOut>,
        block_index: &u64,
    ) -> bool {
        let coinbase_tx = &new_transactions[0];
        if !Self::validate_coinbase_tx(coinbase_tx, block_index) {
            warn!("invalid coinbase transaction: {:?}", coinbase_tx);
            return false;
        }

        let tx_ins: Vec<&TxIn> = new_transactions
            .iter()
            .map(|t| &t.tx_ins)
            .flatten()
            .collect();

        if TxIn::has_duplicates(tx_ins) {
            warn!("txins have duplicates");
            return false;
        }

        // transactions except coinbase transaction
        let normal_transactions = new_transactions.clone().split_off(1);
        normal_transactions
            .iter()
            .map(|tx_in| tx_in.validate(new_unspent_tx_outs))
            .all(|x| x)
    }

    fn validate_coinbase_tx(transaction: &Self, block_index: &u64) -> bool {
        if transaction.get_transaction_id() != transaction.id {
            warn!("invalid coinbase tx id: {}", transaction.id,);
            false
        } else if transaction.tx_ins.len() != 1 {
            warn!("one txIn must be specified in the coinbase transaction");
            false
        } else if transaction.tx_ins[0].tx_out_index != *block_index {
            warn!("the txIn signature in coinbase tx must be the block height");
            false
        } else if transaction.tx_outs.len() != 1 {
            warn!("invalid number of txOuts in coinbase transaction");
            false
        } else if transaction.tx_outs[0].amount != COINBASE_AMOUNT {
            warn!("invalid coinbase amount in coinbase transaction");
            false
        } else {
            true
        }
    }

    pub fn get_coinbase_tx(address: String, block_index: u64) -> Self {
        let tx_in = TxIn {
            tx_out_id: String::new(),
            tx_out_index: block_index,
            signature: String::new(),
        };

        let mut tx = Self {
            id: String::new(),
            tx_ins: vec![tx_in],
            tx_outs: vec![TxOut {
                address,
                amount: COINBASE_AMOUNT,
            }],
        };

        tx.id = tx.get_transaction_id();

        tx
    }

    pub fn update_unspent_tx_out(
        new_transactions: &Vec<Self>,
        a_unspent_tx_outs: &Vec<UnspentTxOut>,
    ) -> Vec<UnspentTxOut> {
        let new_unspent_tx_outs: Vec<UnspentTxOut> = new_transactions
            .iter()
            .map(|t| {
                t.tx_outs
                    .iter()
                    .enumerate()
                    .map(|(index, tx_out)| UnspentTxOut {
                        tx_out_id: t.id.clone(),
                        tx_out_index: index as u64,
                        address: tx_out.address.clone(),
                        amount: tx_out.amount,
                    })
                    .collect()
            })
            .reduce(|a: Vec<UnspentTxOut>, b| vec![a, b].concat())
            .unwrap_or_default();

        let consumed_tx_outs: Vec<UnspentTxOut> = new_transactions
            .iter()
            .map(|t| t.clone().tx_ins)
            .reduce(|a, b| vec![a, b].concat())
            .unwrap_or_default()
            .iter()
            .map(|tx_in| UnspentTxOut {
                tx_out_id: tx_in.tx_out_id.clone(),
                tx_out_index: tx_in.tx_out_index.clone(),
                address: String::new(),
                amount: 0,
            })
            .collect();

        a_unspent_tx_outs
            .clone()
            .into_iter()
            .filter(|u_tx_o| {
                match find_unspent_tx_out(
                    &u_tx_o.tx_out_id,
                    &u_tx_o.tx_out_index,
                    &consumed_tx_outs,
                ) {
                    Some(_) => false,
                    None => true,
                }
            })
            .chain(new_unspent_tx_outs.clone().into_iter())
            .collect()
    }

    pub fn process_transaction(
        new_transactions: &Vec<Self>,
        new_unspent_tx_outs: &Vec<UnspentTxOut>,
        block_index: &u64,
    ) -> Option<Vec<UnspentTxOut>> {
        if !Self::validate_block_transactions(&new_transactions, &new_unspent_tx_outs, block_index)
        {
            warn!("invalid block transaction");
            return None;
        }

        Some(Self::update_unspent_tx_out(
            new_transactions,
            new_unspent_tx_outs,
        ))
    }
}

impl TxIn {
    fn has_duplicates(tx_ins: Vec<&Self>) -> bool {
        let v: Vec<String> = tx_ins
            .iter()
            .map(|tx_in| tx_in.tx_out_id.clone() + &tx_in.tx_out_index.to_string())
            .collect();
        (1..v.len()).any(|i| v[i..].contains(&v[i - 1]))
    }

    pub fn validate(
        &self,
        transaction: &Transaction,
        new_unspent_tx_outs: &Vec<UnspentTxOut>,
    ) -> bool {
        match new_unspent_tx_outs.iter().find(|u_tx_out| {
            u_tx_out.tx_out_id == self.tx_out_id && u_tx_out.tx_out_index == self.tx_out_index
        }) {
            Some(referenced_u_tx_out) => {
                let address = &referenced_u_tx_out.address;

                if let Ok(public_key) = KeyPair::public_key_from_hex(address) {
                    if let Ok(signature) = Signature::from_string(&self.signature) {
                        let r = signature
                            .verify(&transaction.id.as_bytes(), public_key)
                            .unwrap();

                        if !r {
                            warn!("failed signature verify for {:?}", self);
                        }

                        return r;
                    } else {
                        error!("error getting signature");
                        return false;
                    }
                }
                warn!(
                    "could not parse address to public key address: {}",
                    referenced_u_tx_out.address
                );

                if let Err(e) = KeyPair::public_key_from_hex(&referenced_u_tx_out.address) {
                    warn!("{e}");
                }

                false
            }
            None => {
                warn!("referenced txOut not found: {:?}", self);
                false
            }
        }
    }

    pub fn get_amount(&self, new_unspent_tx_outs: &Vec<UnspentTxOut>) -> u64 {
        find_unspent_tx_out(&self.tx_out_id, &self.tx_out_index, new_unspent_tx_outs)
            .unwrap()
            .amount
    }

    // TODO: fix spagetthi
    pub fn sign(
        tx: Transaction,
        tx_in_index: u64,
        private_key: &PrivateKey,
        new_unspent_tx_outs: &Vec<UnspentTxOut>,
    ) -> String {
        let tx_in = &tx.tx_ins[tx_in_index as usize];

        let data_to_sign = tx.id;
        let referenced_u_tx_out =
            match find_unspent_tx_out(&tx_in.tx_out_id, &tx_in.tx_out_index, new_unspent_tx_outs) {
                Some(tx_out) => tx_out,
                None => {
                    panic!("could not find referenced txOut");
                }
            };

        let referenced_address = referenced_u_tx_out.address;

        let public_key = private_key.to_public_key();
        if KeyPair::public_key_to_hex(&public_key) != referenced_address {
            panic!("trying to sign an input with private key that does not match the adress that is referecned in tx_in");
        }

        let signature = Signature::from_sign(data_to_sign.as_bytes(), private_key).unwrap();
        signature.to_encoded()
    }
}

fn find_unspent_tx_out(
    transaction_id: &String,
    index: &u64,
    new_unspent_tx_outs: &Vec<UnspentTxOut>,
) -> Option<UnspentTxOut> {
    new_unspent_tx_outs
        .clone()
        .into_iter()
        .find(|new_tx_o| &new_tx_o.tx_out_id == transaction_id && &new_tx_o.tx_out_index == index)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use openssl::ecdsa::EcdsaSig;
//     use rand_core::OsRng; // requires 'getrandom' feature

//     #[test]
//     fn test_tx_in_validate() {
//         let transaction = Transaction {
//             id: String::from("a"),
//             tx_ins: vec![],
//             tx_outs: vec![],
//         };

//         // let signing_key = SigningKey::random(OsRng);

//         // let verifying_key = VerifyingKey::from(&signing_key);
//         // let mut str_ver_key = EncodedPoint::from_bytes(verifying_key.to_bytes())
//         //     .unwrap()
//         //     .to_string();

//         // let signature: Signature = signing_key.sign(transaction.id.as_bytes());

//         let signature: EcdsaSig = todo!();

//         let tx_in = TxIn {
//             tx_out_index: 13,
//             tx_out_id: String::from("abc"),
//             signature: String::from_utf8(signature.to_der().unwrap()).unwrap(),
//         };

//         let new_unspent_tx_outs = vec![
//             UnspentTxOut {
//                 tx_out_id: String::from("f"),
//                 tx_out_index: 98,
//                 address: String::new(),
//                 amount: 100,
//             },
//             UnspentTxOut {
//                 tx_out_id: String::from("abc"),
//                 tx_out_index: 13,
//                 address: str_ver_key.clone(),
//                 amount: 100,
//             },
//         ];

//         assert!(tx_in.validate(&transaction, &new_unspent_tx_outs));

//         let new_unspent_tx_outs = vec![
//             UnspentTxOut {
//                 tx_out_id: String::from("f"),
//                 tx_out_index: 98,
//                 address: String::new(),
//                 amount: 100,
//             },
//             UnspentTxOut {
//                 tx_out_id: String::from("abcd"),
//                 tx_out_index: 13,
//                 address: str_ver_key.clone(),
//                 amount: 100,
//             },
//         ];

//         assert!(!tx_in.validate(&transaction, &new_unspent_tx_outs));

//         for _ in 0..4 {
//             str_ver_key.pop();
//         }
//         str_ver_key += "abca";

//         let new_unspent_tx_outs = vec![
//             UnspentTxOut {
//                 tx_out_id: String::from("f"),
//                 tx_out_index: 98,
//                 address: String::new(),
//                 amount: 100,
//             },
//             UnspentTxOut {
//                 tx_out_id: String::from("abc"),
//                 tx_out_index: 13,
//                 address: str_ver_key.clone(),
//                 amount: 100,
//             },
//         ];

//         assert!(!tx_in.validate(&transaction, &new_unspent_tx_outs));
//     }
// }
