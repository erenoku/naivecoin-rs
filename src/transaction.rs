use std::{ops::Deref, str::FromStr};

use k256::{
    ecdsa::{signature::Signer, Signature, SigningKey},
    SecretKey,
};
use k256::{
    ecdsa::{signature::Verifier, VerifyingKey},
    EncodedPoint,
};
use log::warn;
use rand_core::OsRng; // requires 'getrandom' feature
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct UnspentTxOut {
    pub tx_out_id: String,
    pub tx_out_index: u64,
    pub address: String,
    pub amount: u64,
}

#[derive(Debug)]
pub struct TxIn {
    pub tx_out_id: String,
    pub tx_out_index: u64,
    pub signature: String,
}

pub struct TxOut {
    pub address: String,
    pub amount: u64,
}

pub struct Transaction {
    pub id: String,

    pub tx_ins: Vec<TxIn>,
    pub tx_outs: Vec<TxOut>,
}

impl Transaction {
    pub fn get_transaction_id(&self) -> String {
        let tx_in_content = self
            .tx_ins
            .iter()
            .map(|tx_in| tx_in.tx_out_id.clone() + &tx_in.tx_out_index.to_string())
            .reduce(|a, b| a + &b)
            .unwrap_or(String::new());

        let tx_out_content = self
            .tx_outs
            .iter()
            .map(|tx_out| tx_out.address.clone() + &tx_out.amount.to_string())
            .reduce(|a, b| a + &b)
            .unwrap_or(String::new());

        let mut hasher = Sha256::new();

        hasher.update(tx_in_content);
        hasher.update(tx_out_content);

        format!("{:x}", hasher.finalize())
    }

    pub fn validate(&self, new_unspent_tx_outs: Vec<UnspentTxOut>) -> bool {
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

    //     const validateBlockTransactions = (aTransactions: Transaction[], aUnspentTxOuts: UnspentTxOut[], blockIndex: number): boolean => {
    //     const coinbaseTx = aTransactions[0];
    //     if (!validateCoinbaseTx(coinbaseTx, blockIndex)) {
    //         console.log('invalid coinbase transaction: ' + JSON.stringify(coinbaseTx));
    //         return false;
    //     }

    //     //check for duplicate txIns. Each txIn can be included only once
    //     const txIns: TxIn[] = _(aTransactions)
    //         .map(tx => tx.txIns)
    //         .flatten()
    //         .value();

    //     if (hasDuplicates(txIns)) {
    //         return false;
    //     }

    //     // all but coinbase transactions
    //     const normalTransactions: Transaction[] = aTransactions.slice(1);
    //     return normalTransactions.map((tx) => validateTransaction(tx, aUnspentTxOuts))
    //         .reduce((a, b) => (a && b), true);

    // };
}

impl TxIn {
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

                if let Ok(encoded_point) = &EncodedPoint::from_str(&address) {
                    if let Ok(key) = VerifyingKey::from_encoded_point(encoded_point) {
                        let signature = Signature::from_str(&self.signature).unwrap();

                        return key.verify(transaction.id.as_bytes(), &signature).is_ok();
                    }
                }

                return false;
            }
            None => {
                warn!("referenced txOut not found: {:?}", self);
                false
            }
        }
    }

    pub fn get_amount(&self, new_unspent_tx_outs: &Vec<UnspentTxOut>) -> u64 {
        find_unspent_tx_out(&self.tx_out_id, self.tx_out_index, new_unspent_tx_outs).amount
    }
}

fn find_unspent_tx_out(
    transaction_id: &String,
    index: u64,
    new_unspent_tx_outs: &Vec<UnspentTxOut>,
) -> UnspentTxOut {
    new_unspent_tx_outs
        .iter()
        .find(|new_tx_o| &new_tx_o.tx_out_id == transaction_id && new_tx_o.tx_out_index == index)
        .unwrap()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_in_validate() {
        let transaction = Transaction {
            id: String::from("a"),
            tx_ins: vec![],
            tx_outs: vec![],
        };

        let signing_key = SigningKey::random(OsRng);

        let verifying_key = VerifyingKey::from(&signing_key);
        let mut str_ver_key = EncodedPoint::from_bytes(verifying_key.to_bytes())
            .unwrap()
            .to_string();

        let signature: Signature = signing_key.sign(transaction.id.as_bytes());

        let tx_in = TxIn {
            tx_out_index: 13,
            tx_out_id: String::from("abc"),
            signature: signature.to_string(),
        };

        let new_unspent_tx_outs = vec![
            UnspentTxOut {
                tx_out_id: String::from("f"),
                tx_out_index: 98,
                address: String::new(),
                amount: 100,
            },
            UnspentTxOut {
                tx_out_id: String::from("abc"),
                tx_out_index: 13,
                address: str_ver_key.clone(),
                amount: 100,
            },
        ];

        assert!(tx_in.validate(&transaction, &new_unspent_tx_outs));

        let new_unspent_tx_outs = vec![
            UnspentTxOut {
                tx_out_id: String::from("f"),
                tx_out_index: 98,
                address: String::new(),
                amount: 100,
            },
            UnspentTxOut {
                tx_out_id: String::from("abcd"),
                tx_out_index: 13,
                address: str_ver_key.clone(),
                amount: 100,
            },
        ];

        assert!(!tx_in.validate(&transaction, &new_unspent_tx_outs));

        for _ in 0..4 {
            str_ver_key.pop();
        }
        str_ver_key += "abca";

        let new_unspent_tx_outs = vec![
            UnspentTxOut {
                tx_out_id: String::from("f"),
                tx_out_index: 98,
                address: String::new(),
                amount: 100,
            },
            UnspentTxOut {
                tx_out_id: String::from("abc"),
                tx_out_index: 13,
                address: str_ver_key.clone(),
                amount: 100,
            },
        ];

        assert!(!tx_in.validate(&transaction, &new_unspent_tx_outs));
    }
}
