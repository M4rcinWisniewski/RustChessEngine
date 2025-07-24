# RustMate ♟️

**RustMate** the procjet is in early development, only fen parsing and bitboards creation is made!


### Build and Run

```bash
git clone https://github.com/yourusername/rustmate.git
cd rustmate
cargo build --release
cargo run
```
## 📄 Example

Given a FEN string:

let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
let parsed = parse_fen::parse_fen(fen);
for (i, n) in parsed.chars().enumerate() {
    // map characters to bitboards
}

## 🧱 Project Structure

src/
├── main.rs          # Entry point
├── board.rs         # Bitboard logic and board display
└── parse_fen.rs     # FEN string parsing

## 🤝 Contributing

Pull requests and ideas are welcome! This is a learning-driven, open project — feel free to fork it and experiment.
## 📝 License

MIT © 2025 [Marcin Wiśniewski]