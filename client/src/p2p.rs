use std::{env, sync::{Arc, Mutex}};

use anyhow::Result;
use iroh::{
    Endpoint, EndpointId, PublicKey, endpoint::{Connection, RecvStream, SendStream, presets}, protocol::{AcceptError, ProtocolHandler, Router},
};
use tokio::io::{AsyncBufReadExt, BufReader};

const ALPN: &[u8] = b"chat/0";

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

/// Reads and writes at the same time on one stream.
// async fn chat(mut send: SendStream, recv: RecvStream) -> Result<()> {
//     // Incoming lines print as they arrive, on their own task.
//     let mut incoming = BufReader::new(recv).lines();
//     let reader = tokio::spawn(async move {
//         while let Ok(Some(line)) = incoming.next_line().await {
//             println!("< {line}");
//         }
//         println!("* peer hung up");
//     });

//     // Outgoing lines come from stdin.
//     let mut stdin = BufReader::new(tokio::io::stdin()).lines();
//     while let Some(line) = stdin.next_line().await? {
//         send.write_all(format!("{line}\n").as_bytes()).await?;
//     }

//     send.finish()?;
//     reader.await?;
//     Ok(())
// }

// pub async fn main() -> Result<()> {
//     tracing_subscriber::fmt::init();

//     let c = Box::new(Handler);
    
//     let ep = Endpoint::bind(presets::N0).await?;
//     let router = Router::builder(ep.clone()).accept(ALPN, c.clone()).spawn();

//     match env::args().nth(1) {
//         // No argument: wait for someone to dial us.
//         None => {
//             ep.online().await;
//             println!("* your id: {}", ep.id());
//             println!("* waiting for a peer");
//             tokio::signal::ctrl_c().await?;
//         }
//         // With an id: dial, then talk.
//         Some(id) => {
//             let conn = ep.connect(id.parse::<EndpointId>()?, ALPN).await?;
//             let (send, recv) = conn.open_bi().await?;
//             println!("* connected, type away");
//             if let Err(err) = chat(send, recv).await {
//                 eprintln!("* chat ended: {err}");
//             }
//             conn.close(0u32.into(), b"bye");
//         }
//     }

//     router.shutdown().await?;
//     Ok(())
// }
pub struct P2P {
    handler: Box<Handler>,
    ep: Endpoint,
    router: Router,
    conn: Option<Connection>
}

impl P2P {
    async fn init() -> (Self, PublicKey) {
        let handler = Box::new(Handler::new());

        let ep = Endpoint::bind(presets::N0).await.unwrap();
        let router = Router::builder(ep.clone()).accept(ALPN, handler.clone()).spawn();

        ep.online().await;
        let id = ep.id();

        (Self {
            handler: handler,
            ep: ep,
            router: router,
            conn: None
        }, id)
    }

    async fn await_connection(&mut self) {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let mutex = self.handler.conn.lock().unwrap();
            if mutex.is_some() { break; }
        }

        let conn = self.handler.conn.lock().unwrap().as_mut().unwrap().clone();

        self.conn = Some(conn);
    }

    async fn connect(id: PublicKey) -> Self {
        let handler = Box::new(Handler::new());

        let ep = Endpoint::bind(presets::N0).await.unwrap();
        let router = Router::builder(ep.clone()).accept(ALPN, handler.clone()).spawn();
        
        let conn = ep.connect(id, ALPN).await.unwrap();

        Self {
            handler: handler,
            ep: ep,
            router: router,
            conn: Some(conn)
        }
    }
}