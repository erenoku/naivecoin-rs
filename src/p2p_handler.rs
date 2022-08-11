use log::info;
use mio::net::TcpStream;

use crate::message::{Message, MessageType};
use crate::BLOCK_CHAIN;
use crate::TRANSACTIN_POOL;

pub struct P2PHandler;

impl P2PHandler {
    pub fn handle_receive_msg(msg: &Message, connection: &mut TcpStream) {
        info!("{:?}", msg.m_type);

        match msg.m_type {
            MessageType::QueryAll => {
                let msg = Message {
                    m_type: MessageType::ResponseBlockchain,
                    content: serde_json::to_string(&BLOCK_CHAIN.read().unwrap().blocks).unwrap(),
                };
                msg.send_request(connection);
            }
            MessageType::QueryLatest => {
                info!("writin");
                let msg = Message {
                    m_type: MessageType::ResponseBlockchain,
                    content: serde_json::to_string(&vec![BLOCK_CHAIN.read().unwrap().get_latest()])
                        .unwrap(),
                };
                msg.send_request(connection);
            }
            MessageType::QueryTransactionPool => {
                let msg = Message {
                    m_type: MessageType::ResponseTransactionPool,
                    content: serde_json::to_string(&*TRANSACTIN_POOL.read().unwrap()).unwrap(),
                };
                msg.send_request(connection);
            }
            MessageType::ResponseBlockchain => {
                msg.handle_blockchain_response();
            }
            MessageType::ResponseTransactionPool => {
                msg.handle_transaction_pool_response();
            }
        }
    }
}
