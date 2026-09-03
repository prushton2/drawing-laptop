use enigo::Mouse;
use enigo::{Button, Coordinate::Abs, Direction::Click, Enigo, Settings};
use iroh::endpoint::presets::{self, Preset};
use iroh::EndpointId;

use crate::protocol::IntoBytes;

mod p2p;
mod protocol;

#[tokio::main]
async fn main() {
    // let mut enigo = Enigo::new(&Settings::default()).unwrap();
    
    // enigo.move_mouse(1280, 800, Abs).unwrap();
    // enigo.button(Button::Left, Click).unwrap();
    let (mut server, key) = p2p::P2P::init().await.unwrap();
    println!("Key: {}", key);
    server.await_connection().await.unwrap();
    println!("Connection established");

    let server_info = protocol::ServerInformation {
        width: 2560,
        height: 1600
    };

    let _ = server.send(&server_info.into_bytes()).await;

    loop {
        let response = server.read().await.unwrap();
        println!("Received: {:?}", response);
    }
}