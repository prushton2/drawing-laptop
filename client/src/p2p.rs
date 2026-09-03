use std::sync::{Arc, Mutex};

use anyhow::Result;
use iroh::{Endpoint, PublicKey, endpoint::{Connection, RecvStream, SendStream, presets}, protocol::{AcceptError, ProtocolHandler, Router}};

const ALPN: &[u8] = b"hello";

#[derive(Debug, Copy, Clone)]
pub enum P2PError {
    ConnectionNotEstablished
}

#[derive(Debug, Clone)]
struct Handler {
    conn: Arc<Mutex<Option<Connection>>>
}

impl Handler {
    fn new() -> Self {
        Self {
            conn: Arc::new(Mutex::new(None))
        }
    }
}

impl ProtocolHandler for Handler {
    async fn accept(&self, conn: Connection) -> Result<(), AcceptError> {
        *self.conn.lock().unwrap() = Some(conn);
        Ok(())
    }
}

pub struct P2P {
    handler: Box<Handler>,
    router: Router,
    conn: Option<(SendStream, RecvStream)>,
}

impl P2P {
    pub async fn init() -> (Self, PublicKey) {
        let handler = Box::new(Handler::new());

        let ep = Endpoint::bind(presets::N0).await.unwrap();
        let router = Router::builder(ep.clone()).accept(ALPN, handler.clone()).spawn();

        ep.online().await;
        let id = ep.id();

        (Self {
            handler: handler,
            router: router,
            conn: None,
        }, id)
    }

    pub async fn await_connection(&mut self) {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let mutex = self.handler.conn.lock().unwrap();
            if mutex.is_some() { break; }
        }

        let conn = self.handler.conn.lock().unwrap().as_mut().unwrap().clone();
        let (send, receive) = conn.accept_bi().await.unwrap();

        self.conn = Some((send, receive));
        
        let _ = self.read();
    }

    pub async fn connect(id: PublicKey) -> Self {
        let handler = Box::new(Handler::new());

        let ep = Endpoint::bind(presets::N0).await.unwrap();
        let router = Router::builder(ep.clone()).accept(ALPN, handler.clone()).spawn();
        
        let conn = ep.connect(id, ALPN).await.unwrap();
        let (send, receive) = conn.open_bi().await.unwrap();
        

        let mut this = Self {
            handler: handler,
            router: router,
            conn: Some((send, receive)),
        };

        let _ = this.send(ALPN);

        this
    }

    pub async fn send(&mut self, message: &[u8]) -> Result<(), P2PError> {
        let (send, _) = self.conn.as_mut().ok_or(P2PError::ConnectionNotEstablished)?;
        let len: [u8; 4] = (message.len() as u32).to_be_bytes();
        send.write_all(&len).await.unwrap();
        send.write_all(message).await.unwrap();
        Ok(())
    }

    pub async fn read(&mut self) -> Result<Vec<u8>, P2PError> {
        let (_, recv) = self.conn.as_mut().ok_or(P2PError::ConnectionNotEstablished)?;
        let mut byte = [0u8; 1];
        let mut length: u32 = 0;
        
        // get length of message (first 4 bytes)
        for _ in 0..4 {
            match recv.read(&mut byte).await.unwrap() {
                None => return Ok(vec![]),
                Some(_) => {
                    // println!("Read byte {:?}", byte);
                    length <<= 8;
                    length |= byte[0] as u32;
                }
            }
        }

        let mut bytes: Vec<u8> = vec![];
        bytes.resize(length as usize, 0);

        let _ = recv.read_exact(&mut bytes[..]).await;

        Ok(bytes)
    }

    pub async fn close(&mut self) {
        let _ = self.router.shutdown().await;
    }
}