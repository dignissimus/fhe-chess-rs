use concrete_shortint::gen_keys;
use concrete_shortint::parameters::PARAM_MESSAGE_7_CARRY_1;
use std::fs::File;
use std::io::Write;

fn main() {
    println!("Generating encryption keys...");
    let (client_key, server_key) = gen_keys(PARAM_MESSAGE_7_CARRY_1);

    println!("Writing encryption keys to disk");
    let mut file = File::create("client-key.bin").unwrap();
    file.write_all(&bincode::serialize(&client_key).unwrap())
        .expect("Unable to write client key to file");

    let mut file = File::create("server-key.bin").unwrap();
    file.write_all(&bincode::serialize(&server_key).unwrap())
        .expect("Unable to write server key to file");
}
