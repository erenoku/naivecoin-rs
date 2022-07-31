use log::info;
use mio::net::TcpStream;

use crate::message::{Message, MessageType};
use crate::BLOCK_CHAIN;

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
                let msg = Message {
                    m_type: MessageType::ResponseBlockchain,
                    content: serde_json::to_string(&vec![BLOCK_CHAIN.read().unwrap().get_latest()])
                        .unwrap(),
                };

                msg.send_request(connection);
            }
            MessageType::ResponseBlockchain => {
                msg.handle_blockchain_response();
            }
        }
    }
}
