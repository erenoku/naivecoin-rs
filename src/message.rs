use mio::{net::TcpStream, Token};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::thread;

use crate::{p2p::Server, Block, BLOCK_CHAIN};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum MessageType {
    QueryLatest,
    QueryAll,
    ResponseBlockchain,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Message {
    pub m_type: MessageType,
    pub content: String,
}

impl Message {
    /// send self to the peer and handle the response
    /// if doesn't handle response use send_response(&self, stream: &mut TcpStream)
    pub fn send_to_peer(&self, peer: &Token) {
        Server::send_to_peer(peer, self.serialize().as_bytes(), None).unwrap();
    }

    pub fn send_request(&self, stream: &mut TcpStream) {
        let json = self.serialize();
        stream.write_all(json.as_bytes()).unwrap();
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn get_message(msg: &str) -> Message {
        serde_json::from_str(msg).unwrap()
    }

    pub fn handle_blockchain_response(&self) {
        let mut received_blocks: Vec<Block> = serde_json::from_str(&self.content).unwrap();
        received_blocks.sort_by(|a, b| a.index.cmp(&b.index));
        let latest_block_received = received_blocks.last().unwrap();
        let latest_block_held = BLOCK_CHAIN.read().unwrap().get_latest();

        if latest_block_received.index > latest_block_held.index {
            if latest_block_held.hash == latest_block_received.previous_hash {
                BLOCK_CHAIN
                    .write()
                    .unwrap()
                    .add(latest_block_received.clone());

                thread::spawn(|| {
                    Message {
                        m_type: MessageType::ResponseBlockchain,
                        content: serde_json::to_string(&vec![BLOCK_CHAIN
                            .read()
                            .unwrap()
                            .get_latest()])
                        .unwrap(),
                    }
                    .broadcast();
                });
            } else if received_blocks.len() == 1 {
                thread::spawn(|| {
                    Message {
                        m_type: MessageType::QueryAll,
                        content: String::new(),
                    }
                    .broadcast();
                });
            } else {
                BLOCK_CHAIN.write().unwrap().replace(received_blocks);
            }
        }
        // else received blockchain is not longer than current blockchain. Do nothing
    }

    pub fn broadcast(self) {
        Server::broadcast(self.serialize().as_bytes());
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

        assert!(serialized == r##"{"m_type":"QueryAll","content":""}"##);
    }

    #[test]
    fn test_deserialize() {
        assert!(
            Message {
                m_type: MessageType::QueryLatest,
                content: String::new()
            } == Message::get_message(r##"{"m_type":"QueryLatest","content":""}"##)
        );
    }
}
