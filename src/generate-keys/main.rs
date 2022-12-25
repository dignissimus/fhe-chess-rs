use concrete::{generate_keys, ConfigBuilder};
use std::fs::File;
use std::io::Write;

fn main() {
    let config = ConfigBuilder::all_disabled().enable_default_uint8().build();
    println!("Generating encryption keys...");
    let (client_key, server_key) = generate_keys(config);

    println!("Writing encryption keys to disk");
    let mut file = File::create("client-key.bin").unwrap();
    file.write_all(&bincode::serialize(&client_key).unwrap())
        .expect("Unable to write client key to file");

    let mut file = File::create("server-key.bin").unwrap();
    file.write_all(&bincode::serialize(&server_key).unwrap())
        .expect("Unable to write server key to file");
}
