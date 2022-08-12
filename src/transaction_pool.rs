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

    pub fn update(&mut self, unspent_tx_outs: &Vec<UnspentTxOut>) {
        info!("try update pool");
        let mut r_indexes = vec![];

        info!("unspent tx outs: {:?}", unspent_tx_outs);
        info!("pool: {:?}", self.0);

        for (i, pool_tx) in self.0.iter().enumerate() {
            for pool_tx_in in pool_tx.tx_ins.iter() {
                if unspent_tx_outs
                    .iter()
                    .find(|&u_tx_out| {
                        u_tx_out.tx_out_id == pool_tx_in.tx_out_id
                            && u_tx_out.tx_out_index == pool_tx_in.tx_out_index
                    })
                    .is_none()
                {
                    info!("found some invalid");
                    r_indexes.push(i);
                }
            }
        }

        for i in r_indexes {
            info!("removing {:?} from transaction pool", self.0[i]);
            self.0.remove(i);
        }
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