use log::info;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::RwLock;
use std::thread;

use crate::block::Block;
use chain::BlockChain;
use http_server::init_http_server;
use message::{Message, MessageType};
use p2p::Server;

// TODO: use traits for p2p_handler, validator and difficulter
mod block;
mod chain;
mod difficulter;
mod http_server;
mod message;
mod p2p;
mod p2p_handler;
mod validator;

static BLOCK_CHAIN: Lazy<RwLock<BlockChain>> = Lazy::new(|| {
    RwLock::new(BlockChain {
        blocks: vec![BlockChain::get_genesis()],
    })
});

// in seconds
const BLOCK_GENERATION_INTERVAL: u32 = 10;
const DIFFICULTY_ADJUSTMENT_INTERVAL: u32 = 10;

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
    env_logger::init();
    let config = Config::from_env();

    for peer in config.initial_peers.split(',') {
        if peer.is_empty() {
            break;
        }

        let token = Server::connect_to_peer(peer.parse().unwrap());

        Message {
            m_type: MessageType::QueryLatest,
            content: String::new(),
        }
        .send_to_peer(&token);
    }

    info!(
        "server running on p2p port: {} and http port: {}",
        config.p2p_port, config.http_port
    );

    let http_port = config.http_port.clone(); // will go inside move closure
    let http_handler = thread::spawn(move || init_http_server(http_port).unwrap());

    // TODO: signal handling and graceful shutdown
    Server {
        addr: format!("0.0.0.0:{}", config.p2p_port).parse().unwrap(),
    }
    .init();
    http_handler.join().unwrap();
}
