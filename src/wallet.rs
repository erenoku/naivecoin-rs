use log::{error, info};
use openssl::ec::EcPoint;

use std::path::Path;
use std::sync::RwLock;

use crate::crypto::{KeyPair, PrivateKey};
use crate::transaction::{Transaction, TxIn, TxOut, UnspentTxOut};
use crate::transaction_pool::TransactionPool;
use crate::{TRANSACTIN_POOL, WALLET};

#[derive(Debug)]
pub struct Wallet {
    pub signing_key_location: String,
}

impl Wallet {
    pub fn global() -> &'static RwLock<Self> {
        WALLET.get().expect("wallet not initialized")
    }

    pub fn get_public_key(&self) -> EcPoint {
        self.get_private_key().to_public_key()
    }

    pub fn get_private_key(&self) -> PrivateKey {
        PrivateKey::read_file_pem(Path::new(&self.signing_key_location)).unwrap()
    }

    pub fn generate_private_key(&self) -> PrivateKey {
        let path = Path::new(&self.signing_key_location);

        if path.metadata().is_err() {
            let key = KeyPair::generate();
            key.private_key.write_file_pem(path).unwrap();

            info!(
                "Wallet generated. public key: {}",
                KeyPair::public_key_to_hex(&key.private_key.to_public_key())
            );

            return key.private_key;
        }

        info!(
            "Using already existing wallet. public key: {}",
            KeyPair::public_key_to_hex(&self.get_public_key())
        );

        self.get_private_key()
    }

    pub fn get_balance(address: String, unspent_tx_outs: &[UnspentTxOut]) -> u64 {
        unspent_tx_outs
            .iter()
            .filter(|u_tx_out| u_tx_out.address == address)
            .map(|u_tx_out| u_tx_out.amount)
            .sum()
    }

    pub fn find_tx_outs_for_amount(
        amount: &u64,
        my_unspent_tx_outs: Vec<&UnspentTxOut>,
    ) -> Option<(Vec<UnspentTxOut>, u64)> {
        let mut current_amount = 0;
        let mut included_unspent_tx_outs: Vec<UnspentTxOut> = vec![];

        for &my_unspent_tx_out in my_unspent_tx_outs.iter() {
            included_unspent_tx_outs.push(my_unspent_tx_out.clone());
            current_amount += my_unspent_tx_out.amount;
            if current_amount >= *amount {
                return Some((included_unspent_tx_outs, current_amount - amount));
            }
        }

        error!("not enough coins to send transaction");
        None
    }

    pub fn create_tx_outs(
        receiver_addr: String,
        my_addr: String,
        amount: u64,
        left_over_amount: u64,
    ) -> Vec<TxOut> {
        let tx_out1 = TxOut {
            address: receiver_addr,
            amount,
        };

        if left_over_amount == 0 {
            vec![tx_out1]
        } else {
            let left_over_tx = TxOut {
                address: my_addr,
                amount: left_over_amount,
            };
            vec![tx_out1, left_over_tx]
        }
    }

    // TODO: find a better place
    fn filter_tx_pool_txs(
        unspent_tx_outs: Vec<UnspentTxOut>,
        pool: &TransactionPool,
    ) -> Vec<UnspentTxOut> {
        let tx_ins: Vec<TxIn> = pool.0.iter().flat_map(|tx| tx.tx_ins.clone()).collect();

        let mut removable: Vec<UnspentTxOut> = Vec::new();
        for u_tx_out in unspent_tx_outs.clone() {
            let tx_in = tx_ins.iter().find(|a_tx_in| {
                a_tx_in.tx_out_index == u_tx_out.tx_out_index
                    && a_tx_in.tx_out_id == u_tx_out.tx_out_id
            });

            if tx_in.is_some() {
                removable.push(u_tx_out);
            }
        }

        unspent_tx_outs
            .into_iter()
            .filter(|u_tx_out| !removable.contains(u_tx_out))
            .collect()
    }

    pub fn create_transaction(
        receiver_addr: String,
        amount: u64,
        private_key: &PrivateKey,
        unspent_tx_outs: Vec<UnspentTxOut>,
    ) -> Option<Transaction> {
        let my_addr = KeyPair::public_key_to_hex(&private_key.to_public_key());
        let my_unspent_tx_outs_a: Vec<UnspentTxOut> = unspent_tx_outs
            .clone()
            .into_iter()
            .filter(|u_tx_out| u_tx_out.address == my_addr)
            .collect();
        let my_unspent_tx_outs: Vec<UnspentTxOut> =
            Self::filter_tx_pool_txs(my_unspent_tx_outs_a, &TRANSACTIN_POOL.read().unwrap());
        // filterTxPoolTxs(myUnspentTxOutsA, txPool);

        let (included_unspent_tx_outs, left_over_amount) =
            Self::find_tx_outs_for_amount(&amount, my_unspent_tx_outs.iter().collect())?;

        let unsigned_tx_ins: Vec<TxIn> = included_unspent_tx_outs
            .iter()
            .map(|u_tx_out| u_tx_out.to_unsigned_tx_in())
            .collect();

        let mut tx = Transaction {
            id: String::new(),
            tx_ins: unsigned_tx_ins,
            tx_outs: Self::create_tx_outs(receiver_addr, my_addr, amount, left_over_amount),
        };
        tx.id = tx.get_transaction_id();

        tx.tx_ins = tx
            .clone()
            .tx_ins
            .iter()
            .enumerate()
            .map(|(index, tx_in)| {
                let mut t = tx_in.clone();
                t.signature = TxIn::sign(tx.clone(), index as u64, private_key, &unspent_tx_outs);
                t
            })
            .collect();

        Some(tx)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use std::fs::File;
    use std::io::{self, BufRead};

    #[test]
    fn test_gen_key() {
        let location = tempfile::NamedTempFile::new().unwrap();
        let wallet = Wallet {
            signing_key_location: location.path().to_str().unwrap().to_owned(),
        };
        fs::remove_file(location.path()).unwrap(); // so it doesn't think this ket already exists

        let priv1 = wallet.generate_private_key();
        let priv2 = wallet.get_private_key();

        assert_eq!(priv1.to_pem().unwrap(), priv2.to_pem().unwrap());

        let pub1 = priv1.to_public_key();
        let pub2 = wallet.get_public_key();

        assert_eq!(
            KeyPair::public_key_to_hex(&pub1),
            KeyPair::public_key_to_hex(&pub2)
        );
    }

    // TODO: make test not spagetthi
    #[test]
    fn test_generate() {
        let location = tempfile::NamedTempFile::new().unwrap();

        let wallet = Wallet {
            signing_key_location: location.path().to_str().unwrap().to_owned(),
        };

        fs::remove_file(location.path()).unwrap();

        wallet.generate_private_key();

        let f = File::open(&location).unwrap();

        let mut lines = io::BufReader::new(&f).lines();
        let line = lines.next().unwrap().unwrap();

        assert!(line.starts_with("-----BEGIN EC PRIVATE KEY-----"));

        let second_line = lines.next().unwrap().unwrap();

        drop(lines);
        drop(f);

        wallet.generate_private_key();

        let f1 = File::open(location).unwrap();

        let mut lines1 = io::BufReader::new(f1).lines();
        let line1 = lines1.next().unwrap().unwrap();

        assert!(line1.starts_with("-----BEGIN EC PRIVATE KEY-----"));

        let second_line1 = lines1.next().unwrap().unwrap();

        assert_eq!(second_line, second_line1);
    }
}
