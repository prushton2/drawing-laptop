use tokio::sync::Mutex;
use std::sync::Arc;

use p2p::protocol::FromBytes;
use p2p::{self, protocol, protocol::IntoBytes};

use crate::mouse::Mouse;

mod mouse;
mod window;

#[tokio::main]
async fn main() {
    // init mouse controller
    // let mut mouse = mouse::enigo::EnigoMouse::new();
    let mut mouse = mouse::dummy::DummyMouse::new();

    let (server, key) = p2p::P2P::init().await.unwrap();
    let pin = p2p::remote_key_store::generate_key();
    let key = key.to_string();
    p2p::remote_key_store::set(&pin, &key.to_string()).await;

    let server_info = protocol::ServerInformation {
        width: 1920,
        height: 1080
    };

    let p2p_arc = Arc::new(Mutex::new(server));
    let p2p_arc_clone = Arc::clone(&p2p_arc);

    let _ = iced::application(
        move || {
            let arc: Arc<Mutex<p2p::P2P>> = p2p_arc_clone.clone();
            window::Window::boot(server_info, &pin, &key, arc)
        },
        window::Window::update,
        window::Window::view
    )
        .run();

    let mut server_lock = p2p_arc.lock().await;

    let _ = server_lock.send(&server_info.into_bytes()).await;

    loop {
        let response = server_lock.read().await.unwrap();
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