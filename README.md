# A fully homomorphic chess engine
The chess engine acts on encrypted data representing the position on the board. The engine never sees the decrypted board.
# Usage
1. Generate the client and server keys with `cargo run --release --bin generate-keys`
2. Start playing chess! `cargo run --release --bin fhe-game`

# Technical details
## Representation of integers
### Unsigned integers
In the model, weights vary between -15 and 15 and act on values of either 0 or 1 which will be multiplied and added together to produce the final result. In FHE, integers already have a representation which one can perform operations on. However, to minimise the number of multiplications I represent unsigned integers as a sequence of four integers. Each integer corresponds a coefficient of to a power of two. The number represented by this sequence is the sum of the coefficients multiplied by the respecitve power of two this allows multiply-add operations to be performed using no more than 3 additions.
### Signed integers
Signed integers are represented using two unsigned integers. The signed integer represented is their difference. This allows the linear evaluation function to be computed in two parts. The first part consisting of the the positive contribution to the final result and the second part consiconsisting of the negative contribution to the final result.
## Board representation
The board is serialised as a vector of 769 ones and zeros. Each number in the first 768 entries of the vector represent the presence of a particular piece type on a specific square. The final entry in the vector encodes which player will play the next move.
## Training procedure
Initially, I train the model using logistic regression with binary cross-entropy loss on positions from a large sample of games from the [Lichess open database](https://database.lichess.org/). The result is a model that learns to identify good positions for white and black and quantifies this with a score score between 0 and 1. I then use the weights from this model to build a linear position evaluation function which is inexpensive to compute. I use this evaluation function to build a chess engine which will play itself over a series of about ten iterations, with each iteration containing 100 games which terminate after either 40 or 80 plies, a total of 40,000 or 80,000 positions. After each iteration, the model weights are re-trained using logistic regression on the positions from the games in the previous iterations. The result is a strong model with a good positional understanding.

# Performance
On my consumer laptop, the engine can evaluate positions at the beginning of the game at a depth of 3 within 1 minute and 30 seconds. On a server, this can be done in just 30 seconds and a depth-four analysis can be completed in about 5 to 10 minutes which is faster than I had anticipated.

The game spends most of its time encrypting the data, evaluating encrypted positions is quite fast.

# Strength
The engine plays well. While playing against itself at a depth of 4, the engine achieves an average centipawn loss of 32, typical of rapid players with a rating of 1700. See example games here: https://lichess.org/study/bMp0VjXe/ooIYTqAS

# Notes and potential enhancements
* The model is strongest in the opening and early middle game. Due to its simple structure, it cannot strategise and when playing against itself it will often opt to draw by repetition. It is for these reasons that contemporary chess engines of this style interpolate between two sets of weights, one set of weights for the beginning of the game and another set of weights for the endgame. For this model, two sets of weights can be computed. One is initialised by training a model on positions from the first half of the game and the other is trained on positions from the second half of the game. During self-play, the engine can play with a higher temperature to encourage it to play games further into the endgame at the start. To maintain suitability for FHE, Interpolation can be performed using a clipped linear transformation on the number of moves that have been played in the game.
