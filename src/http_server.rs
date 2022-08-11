use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;

use crate::block::{Block, UNSPENT_TX_OUTS};
use crate::crypto::KeyPair;
use crate::message::{Message, MessageType};
use crate::wallet::Wallet;
use crate::BLOCK_CHAIN;
use crate::{p2p, TRANSACTIN_POOL};

fn blocks() -> rouille::Response {
    rouille::Response::json(&BLOCK_CHAIN.read().unwrap().blocks)
}

fn connect_to_peer(peer: String) -> rouille::Response {
    if peer.is_empty() {
        return rouille::Response::text("").with_status_code(500);
    }

    let token = p2p::Server::connect_to_peer(peer.parse().unwrap());

    Message {
        m_type: MessageType::QueryLatest,
        content: String::new(),
    }
    .send_to_peer(&token);

    rouille::Response::text("")
}

fn mine_raw_block(body: String) -> rouille::Response {
    let next_block = Block::generate_next_raw(
        serde_json::from_str(&body).unwrap(),
        &BLOCK_CHAIN.read().unwrap(),
    );

    BLOCK_CHAIN.write().unwrap().add(next_block);

    let msg = Message {
        m_type: MessageType::ResponseBlockchain,
        content: serde_json::to_string(&vec![BLOCK_CHAIN.read().unwrap().get_latest()]).unwrap(),
    };

    msg.broadcast();

    rouille::Response::text("")
}

#[derive(Deserialize, Serialize)]
struct TxData {
    address: String,
    amount: u64,
}

fn mine_transaction(body: String) -> rouille::Response {
    let data: TxData = serde_json::from_str(&body).unwrap();

    if let Some(next_block) = Block::generate_next_with_transaction(data.address, data.amount) {
        BLOCK_CHAIN.write().unwrap().add(next_block.clone());

        let msg = Message {
            m_type: MessageType::ResponseBlockchain,
            content: serde_json::to_string(&vec![BLOCK_CHAIN.read().unwrap().get_latest()])
                .unwrap(),
        };
        msg.broadcast();

        rouille::Response::json(&next_block)
    } else {
        rouille::Response::text("error mining transaction").with_status_code(500)
    }
}

fn send_transaction(body: String) -> rouille::Response {
    let private_key = Wallet::global().read().unwrap().get_private_key();

    let data: TxData = serde_json::from_str(&body).expect("error parsing body");
    let tx = Wallet::create_transaction(
        data.address,
        data.amount,
        &private_key,
        UNSPENT_TX_OUTS.read().unwrap().to_vec(),
    )
    .unwrap();

    let ok = TRANSACTIN_POOL
        .write()
        .unwrap()
        .add(tx.clone(), &UNSPENT_TX_OUTS.read().unwrap());

    if !ok {
        return rouille::Response::text("could not send transaction").with_status_code(500);
    }

    Message {
        m_type: MessageType::ResponseTransactionPool,
        content: serde_json::to_string(&*TRANSACTIN_POOL.read().unwrap()).unwrap(),
    }
    .broadcast();

    rouille::Response::json(&tx)
}

fn get_pool() -> rouille::Response {
    let pool = TRANSACTIN_POOL.read().unwrap();
    rouille::Response::json(&*pool)
}

fn get_balance() -> rouille::Response {
    let pub_key = Wallet::global().read().unwrap().get_public_key();
    let balance = Wallet::get_balance(
        KeyPair::public_key_to_hex(&pub_key),
        &UNSPENT_TX_OUTS.read().unwrap(),
    );

    rouille::Response::text(balance.to_string())
}

fn mine_block() -> rouille::Response {
    let next_block = Block::generate_next();
    BLOCK_CHAIN.write().unwrap().add(next_block);

    let msg = Message {
        m_type: MessageType::ResponseBlockchain,
        content: serde_json::to_string(&vec![BLOCK_CHAIN.read().unwrap().get_latest()]).unwrap(),
    };
    msg.broadcast();

    rouille::Response::text("")
}

pub fn init_http_server(http_port: String) {
    rouille::start_server(format!("127.0.0.1:{}", http_port), move |request| {
        rouille::router!(request,

         (GET) (/blocks) => {
            blocks()
         },

         (GET) (/balance) => {
             get_balance()
         },

         (GET) (/pool) => {
             get_pool()
         },

         (POST) (/mineBlock) => {
             mine_block()
         },

         (POST) (/addPeer) => {
            let mut body =  String::new();
            request.data().unwrap().read_to_string(&mut body).unwrap();

            connect_to_peer(body)
         },

         (POST) (/mineRawBlock) => {
            let mut body = String::new();
            request.data().unwrap().read_to_string(&mut body).unwrap();

            mine_raw_block(body)
         },

         (POST) (/mineTransaction) => {
            let mut body = String::new();
            request.data().unwrap().read_to_string(&mut body).unwrap();

            mine_transaction(body)
         },

         (POST) (/sendTransaction) => {
            let mut body = String::new();
            request.data().unwrap().read_to_string(&mut body).unwrap();

            send_transaction(body)
         },

         _ => rouille::Response::empty_404()

        )
    });
}
