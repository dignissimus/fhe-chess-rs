use fhe_chess_rs::common::FhePackedInteger;
use fhe_chess_rs::common::*;
use std::fs;
use std::net::TcpListener;

use std::ops::Mul;

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use tungstenite::protocol::WebSocketConfig;
use tungstenite::Message::*;

use concrete::{set_server_key, ServerKey};

const N_CORES: u8 = 48;

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

    let server_key = fs::read("server-key.bin").unwrap();
    let server_key: ServerKey = bincode::deserialize(&server_key).unwrap();
    set_server_key(server_key.clone());

    for stream in server.incoming() {
        let stream = stream.unwrap();
        let weights = weights.clone();

        // Handle a client connection
        let server_key = server_key.clone();
        let mut config = WebSocketConfig::default();
        config.max_message_size = Some(usize::MAX);
        config.max_frame_size = Some(usize::MAX);
        thread::spawn(move || {
            let websocket = tungstenite::accept_with_config(stream, Some(config));

            let (transmitter, receiver) = mpsc::channel::<ChessMessage>();
            let mut workers: Vec<thread::JoinHandle<()>> = Vec::new();

            let queue: Arc<Mutex<Vec<(u8, FhePosition)>>> = Arc::new(Mutex::new(Vec::new()));

            for worker in 1..=N_CORES {
                println!("Initialising worker {}", worker);
                let transmitter = transmitter.clone();
                let queue = queue.clone();
                let server_key = server_key.clone();

                let weights = weights.clone();
                let handle = thread::spawn(move || {
                    set_server_key(server_key.clone());
                    loop {
                        // Attempt to find work inside the queue
                        let mut queue = queue.lock().unwrap();
                        let data = queue.pop();
                        drop(queue);

                        if let Some((identifier, position)) = data {
                            let zero = position.data.get(0).unwrap().mul(0u8);

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

                            let mut index = 1;
                            for (weight, bit) in white_scores {
                                pack_multiply_add(&mut white_evaluation, &weight, bit);
                                println!("{}", index);
                                index += 1;
                            }

                            let mut index = 1;
                            for (weight, bit) in black_scores {
                                pack_multiply_add(&mut black_evaluation, &weight, bit);
                                println!("{}", index);
                                index += 1;
                            }

                            transmitter
                                .send(ChessMessage::FheEvaluationResult {
                                    identifier,
                                    evaluation: (
                                        white_evaluation.clone(),
                                        black_evaluation.clone(),
                                    ),
                                })
                                .unwrap();
                        }

                        // Rest
                        thread::sleep(time::Duration::from_millis(1));
                    }
                });
                workers.push(handle);
            }

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
                            println!("Received!");
                            let serialised = bincode::serialize(&evaluation).unwrap();
                            println!("Serialised");
                            let mut wsc = wsc.lock().unwrap();
                            println!("Worker thread has acquired lock!");
                            wsc.write_message(Binary(serialised)).unwrap();
                            println!("Sent message!");
                            let mut cx = cc.lock().unwrap();
                            *cx -= 1;
                            drop(cx);
                        }
                    });
                    loop {
                        println!("Main thread has acquired lock");
                        let mut websocket = websocket.lock().unwrap();
                        let message = websocket.read_message().unwrap();
                        println!("Read message!");
                        drop(websocket);
                        println!("Main thread has dropped lock!");
                        if let Binary(serialised) = message {
                            println!("Received a binary message");
                            let data: bincode::Result<ChessMessage> =
                                bincode::deserialize(&serialised);
                            if let Err(msg) = data {
                                println!("Malformed data: {}", msg);
                                continue;
                            }
                            println!("Verbose: {:?}", data);
                            let message = data.unwrap();
                            if let ChessMessage::StreamFhePositions { positions } = message {
                                println!("Read a FHE position, adding to queue");
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
