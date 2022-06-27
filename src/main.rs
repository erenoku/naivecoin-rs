use log::warn;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

use crate::block::Block;
use chain::BlockChain;
use http_server::init_http_server;
use message::{Message, MessageType};
use p2p::{connect_to_peer, init_p2p_server};

mod block;
mod chain;
mod http_server;
mod message;
mod p2p;

static BLOCK_CHAIN: Lazy<RwLock<BlockChain>> = Lazy::new(|| {
    RwLock::new(BlockChain {
        blocks: vec![BlockChain::get_genesis()],
    })
});

static PEERS: Lazy<RwLock<Vec<String>>> = Lazy::new(|| RwLock::new(vec![]));

#[derive(Deserialize, Debug, Serialize, Clone)]
struct Config {
    http_port: String,
    p2p_port: String,
    initial_peers: String,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            http_port: env::var("HTTP_PORT").unwrap_or_else(|_| "8000".into()),
            p2p_port: env::var("P2P_PORT").unwrap_or_else(|_| "5000".into()),
            initial_peers: env::var("INITIAL").unwrap_or_default(),
        }
    }
}

fn main() {
    let config = Config::from_env();

    for peer in config.initial_peers.split(',') {
        if peer.is_empty() {
            break;
        }

        PEERS.write().unwrap().push(peer.to_owned());

        let token = connect_to_peer(peer.parse().unwrap());

        Message {
            m_type: MessageType::QueryLatest,
            content: String::new(),
        }
        .send_to_peer(&token);
    }

    let http_port = config.http_port.clone(); // will go inside move closure
    let http_handler = thread::spawn(move || init_http_server(http_port).unwrap());

    // TODO: signal handling and graceful shutdown
    init_p2p_server(config.p2p_port);
    http_handler.join().unwrap();
}
