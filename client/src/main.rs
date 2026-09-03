use iced::{Length::Fill, widget::{self, container}};

mod p2p;

#[derive(Default)]
struct State {
    mouse_pos: (f32, f32)
}

#[derive(Clone)]
enum Message {
    MoveMouse(f32, f32)
}

impl State {
    fn boot() -> Self {
        Self::default()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::MoveMouse(x, y) => {
                self.mouse_pos = (x, y);
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

fn main() {
    let _ = iced::application(State::boot, State::update, State::view)
        .run();
}