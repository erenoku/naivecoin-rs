#![forbid(unsafe_code)]

use log::{error, info};
use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::RwLock;
use std::thread;

use crate::block::Block;
use crate::wallet::Wallet;
use chain::BlockChain;
use http_server::init_http_server;
use p2p::Server;

// TODO: use traits for p2p_handler, validator and difficulter
mod block;
mod chain;
mod crypto;
mod difficulter;
mod http_server;
mod message;
mod p2p;
mod p2p_handler;
mod transaction;
mod validator;
mod wallet;

static BLOCK_CHAIN: Lazy<RwLock<BlockChain>> = Lazy::new(|| {
    RwLock::new(BlockChain {
        blocks: vec![BlockChain::get_genesis()],
    })
});

static WALLET: OnceCell<RwLock<Wallet>> = OnceCell::new();

// in seconds
const BLOCK_GENERATION_INTERVAL: u32 = 10;
const DIFFICULTY_ADJUSTMENT_INTERVAL: u32 = 10;

const COINBASE_AMOUNT: u64 = 50;

#[derive(Deserialize, Debug, Serialize, Clone)]
struct Config {
    http_port: String,
    p2p_port: String,
    initial_peers: String,
    key_location: String,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            http_port: env::var("HTTP_PORT").unwrap_or(String::from("8000")),
            p2p_port: env::var("P2P_PORT").unwrap_or(String::from("5000")),
            initial_peers: env::var("INITIAL").unwrap_or_default(),
            key_location: env::var("KEY_LOC")
                .unwrap_or(String::from("./node/wallet/private_key.pem")),
        }
    }
}

fn main() {
    env_logger::init();

    let config = Config::from_env();

    let wallet = RwLock::new(Wallet {
        signing_key_location: config.key_location,
    });
    wallet
        .read()
        .expect("could read the wallet")
        .generate_private_key();
    WALLET.set(wallet).unwrap();

    for peer in config.initial_peers.split(',') {
        if peer.is_empty() {
            break;
        }

        if let Ok(peer) = peer.parse() {
            Server::connect_to_peer(peer);
        } else {
            error!("could not parse peer: {}", &peer);
        }
    }

    info!(
        "server running on p2p port: {} and http port: {}",
        config.p2p_port, config.http_port
    );

    let http_port = config.http_port.clone(); // will go inside move closure
    let http_handler = thread::spawn(move || init_http_server(http_port));

    // TODO: signal handling and graceful shutdown
    Server {
        addr: format!("0.0.0.0:{}", config.p2p_port)
            .parse()
            .expect("error parsing server address"),
    }
    .init();
    http_handler.join().unwrap();
}
