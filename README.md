# naivecoin-rs

An implementation of Naivecoin in Rust

## Goals

1. Things should be simple
2. It should perform as well as possible without making things complicated
3. Should be able to accomplish basic tasks, cryptocurrencies are expected to do

## Getting Started

### Start two instances of the app

```bash
# run these command on different terminal emulators
cargo run # it runs with the default configuration
KEY_LOC=node/wallet2/private_key.pem HTTP_PORT=8001 P2P_PORT=5001 INITIAL="0.0.0.0:5000" cargo run
```

### Get Blockchain

```bash
curl localhost:8000/blocks
```

### Mine a Block

```bash 
curl -X POST localhost:8000/mineBlock
```

### Mine a Transaction

```bash
curl --data '{"address":"ADDRESS_OF_THE_SECOND_PEER", "amount":DESIRED_AMOUNT}' localhost:8000/mineTransaction
```

### Send a Transaction to Pool

```bash
curl --data '{"address":"ADDRESS_OF_THE_SECOND_PEER", "amount":DESIRED_AMOUNT}' localhost:8000/sendTransaction
# then mine a block so it gets added to the blockchain
curl -X POST localhost:8000/mineBlock
```

### Get Balance

```bash
curl localhost:8001/balance
# the balance should be DESIRED_AMOUNT
```

## TODO

- [ ] Add UI for viewing and interacting with the blockchain
- [ ] Check security

## References

- https://github.com/lhartikk/naivecoin
- https://lhartikk.github.io/
- https://docs.rs/openssl/latest/openssl/
- https://github.com/conradoqg/naivecoin
