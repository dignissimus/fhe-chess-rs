use fhe_chess_rs::common::ChessMessage::*;
use fhe_chess_rs::common::*;
use std::thread;
use std::time;
use tungstenite::client;
use tungstenite::Message::*;
use url::Url;
use chess::Board;

fn main() {
    let location = Url::parse("ws://0.0.0.0:8085").unwrap();
    let (mut websocket, _resposne) = client::connect(location).unwrap();
    let message = ReadEvaluations;
    let serialised = serde_json::to_string(&message).unwrap();
    websocket
        .write_message(Text(serialised))
        .expect("Error sending a message");
    let board = Board::default();
    let position = Position::from_board(board);
    let message = StreamPosition {
        identifier: 0,
        position,
    };
    let serialised = serde_json::to_string(&message).unwrap();
    websocket.write_message(Text(serialised)).unwrap();
    loop {
        let message = websocket.read_message().unwrap();
        if let Text(message) = message {
            let message: Vec<i8> = serde_json::from_str(&message).unwrap();
            println!("{:?}", message);
        }
        thread::sleep(time::Duration::from_secs(1));
    }
}
