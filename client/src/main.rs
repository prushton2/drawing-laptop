use std::sync::Arc;
use tokio::sync::Mutex;
use iced::{Length::Fill, Subscription, Task, widget::{self, container}, window};

use iroh::EndpointId;

use p2p::{self, protocol::{self, IntoBytes, ServerInformation}};
use p2p::protocol::mouse_click::{MouseButton, MouseState};

struct State {
    server_info: protocol::ServerInformation,
    known_size: (usize, usize),
    mouse_pos: (f32, f32),
    p2p: Arc<Mutex<p2p::P2P>>
}

#[derive(Clone)]
enum Message {
    WindowResized((usize, usize)),
    MoveMouse(f32, f32),
    ClickMouse(MouseButton, MouseState),
    Sent(Result<(), String>),
}

impl State {
    fn boot(p2p: Arc<Mutex<p2p::P2P>>, server_information: ServerInformation) -> Self {
        Self {
            server_info: server_information,
            known_size: (0, 0),
            mouse_pos: (0.0, 0.0),
            p2p: p2p
        }
    }

    fn update(&mut self, message: Message) -> Task<Message>{
        match message {
            Message::MoveMouse(x, y) => {
                self.mouse_pos = (x, y);

                let mouse_pct = (x / self.known_size.0 as f32, y / self.known_size.1 as f32);
                let scaled_mouse_pos = (mouse_pct.0 * self.server_info.width as f32, mouse_pct.1 * self.server_info.height as f32);

                let p2p = self.p2p.clone();
                let message = p2p::protocol::MouseMove {x: scaled_mouse_pos.0 as u32, y: scaled_mouse_pos.1 as u32};

                Task::perform(
                async move {
                        match p2p.lock().await.send(&message.into_bytes()).await {
                            Ok(_) => Ok(()),
                            Err(t) => Err(format!("{:?}", t))
                        }
                    },
                    Message::Sent,
                )
            }
            Message::ClickMouse(button, state) => {
                let p2p = self.p2p.clone();
                let message = p2p::protocol::MouseClick { button: button, state: state };

                Task::perform(
                async move {
                        match p2p.lock().await.send(&message.into_bytes()).await {
                            Ok(_) => Ok(()),
                            Err(t) => Err(format!("{:?} ", t))
                        }
                    },
                    Message::Sent,
                )
            },
            Message::WindowResized(size) => {
                self.known_size = size;
                Task::none()
            },
            Message::Sent(result) => {
                if let Err(e) = result { eprintln!("send failed: {e}"); }
                Task::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        container (
            widget::MouseArea::new(
                widget::row![
                    widget::text(format!("{}, {}", self.mouse_pos.0, self.mouse_pos.1))
                ]
                .width(Fill)
                .height(Fill)
            )
            .on_move(|point| {return Message::MoveMouse(point.x, point.y)})

            .on_press        (Message::ClickMouse(MouseButton::Left,  MouseState::Pressed ))
            .on_release      (Message::ClickMouse(MouseButton::Left,  MouseState::Released))
            .on_right_press  (Message::ClickMouse(MouseButton::Right, MouseState::Pressed ))
            .on_right_release(Message::ClickMouse(MouseButton::Right, MouseState::Released))

        )
        .width(Fill)
        .height(Fill)
        .into()
    }
}

fn subscription(_: &State) -> Subscription<Message> {
    window::resize_events().map(|(_id, size)| Message::WindowResized((size.width as usize, size.height as usize)))
}

#[tokio::main]
async fn main() {
    let key = std::env::args().nth(1).unwrap();
    let mut client = p2p::P2P::connect(key.parse::<EndpointId>().unwrap()).await.unwrap();
    let _ = client.send("test".as_bytes()).await;

    let data = client.read().await.unwrap();

    let server_info = match protocol::FromBytes::parse(&data) {
        protocol::FromBytes::ServerInformation(d) => d,
        _ => panic!("Did not receive server info")
    };
    
    let p2p: Arc<Mutex<p2p::P2P>> = Arc::new(Mutex::new(client));
    let moved_arc = p2p.clone();

    let _ = iced::application(
        move || {
            let arc: Arc<Mutex<p2p::P2P>> = moved_arc.clone();
            State::boot(arc, server_info)
        },
        State::update,
        State::view
    )
        .subscription(subscription)
        .run();

    let mut client = p2p.lock().await;
    client.close().await;
}