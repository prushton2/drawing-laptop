use std::sync::Arc;
use tokio::sync::Mutex;
use iced::{Length::Fill, Task, widget::{self, container}};

use iroh::EndpointId;

use p2p::{self, protocol, protocol::IntoBytes};

struct State {
    mouse_pos: (f32, f32),
    p2p: Arc<Mutex<p2p::P2P>>
}

#[derive(Clone)]
enum Message {
    MoveMouse(f32, f32),
    Sent(Result<(), String>),
}

impl State {
    fn boot(p2p: Arc<Mutex<p2p::P2P>>) -> Self {
        Self {
            mouse_pos: (0.0, 0.0),
            p2p: p2p
        }
    }

    fn update(&mut self, message: Message) -> Task<Message>{
        match message {
            Message::MoveMouse(x, y) => {
                self.mouse_pos = (x, y);

                let mut buf = (x as u32).to_be_bytes().to_vec();
                buf.extend_from_slice(&(y as u32).to_be_bytes());

                let p2p = self.p2p.clone();

                let message = p2p::protocol::MouseMove {x: x as u32, y: y as u32};

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
        )
        .width(Fill)
        .height(Fill)
        .into()
    }
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

    println!("Server info: {:?}", server_info);
    
    let p2p = Arc::new(Mutex::new(client));
    let moved_arc = p2p.clone();

    let _ = iced::application(
        move || {
            let arc: Arc<Mutex<p2p::P2P>> = moved_arc.clone();
            State::boot(arc)
        },
        State::update, 
        State::view
    )
        .run();

    let mut client = p2p.lock().await;
    client.close().await;
}