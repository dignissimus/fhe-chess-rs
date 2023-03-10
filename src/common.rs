use chess::{Board, Color, File, Piece, Rank, Square};
use serde::{Deserialize, Serialize};

use concrete_shortint::{Ciphertext, ClientKey, ServerKey};
use std::fmt;

#[derive(Serialize, Deserialize)]
pub enum ServerState {
    AwaitingGame,
    AwaitingMove,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Position {
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct FhePosition {
    pub data: Vec<Ciphertext>,
}

#[derive(Serialize, Deserialize)]
pub struct PieceType {
    role: Role,
    colour: Colour,
}

#[derive(Serialize, Deserialize)]
pub enum Role {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Serialize, Deserialize)]
pub enum Colour {
    Black,
    White,
}

#[derive(Serialize, Deserialize)]
pub enum ChessMessage {
    StreamPosition {
        identifier: u8,
        position: Position,
    },
    StreamPositions {
        positions: Vec<(u8, Position)>,
    },
    StreamFhePositions {
        positions: Vec<(u64, FhePosition)>,
    },
    ReadEvaluations,
    EvaluationResult {
        identifier: u8,
        evaluation: Evaluation,
    },
    FheEvaluationResult {
        identifier: u64,
        evaluation: FheEvaluation,
    },
}

impl fmt::Debug for ChessMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("...").finish()
    }
}

type FheEvaluation = (FhePackedInteger, FhePackedInteger);
pub type FhePackedInteger = (Ciphertext, Ciphertext, Ciphertext, Ciphertext);
type Evaluation = i8;

#[derive(Serialize, Deserialize, Debug)]
pub struct Move {
    pub positions: Vec<Position>,
}

pub fn read_evaluation(evaluation: &FheEvaluation, client_key: &ClientKey) -> i8 {
    let (white, black) = evaluation;
    read_integer(white, client_key) - read_integer(black, client_key)
}

pub fn read_integer(packed: &FhePackedInteger, client_key: &ClientKey) -> i8 {
    let b0: u64 = client_key.decrypt(&packed.0);
    let b1: u64 = client_key.decrypt(&packed.1);
    let b2: u64 = client_key.decrypt(&packed.2);
    let b3: u64 = client_key.decrypt(&packed.3);

    (b0 + b1 + b2 + b3) as i8
}

pub fn packed_zero(zero: &Ciphertext) -> FhePackedInteger {
    (zero.clone(), zero.clone(), zero.clone(), zero.clone())
}

pub fn pack_multiply_add(
    server_key: &ServerKey,
    pack: &mut FhePackedInteger,
    multiplier: &u8,
    right: &Ciphertext,
) {
    match multiplier {
        1 => {
            server_key.unchecked_add_assign(&mut pack.0, right);
        }
        2 => {
            server_key.unchecked_add_assign(&mut pack.1, right);
        }
        3 => {
            server_key.unchecked_add_assign(&mut pack.1, right);
            server_key.unchecked_add_assign(&mut pack.0, right);
        }
        4 => {
            server_key.unchecked_add_assign(&mut pack.2, right);
        }
        5 => {
            server_key.unchecked_add_assign(&mut pack.0, right);
            server_key.unchecked_add_assign(&mut pack.2, right);
        }
        6 => {
            server_key.unchecked_add_assign(&mut pack.1, right);
            server_key.unchecked_add_assign(&mut pack.2, right);
        }
        7 => {
            server_key.unchecked_add_assign(&mut pack.0, right);
            server_key.unchecked_add_assign(&mut pack.1, right);
            server_key.unchecked_add_assign(&mut pack.2, right);
        }
        8 => {
            server_key.unchecked_add_assign(&mut pack.3, right);
        }

        9 => {
            server_key.unchecked_add_assign(&mut pack.3, right);
            server_key.unchecked_add_assign(&mut pack.0, right);
        }

        10 => {
            server_key.unchecked_add_assign(&mut pack.3, right);
            server_key.unchecked_add_assign(&mut pack.1, right);
        }

        11 => {
            server_key.unchecked_add_assign(&mut pack.3, right);
            server_key.unchecked_add_assign(&mut pack.1, right);
            server_key.unchecked_add_assign(&mut pack.0, right);
        }

        12 => {
            server_key.unchecked_add_assign(&mut pack.3, right);
            server_key.unchecked_add_assign(&mut pack.2, right);
        }

        13 => {
            server_key.unchecked_add_assign(&mut pack.3, right);
            server_key.unchecked_add_assign(&mut pack.2, right);
            server_key.unchecked_add_assign(&mut pack.0, right);
        }

        14 => {
            server_key.unchecked_add_assign(&mut pack.3, right);
            server_key.unchecked_add_assign(&mut pack.2, right);
            server_key.unchecked_add_assign(&mut pack.1, right);
        }

        15 => {
            server_key.unchecked_add_assign(&mut pack.3, right);
            server_key.unchecked_add_assign(&mut pack.2, right);
            server_key.unchecked_add_assign(&mut pack.1, right);
            server_key.unchecked_add_assign(&mut pack.0, right);
        }

        _ => panic!("Oh no"),
    }
}

fn bitboard_location(color: Color, piece: Piece, square: Square) -> usize {
    let piece_index = match piece {
        Piece::Pawn => 0,
        Piece::Knight => 1,
        Piece::Bishop => 2,
        Piece::Rook => 3,
        Piece::Queen => 4,
        Piece::King => 5,
    };
    let colour_index = match color {
        Color::Black => 0,
        Color::White => 1,
    };

    let square_index = square.get_rank().to_index() * 8 + square.get_file().to_index();

    colour_index * (64 * 6) + piece_index * 64 + square_index
}

impl Position {
    pub fn from_board(board: Board) -> Position {
        let mut serialised = vec![0; 64 * 12 + 1];
        for rank in 0..=7 {
            for file in 0..=7 {
                let rank = Rank::from_index(rank);
                let file = File::from_index(file);
                let square = Square::make_square(rank, file);
                if let Some(piece) = board.piece_on(square) {
                    let color = board.color_on(square).unwrap();
                    let location = bitboard_location(color, piece, square);
                    serialised[location] = 1;
                }
            }
        }
        serialised[64 * 12] = match board.side_to_move() {
            Color::Black => 0,
            Color::White => 1,
        };
        Position { data: serialised }
    }

    pub fn to_fhe(&self, client_key: &ClientKey) -> FhePosition {
        let zero = client_key.unchecked_encrypt(0);
        let one = client_key.unchecked_encrypt(1);
        FhePosition {
            data: self
                .data
                .iter()
                .map(|bit| if *bit == 0 { zero.clone() } else { one.clone() })
                .collect(),
        }
    }
}
