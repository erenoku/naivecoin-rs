use actix_web::http::StatusCode;
use actix_web::{get, web, App, HttpResponse, HttpServer};

use crate::block::Block;
use crate::message::{Message, MessageType};
use crate::p2p;
use crate::{BLOCK_CHAIN, PEERS};

#[get("/blocks")]
async fn blocks() -> actix_web::Result<HttpResponse> {
    // response
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("application/json")
        .body(serde_json::to_string(&BLOCK_CHAIN.read().unwrap().blocks).unwrap()))
}

#[get("/peers")]
async fn peers() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(PEERS.read().unwrap().join("\n")))
}

async fn connect_to_peer(peer: String) -> actix_web::Result<HttpResponse> {
    if peer.is_empty() {
        return Ok(HttpResponse::build(StatusCode::BAD_REQUEST).body(""));
    }

    // PEERS.write().unwrap().push(peer.clone());

    let token = p2p::connect_to_peer(peer.parse().unwrap());

    Message {
        m_type: MessageType::QueryLatest,
        content: String::new(),
    }
    .send_to_peer(&token);

    Ok(HttpResponse::build(StatusCode::OK).body(""))
}

async fn mine_block(body: String) -> actix_web::Result<HttpResponse> {
    let next_block = Block::generate_next(body, &BLOCK_CHAIN.read().unwrap());

    BLOCK_CHAIN.write().unwrap().add(next_block);

    let msg = Message {
        m_type: MessageType::ResponseBlockchain,
        content: serde_json::to_string(&vec![BLOCK_CHAIN.read().unwrap().get_latest()]).unwrap(),
    };

    msg.broadcast();

    Ok(HttpResponse::build(StatusCode::OK).body(""))
}

#[actix_web::main]
pub async fn init_http_server(http_port: String) -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(blocks)
            .service(web::resource("/addPeer").route(web::post().to(connect_to_peer)))
            .service(web::resource("/mineBlock").route(web::post().to(mine_block)))
            .service(peers)
    })
    .bind(format!("127.0.0.1:{}", http_port))?
    .run()
    .await
}
