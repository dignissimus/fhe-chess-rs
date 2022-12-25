use chess::Board;
use fhe_chess_rs::common::ChessMessage::*;
use fhe_chess_rs::common::*;
use std::io::{self, BufRead};

use chess::{ChessMove, MoveGen};
use concrete::ClientKey;
use std::collections::HashMap;
use std::fs;
use tungstenite::client;
use tungstenite::Message::*;
use url::Url;

fn main() {
    let location = Url::parse("ws://localhost:8085").unwrap();
    let (mut websocket, _response) = client::connect(location).unwrap();
    println!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    let client_key = fs::read("client-key.bin").expect("Unable to read client key");
    let client_key: ClientKey = bincode::deserialize(&client_key).unwrap();

    let board = Board::default();
    for line in io::stdin().lock().lines() {
        println!("Line");
        let line = line.unwrap();
        let board = board.make_move_new(ChessMove::from_san(&board, &line).unwrap());
        println!("made move");
        let moves = MoveGen::new_legal(&board);
        println!("Gen");
        let positions =
            moves.map(|m| Position::from_board(board.make_move_new(m)).to_fhe(&client_key));
        println!("Pos");
        let moves = MoveGen::new_legal(&board);
        println!("More gen");
        println!("Gen move map");
        let move_map: HashMap<u8, ChessMove> = HashMap::from_iter((0..).zip(moves));
        println!("Gen pos map");
        let position_map: HashMap<u8, FhePosition> = HashMap::from_iter((0..).zip(positions));
        println!("Gen counter");
        let mut counter = position_map.len();
        println!("Eval map");
        let mut evaluations = HashMap::<u8, i8>::new();

        println!("Sending data");
        let message = StreamFhePositions {
            positions: position_map.into_iter().collect(),
        };
        let serialised = bincode::serialize(&message).unwrap();
        websocket.write_message(Binary(serialised)).unwrap();

        println!("Waiting to receive messages");
        while counter > 0 {
            let message = websocket.read_message().unwrap();
            counter -= 1;
            println!("Received message");
            if let Binary(message) = message {
                let message: ChessMessage = bincode::deserialize(&message).unwrap();
                if let FheEvaluationResult {
                    evaluation,
                    identifier,
                } = message
                {
                    let evaluation = read_evaluation(&evaluation, &client_key);
                    println!("Identifier {:?} {:?}", identifier, evaluation);
                    println!("Counter {:?}", counter);
                    evaluations.entry(identifier).or_insert(evaluation);
                }
            }
        }

        let move_index = evaluations.iter().max_by_key(|entry| -entry.1).unwrap();
        println!("The server makes move {}", move_map[move_index.0]);
        let board = board.make_move_new(move_map[move_index.0]);
        println!("{}", board);
    }
}
