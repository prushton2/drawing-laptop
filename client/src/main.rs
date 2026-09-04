use std::sync::Arc;
use tokio::sync::Mutex;

use iroh::EndpointId;

use p2p::{self, protocol::{self, IntoBytes, ServerInformation}};
use p2p::protocol::mouse_click::{MouseButton, MouseState};

mod window;

// #[tokio::main]
fn main() {
    let _ = iced::application(window::Window::boot, window::Window::update, window::Window::view)
        .subscription(window::subscription)
        .run();

}