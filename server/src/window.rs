use tokio::sync::Mutex;
use std::sync::Arc;

use iced::Task;
use iced::widget::{button, column, container, text};

use p2p::protocol::ServerInformation;

pub struct Window {
    server_info: ServerInformation,
    pin: String,
    key: String,
    p2p: Arc<Mutex<p2p::P2P>>,
    waiting: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    StartWait,
    ConnectionEstablished(Result<(), String>),
}

impl Window {
    pub fn boot(server_information: ServerInformation, pin: &String, key: &String, p2p: Arc<Mutex<p2p::P2P>>) -> Self {
        let this = Self {
            server_info: server_information,
            pin: pin.clone(),
            key: key.clone(),
            p2p: p2p,
            waiting: false,
        };

        this
    }

    pub fn update(&mut self, message: Message) -> Task<Message>{
        match message {
            Message::StartWait => {
                self.waiting = true;
                let reference = self.p2p.clone();
                
                Task::perform(
                async move {
                        let mut lock = reference.lock().await;
                        let _ = lock.await_connection().await;
                        Ok(())
                    },
                    Message::ConnectionEstablished,
                )
            }
            Message::ConnectionEstablished(result) => {
                if let Err(e) = result { eprintln!("send failed: {e}"); }
                println!("established");
                iced::window::latest().and_then(iced::window::close)
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        column![
            text(format!("Pin: {}", self.pin)),
            text(format!("Key: {}", self.key)),
            text(format!("Sys Info: {:?}", self.server_info)),
            button("Wait for client").on_press(Message::StartWait),
            text(if self.waiting { "Waiting for client to connect..." } else { "" }),
        ]
        .spacing(10)
        .padding(20)
        .into()
    }
}