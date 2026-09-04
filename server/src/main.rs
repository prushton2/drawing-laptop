use p2p::protocol::FromBytes;
use p2p::{self, protocol, protocol::IntoBytes};

use crate::mouse::Mouse;

mod mouse;

#[tokio::main]
async fn main() {
    // init mouse controller
    let mut mouse = mouse::enigo::EnigoMouse::new();
    // let mut mouse = mouse::dummy::DummyMouse::new();

    let (mut server, key) = p2p::P2P::init().await.unwrap();
    let pin = p2p::remote_key_store::generate_key();
    p2p::remote_key_store::set(&pin, &key.to_string()).await;
    println!("Key: {}", key);
    println!("Pin: {}", pin);

    server.await_connection().await.unwrap();
    println!("Connection established");

    let server_info = protocol::ServerInformation {
        width: 1920,
        height: 1080
    };

    let _ = server.send(&server_info.into_bytes()).await;

    loop {
        let response = server.read().await.unwrap();
        let parsed = p2p::protocol::FromBytes::parse(&response);
        match parsed {
            FromBytes::MouseMove(message) => {
                mouse.move_mouse(message.x, message.y);
            }
            FromBytes::MouseClick(message) => {
                mouse.click_mouse(message.button, message.state);
            }
            _ => {}
        }
    }
}