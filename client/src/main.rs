mod window;

// #[tokio::main]
fn main() {
    let _ = iced::application(window::Window::boot, window::Window::update, window::Window::view)
        .subscription(window::subscription)
        .run();

}