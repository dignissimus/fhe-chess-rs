# A fully homomorphic chess engine
The chess engine acts on encrypted data representing the position on the board. The engine never sees your position.
# Usage
1. Generate the client and server keys with `cargo run --release --bin generate-keys`
2. Start playing chess! `cargo run --release --bin fhe-game`
