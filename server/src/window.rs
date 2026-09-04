use p2p::p2p::P2PError;
use tokio::sync::Mutex;
use std::sync::Arc;

use iced::Task;
use iced::widget::{button, column, text, text_input};

use p2p::protocol::ServerInformation;

pub struct Window {
    server_info: ServerInformation,
    pin: Option<String>,
    key: Option<String>,
    p2p: Option<Arc<Mutex<p2p::P2P>>>,
    wait_reason: String,
    error: String,
}

#[derive(Clone)]
pub enum Message {
    Register,
    AwaitClient(Result<(Arc<Mutex<p2p::P2P>>, String, String), Arc<P2PError>>),
    ConnectionEstablished(Result<(), String>),
    Drop
}

impl Window {
    pub fn boot(server_information: ServerInformation) -> Self {
        let this = Self {
            server_info: server_information,
            pin: None,
            key: None,
            p2p: None,
            wait_reason: String::from(""),
            error: String::from("")
        };

        this
    }

    pub fn update(&mut self, message: Message) -> Task<Message>{
        match message {
            Message::Register => {
                self.wait_reason = "Registering...".to_owned();

                Task::perform(
                    async {
                        let (server, key) = p2p::P2P::init().await.map_err(|e| Arc::new(e))?;

                        let pin = p2p::remote_key_store::generate_key();
                        let key = key.to_string();
                        p2p::remote_key_store::set(&pin, &key.to_string()).await;

                        Ok((Arc::new(Mutex::new(server)), key, pin))
                    },
                    Message::AwaitClient
                )
            },
            Message::AwaitClient(result) => {
                let (p2p, key, pin) = match result {
                    Ok(t) => t,
                    Err(t) => {
                        self.error = format!("{:?}", t);
                        self.wait_reason = String::from("");
                        return Task::none()
                    }
                };

                self.p2p = Some(p2p);
                self.key = Some(key);
                self.pin = Some(pin);

                let arc = self.p2p.as_mut().unwrap();
                let reference = arc.clone();
                
                self.wait_reason = "Waiting for connection".to_owned();
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
            },
            Message::Drop => {
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let pin = format!("Pin: {:?}", self.pin);
        let key = format!("Key: {:?}", self.key);
        column![
            text(&self.error),
            text_input(&pin, &pin).on_input(|_| Message::Drop),
            text_input(&key, &key).on_input(|_| Message::Drop),
            text(format!("Sys Info: {:?}", self.server_info)),
            button("Allow Connections").on_press(Message::Register),
            text(&self.wait_reason),
            text(&self.error),
        ]
        .spacing(10)
        .padding(20)
        .into()
    }
}