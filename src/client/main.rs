use chess::Board;
use fhe_chess_rs::common::ChessMessage::*;
use fhe_chess_rs::common::*;
use std::io::{self, BufRead};
use std::thread;
use std::time;
use tungstenite::client;
use tungstenite::Message::*;
use url::Url;

fn main() {
    let location = Url::parse("ws://0.0.0.0:8085").unwrap();
    let (mut websocket, _resposne) = client::connect(location).unwrap();
    let message = ReadEvaluations;
    let serialised = serde_json::to_string(&message).unwrap();
    websocket
        .write_message(Text(serialised))
        .expect("Error sending a message");
    println!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    for line in io::stdin().lock().lines() {
        // let board = Board::default();
        let board = Board::from_fen(line.unwrap());
        if let None = board {
            println!("Invalid FEN");
        }
        let board = board.unwrap();
        let position = Position::from_board(board);

        println!("Sending data");
        let message = StreamPosition {
            identifier: 0,
            position,
        };
        let serialised = serde_json::to_string(&message).unwrap();
        websocket.write_message(Text(serialised)).unwrap();
        println!("Waiting to receive message");
        let message = websocket.read_message().unwrap();
        println!("Received message");
        if let Text(message) = message {
            let message: ChessMessage = serde_json::from_str(&message).unwrap();
            println!("{:?}", message);
        }
    }
}
