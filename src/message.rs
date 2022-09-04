use log::{info, warn};
use mio::{net::TcpStream, Token};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::thread;

use crate::{
    block::Block,
    chain::BlockChain,
    p2p::Server,
    transaction::{Transaction, UnspentTxOut},
    transaction_pool::TransactionPool,
    validator::{self, Validator},
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum MessageType {
    QueryLatest,
    QueryAll,
    ResponseBlockchain,
    QueryTransactionPool,
    ResponseTransactionPool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub m_type: MessageType,
    pub content: String,
}

impl Message {
    /// send self to the peer and handle the response
    /// if doesn't handle response use send_response(&self, stream: &mut TcpStream)
    pub fn send_to_peer<V: Validator + Send + Sync>(&self, peer: &Token) {
        Server::<V>::send_to_peer(peer, self.serialize().as_bytes(), None).unwrap();
    }

    pub fn send_request(&self, stream: &mut TcpStream) {
        let json = self.serialize();
        let buf = json.as_bytes();

        stream.write_all(buf).unwrap();
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap() + "\0"
    }

    pub fn handle_blockchain_response<V: Validator + Send + Sync>(
        &self,
        chain: &mut BlockChain,
        pool: &mut TransactionPool,
        unspent_tx_outs: &mut Vec<UnspentTxOut>,
        validator: &impl Validator,
    ) {
        info!("received: {}", self.content);
        let mut received_blocks: Vec<Block> = serde_json::from_str(&self.content).unwrap();
        received_blocks.sort_by(|a, b| a.index.cmp(&b.index));
        let latest_block_received = received_blocks.last().unwrap();
        let latest_block_held = chain.get_latest().unwrap();

        if latest_block_received.index > latest_block_held.index {
            if latest_block_held.hash == latest_block_received.previous_hash {
                chain.add(
                    latest_block_received.clone(),
                    pool,
                    unspent_tx_outs,
                    validator,
                );

                let latest = chain.get_latest().clone();

                thread::spawn(|| {
                    Message {
                        m_type: MessageType::ResponseBlockchain,
                        content: serde_json::to_string(&vec![latest]).unwrap(),
                    }
                    .broadcast::<V>();
                });
            } else if received_blocks.len() == 1 {
                thread::spawn(|| {
                    Message {
                        m_type: MessageType::QueryAll,
                        content: String::new(),
                    }
                    .broadcast::<V>();
                });
            } else {
                chain.replace(received_blocks, unspent_tx_outs, pool, validator);
            }
        }
        // else received blockchain is not longer than current blockchain. Do nothing
    }

    pub fn handle_transaction_pool_response<V: Validator + Send + Sync>(
        &self,
        pool: &mut TransactionPool,
        unspent_tx_outs: &Vec<UnspentTxOut>,
    ) {
        let received_transactions: Vec<Transaction> =
            serde_json::from_str(&self.content).expect("error parsing json");
        if received_transactions.is_empty() {
            warn!("received_transactions.len() == 0");
            return;
        }
        for received_tx in received_transactions {
            let ok = pool.add(received_tx, unspent_tx_outs);

            if !ok {
                warn!("error adding transaction");
            } else {
                Message {
                    m_type: MessageType::ResponseTransactionPool,
                    content: serde_json::to_string(pool).unwrap(),
                }
                .broadcast::<V>()
            }
        }
    }

    pub fn broadcast<V: Validator + Send + Sync>(self) {
        Server::<V>::broadcast(self.serialize().as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let serialized = Message {
            m_type: MessageType::QueryAll,
            content: String::new(),
        }
        .serialize();

        assert_eq!(
            serialized,
            r#"{"m_type":"QueryAll","content":""}"#.to_owned() + "\0"
        );
    }
}
