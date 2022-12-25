use chess::Board;
use fhe_chess_rs::common::ChessMessage::*;
use fhe_chess_rs::common::*;
use std::io::{self, BufRead};

use concrete::ClientKey;
use std::fs;
use tungstenite::client;
use tungstenite::Message::*;
use url::Url;

fn main() {
    let location = Url::parse("ws://localhost:8085").unwrap();
    let (mut websocket, _resposne) = client::connect(location).unwrap();
    let message = ReadEvaluations;
    let serialised = serde_json::to_string(&message).unwrap();
    websocket
        .write_message(Text(serialised))
        .expect("Error sending a message");
    println!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    let client_key = fs::read("client-key.bin").expect("Unable to read client key");
    let client_key: ClientKey = bincode::deserialize(&client_key).unwrap();

    for line in io::stdin().lock().lines() {
        let board = Board::from_fen(line.unwrap());
        if board.is_none() {
            println!("Invalid FEN");
        }
        let board = board.unwrap();
        let position = Position::from_board(board).to_fhe(&client_key);

        println!("Sending data");
        let message = StreamFhePositions {
            positions: vec![(0, position)],
        };
        let serialised = bincode::serialize(&message).unwrap();
        websocket.write_message(Binary(serialised)).unwrap();
        println!("Waiting to receive message");
        let message = websocket.read_message().unwrap();
        println!("Received message");
        if let Binary(message) = message {
            let message: ChessMessage = bincode::deserialize(&message).unwrap();
            if let FheEvaluationResult { evaluation, .. } = message {
                println!("{:?}", read_evaluation(&evaluation, &client_key));
            }
        }
    }
}
