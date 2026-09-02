use enigo::Mouse;
use enigo::{Button, Coordinate::Abs, Direction::Click, Enigo, Settings};
use iroh::endpoint::presets::{self, Preset};
use iroh::{Endpoint, EndpointAddr, PublicKey};

#[tokio::main]
async fn main() {
    // let mut enigo = Enigo::new(&Settings::default()).unwrap();
    
    // enigo.move_mouse(1280, 800, Abs).unwrap();
    // enigo.button(Button::Left, Click).unwrap();

    let addr: EndpointAddr = EndpointAddr::new(PublicKey::from_bytes(&[0; 32]).unwrap());
    let ep = Endpoint::bind(presets::N0).await.unwrap();
    let conn = ep.connect(addr, b"my-alpn").await.unwrap();
}