use log::{info, warn};
use serde::{Deserialize, Serialize};

use crate::transaction::{Transaction, TxIn, UnspentTxOut};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TransactionPool(pub Vec<Transaction>);

impl TransactionPool {
    pub fn new() -> Self {
        TransactionPool::default()
    }

    pub fn add(&mut self, tx: Transaction, unspent_tx_outs: &[UnspentTxOut]) -> bool {
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

    pub fn update(&mut self, unspent_tx_outs: &[UnspentTxOut]) {
        info!("try update pool");
        let mut r_indexes: Vec<usize> = vec![];

        for (i, pool_tx) in self.0.iter().enumerate() {
            for pool_tx_in in pool_tx.tx_ins.iter() {
                if !unspent_tx_outs.iter().any(|u_tx_out| {
                    u_tx_out.tx_out_id == pool_tx_in.tx_out_id
                        && u_tx_out.tx_out_index == pool_tx_in.tx_out_index
                }) {
                    info!("found some invalid");
                    r_indexes.push(i);
                    break;
                }
            }
        }

        for i in r_indexes.into_iter().rev() {
            info!("removing {:?} from transaction pool", self.0[i]);
            self.0.remove(i);
        }
    }

    fn contains(&self, tx_in: &TxIn) -> bool {
        let pool_tx_ins: Vec<&TxIn> = self.0.iter().flat_map(|tx| &tx.tx_ins).collect();

        pool_tx_ins.iter().any(|&pool_tx_in| {
            pool_tx_in.tx_out_id == tx_in.tx_out_id && pool_tx_in.tx_out_index == tx_in.tx_out_index
        })
    }
}
