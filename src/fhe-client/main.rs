use chess::Board;
use fhe_chess_rs::common::ChessMessage::*;
use fhe_chess_rs::common::*;
use rayon::prelude::*;
use std::io::{self, BufRead};

use chess::{ChessMove, Color, MoveGen};
use concrete_shortint::ClientKey;
use std::collections::{HashMap, VecDeque};
use std::fs;
use tungstenite::client;
use tungstenite::Message::*;
use url::Url;

const MAX_DEPTH: u8 = 1;

fn multiplier(turn: chess::Color) -> i8 {
    match turn {
        Color::White => 1,
        Color::Black => -1,
    }
}

fn flip(turn: chess::Color) -> chess::Color {
    match turn {
        Color::White => Color::Black,
        Color::Black => Color::White,
    }
}

// Return the evaluation of the position and the best mvoe
fn minimax(
    depth: u8,
    turn: Color,
    candidates: &[u64],
    moves: &HashMap<u64, Vec<u64>>,
    evaluations: &HashMap<u64, i8>,
) -> (i8, u64) {
    if depth == 0 {
        // At the base, return the move with the highest ranking for the player to move
        // This returns the relative evaluation for the player
        candidates
            .iter()
            .map(|candidate| {
                (
                    *evaluations.get(candidate).unwrap() * multiplier(turn),
                    *candidate,
                )
            })
            .max()
            .unwrap()
    } else {
        let ((evaluation, _), candidate) = candidates
            .iter()
            .map(|candidate| (moves.get(candidate).unwrap(), candidate))
            .map(|(moveset, candidate)| {
                (
                    minimax(depth - 1, flip(turn), moveset, moves, evaluations),
                    candidate,
                )
            })
            .min()
            .unwrap();

        (-evaluation, *candidate)
    }
}

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

        // Generate a list of positions that need to be evaluated
        // These are indexed by their hash
        let mut positions: HashMap<u64, Board> = HashMap::new();

        // Create a hash map that will hold the evaluations of the positions
        // These are indexed by the hash of the position
        //  that the evaluation is for
        let mut evaluations: HashMap<u64, i8> = HashMap::new();

        // Represent the game tree as a graph with the positions as nodes
        // The moves in a position represent edges in this graph
        // The moves variable acts as an adjacency list
        let mut moves: HashMap<u64, Vec<u64>> = HashMap::new();

        // Treat creating the game tree as a graph traversal problem
        // Perform BFS starting from the root node
        // The queue variable stores the hashes of the positions currently being considered
        let mut queue: VecDeque<u64> = VecDeque::from([board.get_hash()]);

        // Add the root node
        let root = board.get_hash();
        positions.insert(root, board);

        let mut core_set: HashMap<u64, Board> = HashMap::new();

        for depth in 0..MAX_DEPTH {
            for _ in 0..queue.len() {
                let phash = queue.pop_front().unwrap();

                // Retrieve the representation of the board
                let board = positions[&phash];

                // Generate the legal moves
                let legal_moves = MoveGen::new_legal(&board);

                // Initialise the adjacency list
                let mut resulting_positions: Vec<u64> = Vec::new();

                for legal_move in legal_moves {
                    // Each legal move generates a position
                    // Find this position and store it
                    let board = board.make_move_new(legal_move);
                    resulting_positions.push(board.get_hash());

                    // If we have already come across this position then continue
                    if positions.contains_key(&board.get_hash()) {
                        continue;
                    }

                    positions.insert(board.get_hash(), board);
                    if depth == MAX_DEPTH {
                        core_set.insert(board.get_hash(), board);
                    } else {
                        // If we would like to explore this node, then add it to the queue
                        queue.push_back(board.get_hash());
                    }
                }
                moves.insert(board.get_hash(), resulting_positions);
            }
        }

        let npositions = positions.len();

        println!("Encoding positions...");
        let message = StreamFhePositions {
            positions: core_set
                .into_par_iter()
                .map(|(identifier, board)| {
                    (identifier, Position::from_board(board).to_fhe(&client_key))
                })
                .collect(),
        };

        println!("Sending position data to the server...");
        let serialised = bincode::serialize(&message).unwrap();
        websocket.write_message(Binary(serialised)).unwrap();
        let mut counter = npositions;

        while counter > 0 {
            let message = websocket.read_message().unwrap();
            counter -= 1;
            if let Binary(message) = message {
                let message: ChessMessage = bincode::deserialize(&message).unwrap();
                if let FheEvaluationResult {
                    evaluation,
                    identifier,
                } = message
                {
                    let evaluation = read_evaluation(&evaluation, &client_key);
                    println!("Progress {} / {}", npositions - counter, npositions);
                    evaluations.insert(identifier, evaluation);
                }
            }
        }

        let candidates = moves.get(&root).unwrap();
        let (evaluation, best_move) = minimax(
            MAX_DEPTH - 1,
            Color::Black,
            candidates,
            &moves,
            &evaluations,
        );
        for legal_move in MoveGen::new_legal(&board) {
            let new_board = board.make_move_new(legal_move);
            if new_board.get_hash() == best_move {
                println!(
                    "The server made move {} with evaluation {}",
                    legal_move, evaluation
                );
                board = new_board;
                break;
            }
        }
        println!("{}", board);
    }
}
