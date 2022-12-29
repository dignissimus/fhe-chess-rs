# A fully homomorphic chess engine
The chess engine acts on encrypted data that represents the position on the board. The engine never sees the decrypted position.
# Usage
1. Generate the client and server keys with `cargo run --release --bin generate-keys`
2. Start playing chess! `cargo run --release --bin fhe-game`

# Technical details


# Performance
On my consumer laptop, the engine can evaluate positions at the beginning of the game at a depth of 3 within 1 minute and 30 seconds. On a server, this can be done in just 30 seconds and a depth-four analysis can be completed in about 5 to 10 minutes which is faster than I had anticipated.

Most of the time is spent encrypting the data, evaluating encrypted positions is quite fast.

# Strength

The engine's chess performance is varied. It will sometimes play strong openings but it will occasionally sacrifice its queen for no obvious reason. This could be due to the architecture of the chess engine and how it was trained. In essence, the chess engine is a giant piece square table trained using Machine Learning which isn't ideal. The engine was also trained on positions from the opening which doesn't give a good variety of positions to generalise on.

# Example position
The engine played black in this game
```
1. e4 b6 2. d4 Nf6 3. Nc3 a5 4. Bd3 c6 5. Be3 Qc7 6. Nf3 Qxh2
```
![image](https://user-images.githubusercontent.com/18627392/209901821-2a498884-7e2b-4f89-83cc-cda9dc003fa3.png)

After playing several games with the engine I noticed that it likes certain pieces being in specific squares which, given its stucture, is unsurprising. For example, the engine likes to have its queen on c7 and it often fianchettos the light-square Bishop. The engine likes the move Qxh2 a lot. It has probably learned that a black queen on h2 appears often in winning positions since a queen on h2 is common in certain checkmates (see below). However there is no intrinsic value to having a the black queen on h2 a black queen on h2 is winning in specific patterns such as the one below. Another potential issue could be the sample games that the engine was trained on. Bullet games formed the majority of the training data for the engine and there was no filter on the rating of players.

![image](https://user-images.githubusercontent.com/18627392/209902375-480f9bef-c8b4-444c-85e4-01ae5b02a599.png)
<center>An example where `Qxh2#` is winning</center>

# Reflection
I was able to speed up computation in FHE to even make it possible for the engine to play reasonable games but if I were to attempt this again, I would refine how I trained the model and potentially change the model architecture.
