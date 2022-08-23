use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    process::{Child, Command, Stdio},
    time::Duration,
};
use tempfile::NamedTempFile;

const HTTP_PORT_0: &str = "8000";
const HTTP_PORT_1: &str = "8001";
const HTTP_PORT_2: &str = "8002";

struct InstanceConfig {
    pub http_port: String,
    pub p2p_port: String,
    pub key_loc: String,
    pub initial: Vec<String>,
}

fn start_instance(config: &InstanceConfig) -> Child {
    Command::new("./target/debug/naivecoin-rs")
        .env("HTTP_PORT", &config.http_port)
        .env("KEY_LOC", &config.key_loc)
        .env("INITIAL", &config.initial.join(","))
        .env("P2P_PORT", &config.p2p_port)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to execute process")
}

fn get_tmp_key_loc() -> String {
    NamedTempFile::new()
        .unwrap()
        .path()
        .to_str()
        .unwrap()
        .into()
}

fn start_instances() -> Vec<Child> {
    let mut children = vec![];
    let configs = vec![
        InstanceConfig {
            http_port: HTTP_PORT_0.into(),
            p2p_port: "5000".into(),
            initial: vec![],
            key_loc: get_tmp_key_loc(),
        },
        InstanceConfig {
            http_port: HTTP_PORT_1.into(),
            p2p_port: "5001".into(),
            initial: vec![String::from("0.0.0.0:5000")],
            key_loc: get_tmp_key_loc(),
        },
        InstanceConfig {
            http_port: HTTP_PORT_2.into(),
            p2p_port: "5002".into(),
            initial: vec![String::from("0.0.0.0:5000")],
            key_loc: get_tmp_key_loc(),
        },
    ];
    for config in configs.iter() {
        let c = start_instance(config);
        children.push(c);
        std::thread::sleep(Duration::from_secs(1));
    }

    children
}

async fn mine_block(client: &Client, port: &str) {
    client
        .post(format!("http://localhost:{}/mineBlock", port))
        .send()
        .await
        .unwrap();
}

async fn get_balance(client: &Client, port: &str) -> u32 {
    let req = client
        .get(format!("http://localhost:{}/balance", port))
        .send()
        .await
        .unwrap();

    let text = req.text().await.unwrap();

    text.parse().unwrap()
}

// TODO: get these from main crate
// this will be done by seperating main.rs to lib.rs and main.rs
// this way integration tests will be able to import necessery structs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxIn {
    pub tx_out_id: String,
    pub tx_out_index: u64,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxOut {
    pub address: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub id: String,

    pub tx_ins: Vec<TxIn>,
    pub tx_outs: Vec<TxOut>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Block {
    pub index: u32,
    pub previous_hash: String,
    pub timestamp: u64,
    pub data: Vec<Transaction>,
    pub hash: String,
    pub nonce: u32,
    pub difficulty: u32,
}

async fn get_blocks(client: &Client, port: &str) -> Vec<Block> {
    let req = client
        .get(format!("http://localhost:{}/blocks", port))
        .send()
        .await
        .unwrap();

    let body = req.text().await.unwrap();
    serde_json::from_str(&body).unwrap()
}

async fn get_addr(client: &Client, port: &str) -> String {
    let req = client
        .get(format!("http://localhost:{}/addr", port))
        .send()
        .await
        .unwrap();

    req.text().await.unwrap()
}

async fn mine_transaction(client: &Client, port: &str, addr: &String, amount: u64) {
    client
        .post(format!("http://localhost:{}/mineTransaction", port))
        .body(format!(
            "{{\"address\":\"{}\", \"amount\":{}}}",
            addr, amount
        ))
        .send()
        .await
        .unwrap();
}

async fn send_transaction(client: &Client, port: &str, addr: &String, amount: u64) {
    client
        .post(format!("http://localhost:{}/sendTransaction", port))
        .body(format!(
            "{{\"address\":\"{}\", \"amount\":{}}}",
            addr, amount
        ))
        .send()
        .await
        .unwrap();
}

async fn get_pool(client: &Client, port: &str) -> Vec<Transaction> {
    let req = client
        .get(format!("http://localhost:{}/pool", port))
        .send()
        .await
        .unwrap();

    let body = req.text().await.unwrap();
    serde_json::from_str(&body).unwrap()
}

#[tokio::test]
async fn test_all() {
    let instances = start_instances();

    std::thread::sleep(Duration::from_secs(1));

    let client = Client::new();

    // test mining and getting balance
    mine_block(&client, HTTP_PORT_0).await;
    let balance = get_balance(&client, HTTP_PORT_0).await;
    assert_eq!(balance, 50_u32);
    mine_block(&client, HTTP_PORT_0).await;
    mine_block(&client, HTTP_PORT_0).await;
    let balance = get_balance(&client, HTTP_PORT_0).await;
    assert_eq!(balance, 150_u32);

    std::thread::sleep(Duration::from_secs(1));

    // test if blocks are received
    let blocks0 = get_blocks(&client, HTTP_PORT_0).await;
    let blocks1 = get_blocks(&client, HTTP_PORT_1).await;
    let blocks2 = get_blocks(&client, HTTP_PORT_2).await;
    assert_eq!(blocks0, blocks1);
    assert_eq!(blocks1, blocks2);

    std::thread::sleep(Duration::from_secs(1));

    // test if mining transactions work
    let addr2 = get_addr(&client, HTTP_PORT_2).await;
    mine_transaction(&client, HTTP_PORT_0, &addr2, 100).await;
    let balance0 = get_balance(&client, HTTP_PORT_0).await;
    let balance2 = get_balance(&client, HTTP_PORT_2).await;
    assert_eq!(balance0, 100_u32);
    assert_eq!(balance2, 100_u32);

    // test if sending transactions to pool work
    send_transaction(&client, HTTP_PORT_0, &addr2, 50).await;
    send_transaction(&client, HTTP_PORT_0, &addr2, 50).await;
    let balance0 = get_balance(&client, HTTP_PORT_0).await;
    let balance2 = get_balance(&client, HTTP_PORT_2).await;
    assert_eq!(balance0, 100_u32);
    assert_eq!(balance2, 100_u32);

    std::thread::sleep(Duration::from_secs(1));

    mine_block(&client, HTTP_PORT_1).await;

    std::thread::sleep(Duration::from_secs(1));

    let balance0 = get_balance(&client, HTTP_PORT_0).await;
    let balance1 = get_balance(&client, HTTP_PORT_1).await;
    let balance2 = get_balance(&client, HTTP_PORT_2).await;
    assert_eq!(balance0, 0_u32);
    assert_eq!(balance1, 50_u32);
    assert_eq!(balance2, 200_u32);

    // check if pool has been emptied
    let pool0 = get_pool(&client, HTTP_PORT_0).await;
    let pool1 = get_pool(&client, HTTP_PORT_1).await;
    let pool2 = get_pool(&client, HTTP_PORT_2).await;
    assert_eq!(pool0.len(), 0);
    assert_eq!(pool0, pool1);
    assert_eq!(pool1, pool2);

    for mut instance in instances {
        instance.kill().expect("could not kill child process");
        let status = instance.wait().unwrap();
        assert!(status.code().is_none() || status.code() == Some(0));
    }
}
