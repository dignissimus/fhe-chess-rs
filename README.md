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
## Training procedure
The model was initially trained using logistic regression with binary cross-entropy loss on positions from a large sample of games from the [Lichess open database](https://database.lichess.org/). The result is a model that learns to identify positions that are good for white and black and provides a score between 0 and 1. The weights from this model are then used to build a linear position evaluation function which is inexpensive to compute. This evaluation function is used to buuld a chess engine which will play itself over a series of about 10 iterations with each iteration containing 100 games which terminate after either 40 or 80 plies, a total of 40,000 or 80,000 positions. After each iteration, the model weights are re-trained using logistic regression on the positions from the games in the previous iterations. The result is a strong model with a good positional understanding.

# Performance
On my consumer laptop, the engine can evaluate positions at the beginning of the game at a depth of 3 within 1 minute and 30 seconds. On a server, this can be done in just 30 seconds and a depth-four analysis can be completed in about 5 to 10 minutes which is faster than I had anticipated.

Most of the time is spent encrypting the data, evaluating encrypted positions is quite fast.

# Strength
The engine plays well. While palying itself at a depth of 4, the engine achieves an average centipawn loss of 32, typical of rapid players with a rating of 1700. See example games here: https://lichess.org/study/bMp0VjXe/ooIYTqAS

# Notes and potential enhancements
* The model is strongest in the opening and early middle game. Due to its simplistic structure it cannot strategise and when playing against itself it will often opt to draw by repetition. It is for these reasons that contemprary chess engines of this style interpolate between two sets of weights, one set of weights for the beginning of the game and another set of weights for the endgame. For this model, two sets of weights can be computed. One initalised by training a model on positions from the first half of the game and the other trained on positions from the second half of the game. During self-play, the engine can play with a higher temperature to encourage it to play games further into the endgame at the start. To maintain suitability for FHE, Interpolation can be performed using a clipped linear transformation on the number of moves that have be played in the game.
