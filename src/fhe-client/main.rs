use chess::Board;
use fhe_chess_rs::common::ChessMessage::*;
use fhe_chess_rs::common::*;
use std::io::{self, BufRead};

use chess::{ChessMove, MoveGen};
use concrete_shortint::ClientKey;
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

    let mut board = Board::default();
    println!("Please enter a move");
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let result = ChessMove::from_san(&board, &line);
        if result.is_err() {
            println!("Invalid move");
            continue;
        }
        board = board.make_move_new(result.unwrap());
        let moves = MoveGen::new_legal(&board);
        let positions =
            moves.map(|m| Position::from_board(board.make_move_new(m)).to_fhe(&client_key));
        let moves = MoveGen::new_legal(&board);
        let move_map: HashMap<u8, ChessMove> = HashMap::from_iter((0..).zip(moves));

        println!("Encoding positions");
        let position_map: HashMap<u8, FhePosition> = HashMap::from_iter((0..).zip(positions));
        let mut counter = position_map.len();
        let mut evaluations = HashMap::<u8, i8>::new();
        let n_positions = position_map.len();

        println!("Sending position data to the server");
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
                    println!("Progress {} / {}", n_positions - counter, n_positions);
                    evaluations.entry(identifier).or_insert(evaluation);
                }
            }
        }

        let move_index = evaluations.iter().max_by_key(|entry| -entry.1).unwrap();
        println!("The server makes move {}", move_map[move_index.0]);
        board = board.make_move_new(move_map[move_index.0]);
        println!("{}", board);
    }
}
