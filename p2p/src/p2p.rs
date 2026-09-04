use std::sync::{Arc, Mutex};

use anyhow::Result;
use iroh::{Endpoint, PublicKey, endpoint::{BindError, ConnectError, Connection, ConnectionError, ReadError, RecvStream, SendStream, WriteError, presets}, protocol::{AcceptError, ProtocolHandler, Router}};

const ALPN: &[u8] = b"hello";

#[derive(Debug)]
#[allow(unused)]
pub enum P2PError {
    ConnectionNotEstablished,
    BindError(BindError),
    ConnectionError(ConnectionError),
    ConnectError(ConnectError),
    WriteError(WriteError),
    ReadError(ReadError),
    PoisonedMutex,
    Timeout
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
    pub async fn init() -> Result<(Self, PublicKey), P2PError> {
        let handler = Box::new(Handler::new());

        let ep = Endpoint::bind(presets::N0).await.map_err(|e| P2PError::BindError(e))?;
        let router = Router::builder(ep.clone()).accept(ALPN, handler.clone()).spawn();

        tokio::time::timeout(std::time::Duration::from_secs(5), ep.online()).await.map_err(|_| P2PError::Timeout)?;

        // ep.online().await;
        let id = ep.id();

        Ok((Self {
            handler: handler,
            router: router,
            conn: None,
        }, id))
    }

    pub async fn await_connection(&mut self) -> Result<(), P2PError> {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let mutex = self.handler.conn.lock().unwrap();
            if mutex.is_some() { break; }
        }

        let conn = self.handler.conn.lock().map_err(|_| P2PError::PoisonedMutex)?.as_mut().unwrap().clone();
        let (send, receive) = conn.accept_bi().await.map_err(|e| P2PError::ConnectionError(e))?;

        self.conn = Some((send, receive));
        
        let _ = self.read().await;
        Ok(())
    }

    pub async fn connect(id: PublicKey) -> Result<Self, P2PError> {
        let handler = Box::new(Handler::new());

        let ep = Endpoint::bind(presets::N0).await.map_err(|e| P2PError::BindError(e))?;
        let router = Router::builder(ep.clone()).accept(ALPN, handler.clone()).spawn();
        
        let conn = ep.connect(id, ALPN).await.map_err(|e| P2PError::ConnectError(e))?;
        let (send, receive) = conn.open_bi().await.map_err(|e| P2PError::ConnectionError(e))?;
        

        let mut this = Self {
            handler: handler,
            router: router,
            conn: Some((send, receive)),
        };

        let _ = this.send(ALPN);

        Ok(this)
    }

    pub async fn send(&mut self, message: &[u8]) -> Result<(), P2PError> {
        let (send, _) = self.conn.as_mut().ok_or(P2PError::ConnectionNotEstablished)?;
        let len: [u8; 4] = (message.len() as u32).to_be_bytes();
        send.write_all(&len).await.map_err(|e| P2PError::WriteError(e))?;
        send.write_all(message).await.map_err(|e| P2PError::WriteError(e))?;
        Ok(())
    }

    pub async fn read(&mut self) -> Result<Vec<u8>, P2PError> {
        let (_, recv) = self.conn.as_mut().ok_or(P2PError::ConnectionNotEstablished)?;
        let mut byte = [0u8; 1];
        let mut length: u32 = 0;
        
        // get length of message (first 4 bytes)
        for _ in 0..4 {
            match recv.read(&mut byte).await.map_err(|e| P2PError::ReadError(e))? {
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