mod http_server;

use log::{error, info};
use naivecoin_rs::chain::BlockChain;
use naivecoin_rs::p2p_handler::P2PHandler;
use naivecoin_rs::transaction::UnspentTxOut;
use naivecoin_rs::transaction_pool::TransactionPool;
// use naivecoin_rs::validator::pos::PosValidator;
use naivecoin_rs::validator::pow::PowValidator;
use naivecoin_rs::validator::Validator;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::thread;

use http_server::init_http_server;
use naivecoin_rs::p2p::Server;
use naivecoin_rs::wallet::Wallet;

#[derive(Deserialize, Debug, Serialize, Clone)]
struct Config {
    http_port: String,
    p2p_port: String,
    initial_peers: String,
    key_location: String,
}

#[derive(Debug)]
pub struct App<V: Validator> {
    pub block_chain: Arc<RwLock<BlockChain>>,
    pub transaction_pool: Arc<RwLock<TransactionPool>>,
    pub wallet: Arc<RwLock<Wallet>>,
    pub unspent_tx_outs: Arc<RwLock<Vec<UnspentTxOut>>>,
    pub validator: Arc<RwLock<V>>,
}

impl<V: Validator + Send + Sync> App<V> {
    fn new(
        validator: Arc<RwLock<V>>,
        wallet: Arc<RwLock<Wallet>>,
        unspent_tx_outs: Arc<RwLock<Vec<UnspentTxOut>>>,
    ) -> App<V> {
        wallet.read().unwrap().generate_private_key();

        App {
            block_chain: Default::default(),
            transaction_pool: Default::default(),
            wallet,
            unspent_tx_outs,
            validator,
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            http_port: env::var("HTTP_PORT").unwrap_or_else(|_| String::from("8000")),
            p2p_port: env::var("P2P_PORT").unwrap_or_else(|_| String::from("5000")),
            initial_peers: env::var("INITIAL").unwrap_or_default(),
            key_location: env::var("KEY_LOC")
                .unwrap_or_else(|_| String::from("./node/wallet/private_key.pem")),
        }
    }
}

fn main() {
    env_logger::init();

    let config = Config::from_env();
    let wallet = Arc::new(RwLock::new(Wallet {
        signing_key_location: config.key_location,
    }));
    let unspent_tx_outs: Arc<RwLock<Vec<UnspentTxOut>>> = Default::default();
    let validator = Arc::new(RwLock::new(PowValidator {
        // wallet: wallet.clone(),
        // unspent_tx_outs: unspent_tx_outs.clone(),
    }));
    let app = Arc::new(RwLock::new(App::new(validator, wallet, unspent_tx_outs)));

    for peer in config.initial_peers.split(',') {
        if peer.is_empty() {
            break;
        }

        if let Ok(peer) = peer.parse() {
            Server::<PowValidator>::connect_to_peer(peer);
        } else {
            error!("could not parse peer: {}", &peer);
        }
    }

    info!(
        "server running on p2p port: {} and http port: {}",
        config.p2p_port, config.http_port
    );

    let http_port = config.http_port.clone(); // will go inside move closure
    let happ = app.clone();
    let http_handler = thread::spawn(move || init_http_server(http_port, happ));

    // TODO: signal handling and graceful shutdown
    thread::spawn(move || {
        let rapp = app.read().unwrap();
        Server {
            addr: format!("0.0.0.0:{}", config.p2p_port)
                .parse()
                .expect("error parsing server address"),
            handler: P2PHandler {
                chain: rapp.block_chain.clone(),
                transaction_pool: rapp.transaction_pool.clone(),
                unspent_tx_outs: rapp.unspent_tx_outs.clone(),
                validator: rapp.validator.clone(),
            },
        }
        .init();
    })
    .join()
    .unwrap();
    http_handler.join().unwrap();
}
