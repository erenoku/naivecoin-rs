use log::info;
use mio::event::Event;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Registry, Token};
use once_cell::sync::Lazy;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::str::from_utf8;
use std::sync::Arc;
use std::sync::RwLock;

use std::{thread, thread::JoinHandle};

use crate::message::{Message, MessageType};
use crate::p2p_handler::P2PHandler;

const SERVER: Token = Token(0);

pub type ConnectionState = (TcpStream, Vec<u8>, u32);

static POLL: Lazy<RwLock<Poll>> = Lazy::new(|| RwLock::new(Poll::new().unwrap()));
static UNIQUE_TOKEN: Lazy<RwLock<Token>> = Lazy::new(|| RwLock::new(Token(SERVER.0 + 1)));
static CONNECTIONS: Lazy<RwLock<HashMap<Token, ConnectionState>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub struct Server {
    pub addr: SocketAddr,
}

impl Server {
    // init the p2p server return the thread handler
    pub fn init(&self) -> JoinHandle<()> {
        let addr = Arc::new(self.addr);

        thread::spawn(move || {
            let mut events = Events::with_capacity(128);

            let mut listener = TcpListener::bind(*addr).unwrap();

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

                                let token = Self::next(UNIQUE_TOKEN.write().unwrap().borrow_mut());
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
                                c.insert(token, (connection, vec![0; 4096], 0));
                                drop(c);
                            }
                        }
                        token => {
                            let mut c = CONNECTIONS.write().unwrap();
                            // Maybe received an event for a TCP connection.
                            let done = if let Some(connection) = c.get_mut(&token) {
                                let (connection, buf, bytes_read) = connection;
                                Self::handle_connection_event(
                                    POLL.write().unwrap().registry(),
                                    connection,
                                    buf,
                                    bytes_read,
                                    event,
                                )
                                .unwrap()
                            } else {
                                // Sporadic events happen, we can safely ignore them.
                                false
                            };
                            if done {
                                if let Some((mut connection, ..)) = c.remove(&token) {
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

    pub fn connect_to_peer(addr: SocketAddr) -> Token {
        let mut connection = TcpStream::connect(addr).unwrap();
        let token = Self::next(UNIQUE_TOKEN.write().unwrap().borrow_mut());

        POLL.write()
            .unwrap()
            .registry()
            .register(
                &mut connection,
                token,
                Interest::READABLE.add(Interest::WRITABLE),
            )
            .unwrap();

        CONNECTIONS
            .write()
            .unwrap()
            .insert(token, (connection, vec![0; 4096], 0));

        token
    }

    pub fn send_to_peer(
        t: &Token,
        buf: &[u8],
        connection: Option<&mut TcpStream>,
    ) -> std::io::Result<usize> {
        if let Some(stream) = connection {
            stream.write(buf)
        } else {
            let mut c = CONNECTIONS.write().unwrap();
            let (stream, ..) = c.get_mut(t).unwrap();
            stream.write(buf)
        }
    }

    pub fn broadcast(buf: &[u8]) {
        let mut c = CONNECTIONS.write().unwrap();

        let mut tokens: Vec<Token> = vec![];

        for (t, _) in c.iter() {
            tokens.push(*t);
        }
        for t in tokens.iter() {
            let (stream, ..) = c.get_mut(t).unwrap();
            stream.write_all(buf).unwrap();
        }
    }

    /// Returns `true` if the connection is done.
    fn handle_connection_event(
        registry: &Registry,
        connection: &mut TcpStream,
        buf: &mut Vec<u8>,
        bytes_read: &mut u32,
        event: &Event,
    ) -> io::Result<bool> {
        let query_chain_msg = Message {
            m_type: MessageType::QueryLatest,
            content: String::new(),
        }
        .serialize();

        let query_transaction_msg = Message {
            m_type: MessageType::QueryTransactionPool,
            content: String::new(),
        }
        .serialize();

        let chain_data = query_chain_msg.as_bytes();
        let transaction_data = query_transaction_msg.as_bytes();

        if event.is_writable() {
            // We can (maybe) write to the connection.
            match Self::send_to_peer(&event.token(), chain_data, Some(connection)) {
                // We want to write the entire `DATA` buffer in a single go. If we
                // write less we'll return a short write error (same as
                // `io::Write::write_all` does).
                Ok(n) if n < chain_data.len() => return Err(io::ErrorKind::WriteZero.into()),
                Ok(_) => {
                    // After we've written something we'll reregister the connection
                    // to only respond to readable events.
                    registry.reregister(connection, event.token(), Interest::READABLE)?
                }
                // Would block "errors" are the OS's way of saying that the
                // connection is not actually ready to perform this I/O operation.
                Err(ref err) if Self::would_block(err) => {}
                // Got interrupted (how rude!), we'll try again.
                Err(ref err) if Self::interrupted(err) => {
                    return Self::handle_connection_event(
                        registry, connection, buf, bytes_read, event,
                    )
                }
                // Other errors we'll consider fatal.
                Err(err) => return Err(err),
            }

            match Self::send_to_peer(&event.token(), transaction_data, Some(connection)) {
                // We want to write the entire `DATA` buffer in a single go. If we
                // write less we'll return a short write error (same as
                // `io::Write::write_all` does).
                Ok(n) if n < transaction_data.len() => return Err(io::ErrorKind::WriteZero.into()),
                Ok(_) => {
                    // After we've written something we'll reregister the connection
                    // to only respond to readable events.
                    registry.reregister(connection, event.token(), Interest::READABLE)?
                }
                // Would block "errors" are the OS's way of saying that the
                // connection is not actually ready to perform this I/O operation.
                Err(ref err) if Self::would_block(err) => {}
                // Got interrupted (how rude!), we'll try again.
                Err(ref err) if Self::interrupted(err) => {
                    return Self::handle_connection_event(
                        registry, connection, buf, bytes_read, event,
                    )
                }
                // Other errors we'll consider fatal.
                Err(err) => return Err(err),
            }
        }

        if event.is_readable() {
            let mut connection_closed = false;

            // We can (maybe) read from the connection.
            'outer: loop {
                match connection.read(&mut buf[*bytes_read as usize..]) {
                    Ok(0) => {
                        // Reading 0 bytes means the other side has closed the
                        // connection or is done writing, then so are we.
                        connection_closed = true;
                        break;
                    }
                    Ok(n) => {
                        *bytes_read += n as u32;
                        if *bytes_read as usize == buf.len() {
                            buf.resize(buf.len() + 1024, 0);
                        }
                    }
                    // Would block "errors" are the OS's way of saying that the
                    // connection is not actually ready to perform this I/O operation.
                    Err(ref err) if Self::would_block(err) => {
                        let mut last_read: bool;
                        loop {
                            let received_data = &buf[..*bytes_read as usize];

                            let s = received_data
                                .iter()
                                .cloned()
                                .take_while(|&ch| ch != 0)
                                .collect::<Vec<u8>>();

                            // last read
                            last_read = s.len() + 1 == received_data.len();

                            // partial read
                            if !received_data[s.len()] == b'\0' {
                                break 'outer;
                            }

                            *buf = buf[s.len() + 1..].to_vec();
                            *bytes_read -= s.len() as u32 + 1;

                            // logic
                            if let Ok(str_buf) = from_utf8(&s) {
                                match serde_json::from_str::<Message>(str_buf) {
                                    Ok(msg) => {
                                        Self::handle_receive_msg(&msg, connection);
                                    }
                                    Err(e) => {
                                        info!("error {e}");
                                        break 'outer;
                                    }
                                }
                            }

                            if last_read {
                                *buf = vec![0; 4096];
                                *bytes_read = 0_u32;
                                break 'outer;
                            }
                        }
                    }
                    Err(ref err) if Self::interrupted(err) => continue,
                    // Other errors we'll consider fatal.
                    Err(err) => return Err(err),
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

    fn handle_receive_msg(msg: &Message, connection: &mut TcpStream) {
        P2PHandler::handle_receive_msg(msg, connection)
    }
}
