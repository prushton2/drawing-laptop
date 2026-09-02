use enigo::Mouse;
use enigo::{Button, Coordinate::Abs, Direction::Click, Enigo, Settings};

fn main() {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    
    enigo.move_mouse(1280, 800, Abs).unwrap();
    enigo.button(Button::Left, Click).unwrap();
}