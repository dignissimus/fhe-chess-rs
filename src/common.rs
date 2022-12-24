use chess::{Board, Color, File, Piece, Rank, Square};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug)]
pub enum ChessMessage {
    StreamPosition {
        identifier: u8,
        position: Position,
    },
    StreamPositions {
        positions: Vec<(u8, Position)>,
    },
    ReadEvaluations, // Remove me, only use the below
    EvaluationResult {
        identifier: u8,
        evaluation: Evaluation,
    },
}

type Evaluation = i8;

#[derive(Serialize, Deserialize, Debug)]
pub struct Move {
    pub positions: Vec<Position>,
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
}
