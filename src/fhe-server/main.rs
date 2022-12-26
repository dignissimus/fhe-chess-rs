use fhe_chess_rs::common::FhePackedInteger;
use fhe_chess_rs::common::*;
use std::fs;
use std::net::TcpListener;

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use tungstenite::protocol::WebSocketConfig;
use tungstenite::Message::*;

use concrete_shortint::ServerKey;

const N_CORES: u8 = 1;

fn main() {
    println!(
        "{:?}",
        serde_json::to_string(&ChessMessage::ReadEvaluations)
    );
    println!(
        "{:?}",
        serde_json::to_string(&ChessMessage::StreamPosition {
            identifier: 1,
            position: Position { data: Vec::new() }
        })
    );

    let weights = fs::read_to_string("weights.json").unwrap();
    let weights: Vec<i8> = serde_json::from_str(&weights).unwrap();
    let weights = Arc::new(weights);
    let server = TcpListener::bind("localhost:8085").unwrap();

    for stream in server.incoming() {
        println!("Loading the server key...");
        let server_key = fs::read("server-key.bin").unwrap();
        let server_key: ServerKey = bincode::deserialize(&server_key).unwrap();
        println!("Loaded the server key!");

        let stream = stream.unwrap();
        let weights = weights.clone();
        let mut config = WebSocketConfig::default();
        config.max_message_size = Some(usize::MAX);
        config.max_frame_size = Some(usize::MAX);
        thread::spawn(move || {
            let websocket = tungstenite::accept_with_config(stream, Some(config));

            let (transmitter, receiver) = mpsc::channel::<ChessMessage>();
            let mut workers: Vec<thread::JoinHandle<()>> = Vec::new();

            let queue: Arc<Mutex<Vec<(u64, FhePosition)>>> = Arc::new(Mutex::new(Vec::new()));

            for worker in 1..=N_CORES {
                println!("Initialising worker {}", worker);
                let transmitter = transmitter.clone();
                let queue = queue.clone();

                let weights = weights.clone();
                let server_key = server_key.clone();
                let zero = server_key.create_trivial(0);
                let handle = thread::spawn(move || {
                    loop {
                        // Attempt to find work inside the queue
                        let mut queue = queue.lock().unwrap();
                        let data = queue.pop();
                        drop(queue);

                        if data.is_none() {
                            thread::sleep(time::Duration::from_millis(1));
                            continue;
                        }

                        let (identifier, position) = data.unwrap();

                        let white_scores = weights
                            .iter()
                            .zip(position.data.iter())
                            .filter(|(weight, _)| **weight > 0)
                            .map(|(weight, bit)| (*weight as u8, bit));

                        let black_scores = weights
                            .iter()
                            .zip(position.data.iter())
                            .filter(|(weight, _)| **weight < 0)
                            .map(|(weight, bit)| (weight.unsigned_abs() as u8, bit));

                        let mut white_evaluation: FhePackedInteger = packed_zero(&zero);
                        let mut black_evaluation: FhePackedInteger = packed_zero(&zero);

                        for (weight, bit) in white_scores {
                            pack_multiply_add(&server_key, &mut white_evaluation, &weight, bit);
                        }

                        for (weight, bit) in black_scores {
                            pack_multiply_add(&server_key, &mut black_evaluation, &weight, bit);
                        }

                        transmitter
                            .send(ChessMessage::FheEvaluationResult {
                                identifier,
                                evaluation: (white_evaluation.clone(), black_evaluation.clone()),
                            })
                            .unwrap();
                    }
                });
                workers.push(handle);
            }

            println!("All workers created.");
            println!("Ready to receive positions!");

            match websocket {
                Err(msg) => {
                    println!("Error while accepting websocket connection: {}", msg);
                }

                Ok(websocket) => {
                    let counter: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
                    let websocket = Arc::new(Mutex::new(websocket));
                    let wsc = websocket.clone();

                    let cc = counter.clone();
                    thread::spawn(move || {
                        for evaluation in receiver.iter() {
                            let serialised = bincode::serialize(&evaluation).unwrap();
                            let mut wsc = wsc.lock().unwrap();
                            wsc.write_message(Binary(serialised)).unwrap();
                            let mut cx = cc.lock().unwrap();
                            *cx -= 1;
                            println!("{} positions left to go", cx);
                            drop(cx);
                        }
                    });
                    loop {
                        let mut websocket = websocket.lock().unwrap();
                        let message = websocket.read_message().unwrap();
                        drop(websocket);
                        if let Binary(serialised) = message {
                            let data: bincode::Result<ChessMessage> =
                                bincode::deserialize(&serialised);
                            if let Err(msg) = data {
                                println!("Malformed data: {}", msg);
                                continue;
                            }
                            println!("Verbose: {:?}", data);
                            let message = data.unwrap();
                            if let ChessMessage::StreamFhePositions { positions } = message {
                                println!("Received positions! Processing.");
                                let mut qx = queue.lock().unwrap();
                                let mut cx = counter.lock().unwrap();
                                *cx += positions.len() as u64;
                                drop(cx);
                                for position in positions {
                                    qx.push(position);
                                }
                                drop(qx);
                                loop {
                                    let counter = counter.lock().unwrap();
                                    if *counter == 0 {
                                        break;
                                    }
                                    drop(counter);
                                    thread::sleep(time::Duration::from_millis(1));
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}
