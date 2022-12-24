use fhe_chess_rs::common::*;
use std::fs;
use std::net::TcpListener;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use tungstenite::Message::*;

const N_CORES: u8 = 8;

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
    let server = TcpListener::bind("0.0.0.0:8085").unwrap();

    for stream in server.incoming() {
        let stream = stream.unwrap();
        let weights = weights.clone();

        // Handle a client connection
        thread::spawn(move || {
            let websocket = tungstenite::accept(stream);

            let (transmitter, receiver) = mpsc::channel::<ChessMessage>();
            let mut workers: Vec<thread::JoinHandle<()>> = Vec::new();

            let queue: Arc<Mutex<Vec<(u8, Position)>>> = Arc::new(Mutex::new(Vec::new()));

            for worker in 1..=N_CORES {
                println!("Initialising worker {}", worker);
                let transmitter = transmitter.clone();
                let queue = queue.clone();

                let weights = weights.clone();
                let handle = thread::spawn(move || {
                    loop {
                        // Attempt to find work inside the queue
                        let mut queue = queue.lock().unwrap();
                        let data = queue.pop();
                        drop(queue);

                        if let Some((identifier, position)) = data {
                            let evaluation = weights
                                .iter()
                                .zip(position.data.iter())
                                .map(|(weight, bit)| *weight * (*bit as i8))
                                .sum();
                            println!("{:?}", (identifier, evaluation));
                            transmitter
                                .send(ChessMessage::EvaluationResult {
                                    identifier,
                                    evaluation,
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
                            let serialised = serde_json::to_string(&evaluation).unwrap();
                            println!("Serialised");
                            let mut wsc = wsc.lock().unwrap();
                            println!("Worker thread has acquired lock!");
                            wsc.write_message(Text(serialised)).unwrap();
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
                        drop(websocket);
                        println!("Main thread has dropped lock!");
                        if let Text(json) = message {
                            let data = serde_json::from_str(&json);
                            if let Err(msg) = data {
                                println!("Malformed data: {}", msg);
                                continue;
                            }
                            println!("Verbose: {:?}", data);
                            let message = data.unwrap();
                            if let ChessMessage::StreamPositions { positions } = message {
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
