use futures_util::{future, pin_mut, StreamExt, SinkExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use hex_literal::hex;

#[tokio::main]
async fn main() {
    let (mut socket, _response) = connect_async("ws://127.0.0.1:8080/").await.unwrap();
    let (mut write, read) = socket.split();

    let message = Message::Binary(hex!("deadbeef").to_vec());
    // socket.write_message(message.clone()).unwrap();
    // assert_eq!(socket.read_message().unwrap(), message);

    write.send(message).await.unwrap();

    // socket.close(None).await.unwrap();
}
