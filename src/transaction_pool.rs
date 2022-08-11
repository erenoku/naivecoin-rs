use log::{info, warn};
use serde::{Deserialize, Serialize};

use crate::transaction::{Transaction, TxIn, UnspentTxOut};

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionPool(pub Vec<Transaction>);

impl TransactionPool {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn add(&mut self, tx: Transaction, unspent_tx_outs: &Vec<UnspentTxOut>) -> bool {
        info!("trying to push tx: {:?} to pool", tx);

        if !tx.validate(unspent_tx_outs) {
            return false;
        }

        for tx_in in tx.tx_ins.iter() {
            if self.contains(tx_in) {
                warn!("the pool {:?} already contains {:?}", self, tx);

                return false;
            }
        }

        info!("successfullt pushed tx");

        self.0.push(tx);
        true
    }

    fn contains(&self, tx_in: &TxIn) -> bool {
        let pool_tx_ins: Vec<&TxIn> = self.0.iter().map(|tx| &tx.tx_ins).flatten().collect();

        pool_tx_ins
            .iter()
            .find(|&&pool_tx_in| {
                pool_tx_in.tx_out_id == tx_in.tx_out_id
                    && pool_tx_in.tx_out_index == tx_in.tx_out_index
            })
            .is_some()
    }
}
