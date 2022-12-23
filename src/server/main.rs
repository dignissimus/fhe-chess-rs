use concrete::prelude::*;
use concrete::{generate_keys, set_server_key, ConfigBuilder, FheBool};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Result, Value};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use tungstenite;
use tungstenite::Message;
use tungstenite::Message::*;

const SIZE: usize = 561;
const N_THREADS: u8 = 8;

#[derive(Serialize, Deserialize)]
enum ServerState {
    AwaitingGame,
    AwaitingMove,
}

#[derive(Serialize, Deserialize, Debug)]
struct Position {
    bitboards: Map<String, Value>,
}

#[derive(Serialize, Deserialize)]
struct PieceType {
    role: Role,
    colour: Colour,
}

#[derive(Serialize, Deserialize)]
enum Role {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Serialize, Deserialize)]
enum Colour {
    Black,
    White,
}

#[derive(Serialize, Deserialize)]
struct BitBoard {
    data: Vec<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
enum ChessMessage {
    StreamMove { identifier: u8, movement: Move },
    ReadEvaluations,
}

#[derive(Serialize, Deserialize, Debug)]
struct Move {
    positions: Vec<Position>,
}

fn main() {
    println!(
        "{:?}",
        serde_json::to_string(&ChessMessage::ReadEvaluations)
    );
    println!(
        "{:?}",
        serde_json::to_string(&ChessMessage::StreamMove {
            identifier: 1,
            movement: Move {
                positions: Vec::new()
            }
        })
    );
    let server = TcpListener::bind("0.0.0.0:8085").unwrap();
    for stream in server.incoming() {
        let stream = stream.unwrap();
        thread::spawn(move || {
            let websocket = tungstenite::accept(stream);
            match websocket {
                Err(msg) => {
                    println!("Error while accepting websocket connection: {}", msg);
                }

                Ok(mut websocket) => loop {
                    let message = websocket.read_message().unwrap();
                    if let Text(json) = message {
                        let data: Result<ChessMessage> = serde_json::from_str(&json);
                        if let Err(msg) = data {
                            println!("Malformed data: {}", msg);
                            continue;
                        }
                        println!("Verbose: {:?}", data);
                        let message = data.unwrap();
                        match message {
                            ChessMessage::StreamMove { .. } => {}
                            ChessMessage::ReadEvaluations => {
                                let response = serde_json::to_string(&vec![1, 2, 3]).unwrap();
                                websocket.write_message(Text(response));
                            }
                        }
                    }
                },
            }
        });
    }
}

fn test_main() {
    let config = ConfigBuilder::all_disabled().enable_default_bool().build();
    let (client_key, server_key) = generate_keys(config);
    let queue: Arc<Mutex<Vec<(FheBool, FheBool)>>> = Arc::new(Mutex::new(Vec::new()));

    for _ in 1..SIZE {
        let a = FheBool::encrypt(false, &client_key);
        let b = FheBool::encrypt(true, &client_key);
        let mut queue = queue.lock().unwrap();
        queue.push((a, b));
        drop(queue);
    }

    let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();
    for _ in 1..N_THREADS {
        let queue = queue.clone();
        let key = server_key.clone();
        threads.push(thread::spawn(move || {
            set_server_key(key);
            loop {
                let mut queue = queue.lock().unwrap();
                let data = queue.pop();
                drop(queue);
                if let Some((left, right)) = data {
                    let _result = left & right;
                } else {
                    break;
                }
            }
        }));
    }

    let mut psize = 0;
    loop {
        let queue = queue.clone();
        let length = queue.lock().unwrap().len();
        drop(queue);
        if length == 0 {
            break;
        }
        if length != psize {
            println!("Queue size: {}", length);
            psize = length;
        }
        thread::sleep(std::time::Duration::from_secs(1));
    }
}
