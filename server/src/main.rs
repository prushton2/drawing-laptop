use iced::Theme;

use p2p::protocol;

mod mouse;
mod window;

fn main() {
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
        .theme(Theme::CatppuccinFrappe)
        .run();   
}