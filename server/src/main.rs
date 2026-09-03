// use enigo::Mouse;
// use enigo::{Button, Coordinate::Abs, Direction::Click, Enigo, Settings};
// use iroh::endpoint::presets::{self, Preset};
// use iroh::{Endpoint, EndpointAddr, PublicKey};

// #[tokio::main]
// async fn main() {
//     // let mut enigo = Enigo::new(&Settings::default()).unwrap();
    
//     // enigo.move_mouse(1280, 800, Abs).unwrap();
//     // enigo.button(Button::Left, Click).unwrap();

//     let addr: EndpointAddr = EndpointAddr::new(PublicKey::from_bytes(&[0; 32]).unwrap());
//     let ep = Endpoint::bind(presets::N0).await.unwrap();
//     let conn = ep.connect(addr, b"my-alpn").await.unwrap();
// }

use iroh::EndpointId;

mod p2p;


#[tokio::main]
async fn main() {
    match std::env::args().nth(1) {
        None => {
            let (mut server, key) = p2p::P2P::init().await;
            println!("Key: {}", key);
            server.await_connection().await;
            println!("Connection established");

            let message = server.read().await.unwrap();
            println!("< {:?}", message);

            let _ = server.send("123".as_bytes()).await;
            println!("Sent!");

            let message = server.read().await.unwrap();
            println!("< {:?}", message);

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            server.close().await;

        },
        Some(key) => {
            let mut server = p2p::P2P::connect(key.parse::<EndpointId>().unwrap()).await;
            println!("Connection established");

            let _ = server.send("123456".as_bytes()).await;
            println!("Sent!");

            let message = server.read().await.unwrap();
            println!("< {:?}", message);

            let _ = server.send("12345".as_bytes()).await;
            println!("Sent!");

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            server.close().await;
        }
    }
}