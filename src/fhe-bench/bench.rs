use concrete::prelude::*;
use concrete::{generate_keys, set_server_key, ClientKey, ConfigBuilder, FheUint8, ServerKey};
use std::fs;
use std::ops::Mul;
use std::time::Instant;

fn bench(a: &FheUint8, b: &FheUint8) {
    let _result = a + b;
}

fn main() {
    /*
    println!("Generating keys...");
    let start = Instant::now();
    let config = ConfigBuilder::all_disabled().enable_default_uint8().build();
    let (client_key, server_key) = generate_keys(config);
    println!("Took {:?} to generate server keys", start.elapsed());
    */

    let client_key = fs::read("client-key.bin").unwrap();
    let server_key = fs::read("server-key.bin").unwrap();
    let client_key: ClientKey = bincode::deserialize(&client_key).unwrap();
    let server_key: ServerKey = bincode::deserialize(&server_key).unwrap();
    set_server_key(server_key);

    println!("Encrypting values...");
    let clear_a = 27u8;
    let clear_b = 128u8;
    let a = FheUint8::encrypt(clear_a, &client_key);
    let b = FheUint8::encrypt(clear_b, &client_key);

    println!("Measuring...");
    let start = Instant::now();
    for _ in 1..=10 {
        let _result = &a.clone().mul(1u8) + &b.clone().mul(2u8);
    }
    println!(
        "Took {:?} to perform 10 additions on 8-bit integers",
        start.elapsed()
    );
}
