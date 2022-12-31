# A fully homomorphic chess engine
The chess engine acts on encrypted data that represents the position on the board. The engine never sees the decrypted position.
# Usage
1. Generate the client and server keys with `cargo run --release --bin generate-keys`
2. Start playing chess! `cargo run --release --bin fhe-game`

# Technical details
## Representation of integers
## Multi-threading
## Board representation
## Model architecture
## Training method

# Performance
On my consumer laptop, the engine can evaluate positions at the beginning of the game at a depth of 3 within 1 minute and 30 seconds. On a server, this can be done in just 30 seconds and a depth-four analysis can be completed in about 5 to 10 minutes which is faster than I had anticipated.

Most of the time is spent encrypting the data, evaluating encrypted positions is quite fast.

# Strength
After re-training the model through self-play, the engine plays well. See example games here: https://lichess.org/study/bMp0VjXe/ooIYTqAS
