use std::io::Read;

use crate::block::Block;
use crate::message::{Message, MessageType};
use crate::p2p;
use crate::BLOCK_CHAIN;

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

fn mine_block(body: String) -> rouille::Response {
    let next_block = Block::generate_next(body, &BLOCK_CHAIN.read().unwrap());

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

         (POST) (/addPeer) => {
            let mut body =  String::new();
            request.data().unwrap().read_to_string(&mut body).unwrap();

            connect_to_peer(body)
         },

         (POST) (/mineBlock) => {
            let mut body= String::new();
            request.data().unwrap().read_to_string(&mut body).unwrap();

            mine_block(body)
         },

         _ => rouille::Response::empty_404()

        )
    });
}
