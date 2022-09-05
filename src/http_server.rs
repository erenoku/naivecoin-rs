use log::info;
use naivecoin_rs::validator::Validator;
use serde::{Deserialize, Serialize};

use std::io::Read;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use naivecoin_rs::block::Block;
use naivecoin_rs::crypto::KeyPair;
use naivecoin_rs::message::{Message, MessageType};
use naivecoin_rs::p2p;
use naivecoin_rs::wallet::Wallet;

use crate::App;

fn blocks<V: Validator>(app: &App<V>) -> rouille::Response {
    rouille::Response::json(&app.block_chain.read().unwrap().blocks)
}

fn connect_to_peer<V: Validator + Send + Sync>(peer: String, app: &App<V>) -> rouille::Response {
    if peer.is_empty() {
        return rouille::Response::text("").with_status_code(500);
    }

    let token = p2p::Server::<V>::connect_to_peer(peer.parse().unwrap());

    Message {
        m_type: MessageType::QueryLatest,
        content: String::new(),
    }
    .send_to_peer::<V>(&token);

    rouille::Response::text("")
}

fn mine_raw_block<V: Validator + Send + Sync>(body: String, app: &App<V>) -> rouille::Response {
    let mut chain = app.block_chain.write().unwrap();

    let next_block = Block::generate_next_raw(
        serde_json::from_str(&body).unwrap(),
        &chain,
        &*app.validator.read().unwrap(),
    );

    chain.add(
        next_block,
        &mut app.transaction_pool.write().unwrap(),
        &mut app.unspent_tx_outs.write().unwrap(),
        &*app.validator.read().unwrap(),
    );

    let msg = Message {
        m_type: MessageType::ResponseBlockchain,
        content: serde_json::to_string(&vec![chain.get_latest()]).unwrap(),
    };

    msg.broadcast::<V>();

    rouille::Response::text("")
}

#[derive(Deserialize, Serialize)]
struct TxData {
    address: String,
    amount: u64,
}

fn mine_transaction<V: Validator + Send + Sync>(body: String, app: &App<V>) -> rouille::Response {
    let data: TxData = serde_json::from_str(&body).unwrap();

    let mut chain = app.block_chain.write().unwrap();
    let wallet = app.wallet.read().unwrap();
    let mut pool = app.transaction_pool.write().unwrap();
    let u_tx_outs = app.unspent_tx_outs.write().unwrap();

    if let Some(next_block) = Block::generate_next_with_transaction(
        data.address,
        data.amount,
        &chain,
        &wallet,
        &pool,
        u_tx_outs,
        &*app.validator.read().unwrap(),
    ) {
        let mut u_tx_outs = app.unspent_tx_outs.write().unwrap();
        chain.add(
            next_block.clone(),
            &mut pool,
            &mut u_tx_outs,
            &*app.validator.read().unwrap(),
        );

        let msg = Message {
            m_type: MessageType::ResponseBlockchain,
            content: serde_json::to_string(&vec![chain.get_latest()]).unwrap(),
        };
        drop(chain);
        drop(wallet);
        drop(pool);
        msg.broadcast::<V>();

        rouille::Response::json(&next_block)
    } else {
        rouille::Response::text("error mining transaction").with_status_code(500)
    }
}

fn send_transaction<V: Validator + Send + Sync>(body: String, app: &App<V>) -> rouille::Response {
    let (tx, msg) = {
        let private_key = app.wallet.read().unwrap().get_private_key();
        let mut pool = app.transaction_pool.write().unwrap();
        let u_tx_outs = app.unspent_tx_outs.read().unwrap();

        let data: TxData = serde_json::from_str(&body).expect("error parsing body");
        let tx = Wallet::create_transaction(
            data.address,
            data.amount,
            &private_key,
            &u_tx_outs.to_vec(),
            &pool,
        )
        .unwrap();

        let ok = pool.add(tx.clone(), &u_tx_outs);

        if !ok {
            return rouille::Response::text("could not send transaction").with_status_code(500);
        }

        (
            tx,
            Message {
                m_type: MessageType::ResponseTransactionPool,
                content: serde_json::to_string(&*pool).unwrap(),
            },
        )
    };
    msg.broadcast::<V>();

    rouille::Response::json(&tx)
}

fn get_pool<V: Validator>(app: &App<V>) -> rouille::Response {
    let pool = app.transaction_pool.read().unwrap();
    rouille::Response::json(&*pool)
}

fn get_balance<V: Validator>(app: &App<V>) -> rouille::Response {
    let wallet = app.wallet.read().unwrap();
    let u_tx_outs = app.unspent_tx_outs.read().unwrap();

    let pub_key = wallet.get_public_key();
    let balance = Wallet::get_balance(KeyPair::public_key_to_hex(&pub_key), &u_tx_outs);

    rouille::Response::text(balance.to_string())
}

fn get_public_key<V: Validator>(app: &App<V>) -> rouille::Response {
    let public_key = app.wallet.read().unwrap().get_public_key();
    rouille::Response::text(KeyPair::public_key_to_hex(&public_key))
}

fn mine_block<V: Validator + Send + Sync>(app: &App<V>) -> rouille::Response {
    let msg = {
        let mut chain = app.block_chain.write().unwrap();
        let wallet = app.wallet.read().unwrap();
        let mut pool = app.transaction_pool.write().unwrap();

        let next_block =
            Block::generate_next(&chain, &wallet, &pool, &*app.validator.read().unwrap());

        let mut unspent_tx_outs = app.unspent_tx_outs.write().unwrap();
        chain.add(
            next_block,
            &mut pool,
            &mut unspent_tx_outs,
            &*app.validator.read().unwrap(),
        );

        Message {
            m_type: MessageType::ResponseBlockchain,
            content: serde_json::to_string(&vec![chain.get_latest()]).unwrap(),
        }
    };
    msg.broadcast::<V>();

    // thread::sleep(Duration::from_secs_f32(0.5));

    rouille::Response::text("")
}

pub fn init_http_server<V: Validator + Send + Sync + 'static>(
    http_port: String,
    app: Arc<RwLock<App<V>>>,
) {
    rouille::start_server(format!("127.0.0.1:{}", http_port), move |request| {
        rouille::router!(request,

         (GET) (/blocks) => {
            blocks(&app.read().unwrap())
         },

         (GET) (/addr) => {
            get_public_key(&app.read().unwrap())
         },

         (GET) (/balance) => {
             get_balance(&app.read().unwrap())
         },

         (GET) (/pool) => {
             get_pool(&app.read().unwrap())
         },

         (POST) (/mineBlock) => {
             mine_block(&app.read().unwrap())
         },

         (POST) (/addPeer) => {
            let mut body =  String::new();
            request.data().unwrap().read_to_string(&mut body).unwrap();

            connect_to_peer(body, &app.read().unwrap())
         },

         (POST) (/mineRawBlock) => {
            let mut body = String::new();
            request.data().unwrap().read_to_string(&mut body).unwrap();

            mine_raw_block(body, &app.read().unwrap())
         },

         (POST) (/mineTransaction) => {
            let mut body = String::new();
            request.data().unwrap().read_to_string(&mut body).unwrap();

            mine_transaction(body, &app.read().unwrap())
         },

         (POST) (/sendTransaction) => {
            let mut body = String::new();
            request.data().unwrap().read_to_string(&mut body).unwrap();

            send_transaction(body, &app.read().unwrap())
         },

         _ => rouille::Response::empty_404()

        )
    });
}
