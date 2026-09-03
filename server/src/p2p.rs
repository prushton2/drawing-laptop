use std::sync::{Arc, Mutex};

use anyhow::Result;
use iroh::{Endpoint, PublicKey, endpoint::{Connection, RecvStream, SendStream, presets}, protocol::{AcceptError, ProtocolHandler, Router}};

const ALPN: &[u8] = b"chat/0";

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

        println!("Awaiting initial bytes");
        
        let mut initial_string = String::from("");
        while initial_string.len() < 4 {
            match Self::read(self).await.unwrap() {
                Some(t) => initial_string += &t,
                None => {}
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    pub async fn connect(id: PublicKey) -> Self {
        let handler = Box::new(Handler::new());

        let ep = Endpoint::bind(presets::N0).await.unwrap();
        let router = Router::builder(ep.clone()).accept(ALPN, handler.clone()).spawn();
        
        let conn = ep.connect(id, ALPN).await.unwrap();
        let (mut send, receive) = conn.open_bi().await.unwrap();
        let _ = send.write_all(b"Open\n").await.unwrap();

        Self {
            handler: handler,
            router: router,
            conn: Some((send, receive)),
        }
    }


    pub async fn send(&mut self, message: &str) -> Result<(), P2PError> {
        let (send, _) = self.conn.as_mut().ok_or(P2PError::ConnectionNotEstablished)?;
        send.write_all(message.as_bytes()).await.unwrap();
        send.write_all(b"\n").await.unwrap();
        Ok(())
    }

    pub async fn read(&mut self) -> Result<Option<String>, P2PError> {
        let (_, recv) = self.conn.as_mut().ok_or(P2PError::ConnectionNotEstablished)?;
        let mut line = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            match recv.read(&mut byte).await.unwrap() {
                None | Some(0) => return Ok(None),        // stream finished
                _ if byte[0] == b'\n' => break,
                _ => line.push(byte[0]),
            }
        }
        Ok(Some(String::from_utf8_lossy(&line).into_owned()))
    }

    pub async fn close(&mut self) {
        let _ = self.router.shutdown().await;
    }
}