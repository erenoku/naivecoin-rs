use log::info;
use mio::event::Event;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use once_cell::sync::Lazy;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::str::from_utf8;
use std::sync::RwLock;
use std::{thread, thread::JoinHandle};

use crate::message::{Message, MessageType};
use crate::BLOCK_CHAIN;

fn handle_receive_msg(msg: &str, connection: &mut TcpStream) {
    let message = Message::get_message(msg);

    info!("{:?}", message.m_type);

    match message.m_type {
        MessageType::QueryAll => {
            let msg = Message {
                m_type: MessageType::ResponseBlockchain,
                content: serde_json::to_string(&BLOCK_CHAIN.read().unwrap().blocks).unwrap(),
            };
            msg.send_request(connection);
        }
        MessageType::QueryLatest => {
            let msg = Message {
                m_type: MessageType::ResponseBlockchain,
                content: serde_json::to_string(&vec![BLOCK_CHAIN.read().unwrap().get_latest()])
                    .unwrap(),
            };

            msg.send_request(connection);
        }
        MessageType::ResponseBlockchain => {
            message.handle_blockchain_response();
        }
    }
}

pub fn connect_to_peer(addr: SocketAddr) -> Token {
    let mut connection = TcpStream::connect(addr).unwrap();
    let token = next(UNIQUE_TOKEN.write().unwrap().borrow_mut());

    POLL.write()
        .unwrap()
        .registry()
        .register(
            &mut connection,
            token,
            Interest::READABLE.add(Interest::WRITABLE),
        )
        .unwrap();

    CONNECTIONS.write().unwrap().insert(token, connection);

    token
}

pub fn send_to_peer(t: &Token, buf: &[u8]) {
    let mut c = CONNECTIONS.write().unwrap();
    let stream = c.get_mut(t).unwrap();

    stream.write_all(buf).unwrap();
}

pub fn broadcast(buf: &[u8]) {
    let c = CONNECTIONS.read().unwrap();

    for (_, mut stream) in c.iter() {
        stream.write_all(buf).unwrap();
    }
}

const SERVER: Token = Token(0);

static POLL: Lazy<RwLock<Poll>> = Lazy::new(|| RwLock::new(Poll::new().unwrap()));
static UNIQUE_TOKEN: Lazy<RwLock<Token>> = Lazy::new(|| RwLock::new(Token(SERVER.0 + 1)));
static CONNECTIONS: Lazy<RwLock<HashMap<Token, TcpStream>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

// init the p2p server return the thread handler
pub fn init_p2p_server(port: String) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut events = Events::with_capacity(128);

        let addr = format!("0.0.0.0:{}", port).parse().unwrap();

        let mut listener = TcpListener::bind(addr).unwrap();

        POLL.write()
            .unwrap()
            .registry()
            .register(&mut listener, SERVER, Interest::READABLE)
            .unwrap();

        loop {
            POLL.write().unwrap().poll(&mut events, None).unwrap();

            for event in &events {
                match event.token() {
                    SERVER => {
                        // Received an event for the TCP server socket, which
                        // indicates we can accept an connection.
                        loop {
                            let (mut connection, address) = match listener.accept() {
                                Ok((connection, address)) => (connection, address),
                                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                                    // If we get a `WouldBlock` error we know our
                                    // listener has no more incoming connections queued,
                                    // so we can return to polling and wait for some
                                    // more.
                                    break;
                                }
                                Err(e) => {
                                    // If it was any other kind of error, something went
                                    // wrong and we terminate with an error.
                                    // TODO: error handling
                                    panic!("{}", e);
                                }
                            };

                            info!("Accepted connection from: {}", address);

                            let token = next(UNIQUE_TOKEN.write().unwrap().borrow_mut());
                            POLL.read()
                                .unwrap()
                                .registry()
                                .register(
                                    &mut connection,
                                    token,
                                    Interest::READABLE.add(Interest::WRITABLE),
                                )
                                .unwrap();

                            let mut c = CONNECTIONS.write().unwrap();
                            c.insert(token, connection);
                            drop(c);
                        }
                    }
                    token => {
                        let mut c = CONNECTIONS.write().unwrap();
                        // Maybe received an event for a TCP connection.
                        let done = if let Some(connection) = c.get_mut(&token) {
                            handle_connection_event(connection, event).unwrap()
                        } else {
                            // Sporadic events happen, we can safely ignore them.
                            false
                        };
                        if done {
                            if let Some(mut connection) = c.remove(&token) {
                                POLL.write()
                                    .unwrap()
                                    .registry()
                                    .deregister(&mut connection)
                                    .unwrap();
                            }
                        }
                    }
                }
            }
        }
    })
}

/// Returns `true` if the connection is done.
fn handle_connection_event(connection: &mut TcpStream, event: &Event) -> io::Result<bool> {
    if event.is_readable() {
        let mut connection_closed = false;
        let mut received_data = vec![0; 4096];
        let mut bytes_read = 0;
        // We can (maybe) read from the connection.
        loop {
            match connection.read(&mut received_data[bytes_read..]) {
                Ok(0) => {
                    // Reading 0 bytes means the other side has closed the
                    // connection or is done writing, then so are we.
                    connection_closed = true;
                    break;
                }
                Ok(n) => {
                    bytes_read += n;
                    if bytes_read == received_data.len() {
                        received_data.resize(received_data.len() + 1024, 0);
                    }
                }
                // Would block "errors" are the OS's way of saying that the
                // connection is not actually ready to perform this I/O operation.
                Err(ref err) if would_block(err) => break,
                Err(ref err) if interrupted(err) => continue,
                // Other errors we'll consider fatal.
                Err(err) => return Err(err),
            }
        }

        if bytes_read != 0 {
            let received_data = &received_data[..bytes_read];
            if let Ok(str_buf) = from_utf8(received_data) {
                handle_receive_msg(str_buf, connection);
                info!("Received data: {}", str_buf.trim_end());
            } else {
                info!("Received (none UTF-8) data: {:?}", received_data);
            }
        }

        if connection_closed {
            info!("Connection closed");
            return Ok(true);
        }
    }

    Ok(false)
}

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

fn interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}

fn next(current: &mut Token) -> Token {
    let next = current.0;
    current.0 += 1;
    Token(next)
}
