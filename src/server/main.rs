use fhe_chess_rs::common::*;
use std::fs;
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use tungstenite::Message::*;

fn main() {
    println!(
        "{:?}",
        serde_json::to_string(&ChessMessage::ReadEvaluations)
    );
    println!(
        "{:?}",
        serde_json::to_string(&ChessMessage::StreamPosition {
            identifier: 1,
            position: Position {
                data: Vec::new()
            }
        })
    );

    let weights = fs::read_to_string("weights.json").unwrap();
    let _weights: Vec<i8> = serde_json::from_str(&weights).unwrap();
    let server = TcpListener::bind("0.0.0.0:8085").unwrap();

    for stream in server.incoming() {
        let stream = stream.unwrap();
        thread::spawn(move || {
            let websocket = tungstenite::accept(stream);
            let (_transmitter, receiver) = mpsc::channel::<ChessMessage>();
            match websocket {
                Err(msg) => {
                    println!("Error while accepting websocket connection: {}", msg);
                }

                Ok(mut websocket) => loop {
                    let message = websocket.read_message().unwrap();
                    if let Text(json) = message {
                        let data = serde_json::from_str(&json);
                        if let Err(msg) = data {
                            println!("Malformed data: {}", msg);
                            continue;
                        }
                        println!("Verbose: {:?}", data);
                        let message = data.unwrap();
                        match message {
                            ChessMessage::StreamPosition { .. } => {
                                let evaluations: Vec<ChessMessage> = receiver.try_iter().collect();
                                let response = serde_json::to_string(&evaluations).unwrap();
                                websocket
                                    .write_message(Text(response))
                                    .expect("Error sending the message");
                            }
                            ChessMessage::ReadEvaluations => {
                                let response = serde_json::to_string(&vec![1, 2, 3]).unwrap();
                                websocket
                                    .write_message(Text(response))
                                    .expect("Error sending the message");
                            }

                            _ => {}
                        }
                    }
                },
            }
        });
    }
}
