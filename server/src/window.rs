use std::sync::Arc;
use std::thread::JoinHandle;

use tokio::sync::Mutex;

use iced::Length::Fill;
use iced::Task;
use iced::widget::{space, button, column, container, row, text, text_input};

use p2p::protocol::{FromBytes, IntoBytes, ServerInformation};
use p2p::p2p::P2PError;

use crate::mouse::{self, Mouse};

pub struct Window {
    p2p: Arc<Mutex<Option<p2p::P2P>>>,
    server_info: ServerInformation,
    mouse_move_handle: Option<JoinHandle<i32>>,

    pin: Option<String>,
    key: Option<String>,
    
    wait_reason: String,
    error: String,
}

#[derive(Clone)]
pub enum Message {
    Register,
    AwaitClient(Result<(Arc<Mutex<Option<p2p::P2P>>>, String, String), Arc<P2PError>>),
    ConnectionEstablished(Result<(), String>),
    Disconnect,
    Drop,
    Null(())
}

impl Window {
    pub fn boot(server_information: ServerInformation) -> Self {
        let this = Self {
            server_info: server_information,
            p2p: Arc::new(Mutex::new(None)),
            mouse_move_handle: None,
            pin: None,
            key: None,
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
                    async move {
                        let (server, key) = p2p::P2P::init().await.map_err(|e| Arc::new(e))?;

                        let pin = p2p::remote_key_store::generate_key();
                        let key = key.to_string();
                        p2p::remote_key_store::set(&pin, &key.to_string()).await;

                        Ok((Arc::new(Mutex::new(Some(server))), key, pin))
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

                self.p2p = p2p;
                self.key = Some(key);
                self.pin = Some(pin);

                let arc = self.p2p.clone();
                
                self.wait_reason = "Waiting for connection".to_owned();
                Task::perform(
                async move {
                        let mut lock = arc.lock().await;
                        let p2p = lock.as_mut().unwrap();
                        let _ = p2p.await_connection().await;
                        Ok(())
                    },
                    Message::ConnectionEstablished,
                )
            }
            Message::ConnectionEstablished(result) => {
                if let Err(e) = result { eprintln!("send failed: {e}"); }
                self.wait_reason = "Connection Established".to_owned();

                let p2p_arc = self.p2p.clone();
                let server_info_bytes = self.server_info.into_bytes();

                let handle: JoinHandle<i32> = std::thread::spawn(move || {
                    tokio::runtime::Runtime::new().unwrap().block_on(async move { 
                        
                        let mut server_lock = p2p_arc.lock().await;
                        let server = server_lock.as_mut().unwrap();

                        let _ = server.send(&server_info_bytes).await;

                        drop(server_lock);

                        let mut mouse = mouse::EnigoMouse::new();
                        
                        loop {
                            let mut server_lock = p2p_arc.lock().await;
                            let server = server_lock.as_mut().unwrap();

                            let response = server.read().await.unwrap();
                            
                            drop(server_lock);
                            
                            let parsed = p2p::protocol::FromBytes::parse(&response);
                            match parsed {
                                FromBytes::MouseMove(message) => {
                                    mouse.move_mouse(message.x, message.y);
                                }
                                FromBytes::MouseClick(message) => {
                                    mouse.click_mouse(message.button, message.state);
                                }
                                _ => {}
                            }
                        }
                    })
                });

                self.mouse_move_handle = Some(handle);

                Task::none()
            },
            Message::Disconnect => {
                self.mouse_move_handle = None;
                self.pin = None;
                self.key = None;
                self.error = String::from("");
                self.wait_reason = String::from("");
                let p2p_arc = self.p2p.clone();

                self.p2p = Arc::new(Mutex::new(None));


                Task::perform(
                async move {
                        let mut lock = p2p_arc.lock().await;
                        if let Some(p2p) = lock.as_mut() {
                            let _ = p2p.close().await;
                        } else {
                            println!("Could not close connection");
                        }
                        
                        ()
                    },
                    Message::Null,
                )
            },
            Message::Drop => {
                Task::none()
            },
            Message::Null(_) => {
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        if self.mouse_move_handle.is_some() {
            return button("Disconnect").on_press(Message::Disconnect).into();
        }

        let pin = self.pin.clone().unwrap_or("None".to_owned());
        let key = self.key.clone().unwrap_or("None".to_owned());
        
        container (
            column![
                row![text("Pin"), space().width(24), text_input(&pin, &pin).on_input(|_| Message::Drop)],
                row![text("Key"), space().width(20), text_input(&key, &key).on_input(|_| Message::Drop)],
                space().height(20),
                // text(format!("Sys Info: {:?}", self.server_info)),
                container(button("Allow Connections").on_press(Message::Register)).center_x(Fill),
                text(&self.wait_reason),
                text(&self.error),
            ]
            .max_width(400)

        )
        .center_x(Fill)
        .center_y(Fill)
        .into()
    }
}


