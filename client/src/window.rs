use std::sync::Arc;
use iroh::EndpointId;
use tokio::sync::Mutex;
use iced::{Length::Fill, Subscription, Task, window};
use iced::widget::{self, button, column, row, container, text, text_input};
use p2p::{self, protocol::{self, IntoBytes}};
use p2p::protocol::mouse_click::{MouseButton, MouseState};


#[derive(Default)]
pub struct Window {
    server_info: Option<protocol::ServerInformation>,
    p2p: Option<Arc<Mutex<p2p::P2P>>>,

    known_size: (usize, usize),
    
    pin_textbox: String,
    key_textbox: String,
}

#[derive(Clone)]
pub enum Message {
    MouseMove(f32, f32),
    MouseClick(MouseButton, MouseState),
    WindowResized((usize, usize)),

    PinSubmitted,
    P2PCreated((Arc<Mutex<p2p::P2P>>, protocol::ServerInformation)),
    
    PINTextbox(String),
    KeyTextbox(String),
    Sent(Result<(), String>),
}

impl Window {
    pub fn boot() -> Self {
        Self::default()
    }

    pub fn update(&mut self, message: Message) -> Task<Message>{
        match message {
            Message::MouseMove(x, y) => {
                let server_info = match self.server_info {
                    Some(t) => t,
                    None => return Task::none()
                };

                let p2p = match &self.p2p {
                    Some(t) => t.clone(),
                    None => return Task::none()
                };

                let mouse_pct = (x / self.known_size.0 as f32, y / self.known_size.1 as f32);
                let scaled_mouse_pos = (mouse_pct.0 * server_info.width as f32, mouse_pct.1 * server_info.height as f32);

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
            Message::MouseClick(button, state) => {
                let p2p = match &self.p2p {
                    Some(t) => t.clone(),
                    None => return Task::none()
                };

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
            Message::PinSubmitted => {
                let pin_textbox = self.pin_textbox.clone();
                let key_textbox = self.key_textbox.clone();
                Task::perform(
                    async move {
                        let key;
                        if key_textbox.len() == 0 {
                            key = p2p::remote_key_store::get(&pin_textbox).await;
                            p2p::remote_key_store::delete(&pin_textbox).await;
                        } else {
                            key = key_textbox;
                        }

                        let mut client = p2p::P2P::connect(key.parse::<EndpointId>().unwrap()).await.unwrap();
                        let _ = client.send("test".as_bytes()).await;

                        let data = client.read().await.unwrap();

                        let server_info = match protocol::FromBytes::parse(&data) {
                            protocol::FromBytes::ServerInformation(d) => d,
                            _ => panic!("Did not receive server info")
                        };
                        
                        let p2p: Arc<Mutex<p2p::P2P>> = Arc::new(Mutex::new(client));

                        (p2p, server_info)
                    },
                    Message::P2PCreated,
                )
            },

            Message::P2PCreated((p2p, server_info)) => {
                self.p2p = Some(p2p);
                self.server_info = Some(server_info);
                Task::none()
            },

            Message::WindowResized(size) => {
                self.known_size = size;
                Task::none()
            },
            Message::Sent(result) => {
                if let Err(e) = result { eprintln!("send failed: {e}"); }
                Task::none()
            },

            Message::PINTextbox(f) => {
                self.pin_textbox = f;
                Task::none()
            },
            Message::KeyTextbox(f) => {
                self.key_textbox = f;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        if self.p2p.is_none() {
            return container (
                column![
                    text("Input device PIN"),
                    row![text_input("000000", &self.pin_textbox).on_input(Message::PINTextbox), button("Confirm").on_press(Message::PinSubmitted)],
                    text("OR"),
                    text("Input device Key"),
                    row![text_input("", &self.key_textbox).on_input(Message::KeyTextbox), button("Confirm").on_press(Message::PinSubmitted)],
                    
                ]
            )
            .width(Fill)
            .height(Fill)
            .into()
        }

        return container (
            widget::MouseArea::new(
                widget::row![
                    // widget::text(format!("{}, {}", self.mouse_pos.0, self.mouse_pos.1))
                ]
                .width(Fill)
                .height(Fill)
            )
            .on_move(|point| {return Message::MouseMove(point.x, point.y)})

            .on_press        (Message::MouseClick(MouseButton::Left,  MouseState::Pressed ))
            .on_release      (Message::MouseClick(MouseButton::Left,  MouseState::Released))
            .on_right_press  (Message::MouseClick(MouseButton::Right, MouseState::Pressed ))
            .on_right_release(Message::MouseClick(MouseButton::Right, MouseState::Released))

        )
        .width(Fill)
        .height(Fill)
        .into()
    }
}

pub fn subscription(_: &Window) -> Subscription<Message> {
    window::resize_events().map(|(_id, size)| Message::WindowResized((size.width as usize, size.height as usize)))
}