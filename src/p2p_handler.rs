use std::sync::{Arc, RwLock};

use log::info;
use mio::net::TcpStream;

use crate::{
    chain::BlockChain,
    message::{Message, MessageType},
    transaction::UnspentTxOut,
    transaction_pool::TransactionPool,
    validator::Validator,
};

pub struct P2PHandler<V>
where
    V: Validator,
{
    pub chain: Arc<RwLock<BlockChain>>,
    pub transaction_pool: Arc<RwLock<TransactionPool>>,
    pub unspent_tx_outs: Arc<RwLock<Vec<UnspentTxOut>>>,
    pub validator: Arc<RwLock<V>>,
}

impl<V: Validator + Send + Sync> P2PHandler<V> {
    pub fn handle_receive_msg(&self, msg: &Message, connection: &mut TcpStream) {
        info!("{:?}", msg.m_type);

        match msg.m_type {
            MessageType::QueryAll => {
                let msg = Message {
                    m_type: MessageType::ResponseBlockchain,
                    content: serde_json::to_string(&self.chain.read().unwrap().blocks).unwrap(),
                };
                msg.send_request(connection);
            }
            MessageType::QueryLatest => {
                info!("writin");
                let msg = Message {
                    m_type: MessageType::ResponseBlockchain,
                    content: serde_json::to_string(&vec![self.chain.read().unwrap().get_latest()])
                        .unwrap(),
                };
                msg.send_request(connection);
            }
            MessageType::QueryTransactionPool => {
                let msg = Message {
                    m_type: MessageType::ResponseTransactionPool,
                    content: serde_json::to_string(&*self.transaction_pool.read().unwrap())
                        .unwrap(),
                };
                msg.send_request(connection);
            }
            MessageType::ResponseBlockchain => {
                msg.handle_blockchain_response::<V>(
                    &mut self.chain.write().unwrap(),
                    &mut self.transaction_pool.write().unwrap(),
                    &mut self.unspent_tx_outs.write().unwrap(),
                    &*self.validator.read().unwrap(),
                );
            }
            MessageType::ResponseTransactionPool => {
                msg.handle_transaction_pool_response::<V>(
                    &mut self.transaction_pool.write().unwrap(), // FIXME: deadlock
                    &mut self.unspent_tx_outs.write().unwrap(),
                );
            }
        }
    }
}
