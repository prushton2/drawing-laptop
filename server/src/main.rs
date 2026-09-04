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


    let server_info = protocol::ServerInformation {
        width: 1920,
        height: 1080
    };

    let _ = iced::application(
        move || {
            window::Window::boot(server_info)
        },
        window::Window::update,
        window::Window::view
    )
        .run();

    // let mut server_lock = p2p_arc.lock().await;

    // let _ = server_lock.send(&server_info.into_bytes()).await;

    // loop {
    //     let response = server_lock.read().await.unwrap();
    //     let parsed = p2p::protocol::FromBytes::parse(&response);
    //     match parsed {
    //         FromBytes::MouseMove(message) => {
    //             mouse.move_mouse(message.x, message.y);
    //         }
    //         FromBytes::MouseClick(message) => {
    //             mouse.click_mouse(message.button, message.state);
    //         }
    //         _ => {}
    //     }
    // }
}