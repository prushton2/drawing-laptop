use tokio::sync::Mutex;
use std::sync::Arc;

use p2p::protocol::FromBytes;
use p2p::{self, protocol, protocol::IntoBytes};

use crate::mouse::Mouse;

mod mouse;
mod window;

fn main() {
    // init mouse controller
    // let mut mouse = mouse::enigo::EnigoMouse::new();

    let p2p_handle = Arc::new(Mutex::new(None));
    let p2p_clone = p2p_handle.clone();

    let server_info = protocol::ServerInformation {
        width: 1920,
        height: 1080
    };

    let _ = iced::application(
        move || {
            window::Window::boot(server_info, p2p_clone.clone())
        },
        window::Window::update,
        window::Window::view
    )
        .run();   
}