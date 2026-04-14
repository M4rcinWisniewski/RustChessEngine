#  Rust Chess Engine


A simple chess engine written in Rust, implementing a Negamax search algorithm.  
Currently supports move generation and evaluation, with a basic Negamax search. Alpha-beta pruning and more are planned for future improvements.

### Build and Run

Engine is now in the middle of major refactoring before its first release! Its not complete, building running is not recomended.

## 🧱 Project Structure

```bash
src/
├── main.rs          # Entry point
├── scripts/
│   └── make_book.py
├── engine/
│   ├── mod.rs
│   ├── parse_fen.rs      # FEN string parsing
│   ├── board.rs          # Bitboard logic and board display
│   ├── evaluation.rs     # Evaluation function
│   ├── game_over.rs      # Simple mating logic
│   ├── make_move.rs      # Legal move application
│   ├── movegen.rs        # Pseudo-legal move generation
│   └── search.rs         # Negamax search algorithm
│
└── opening_book/
    ├── mod.rs
    └── book.rs          # Loading and using the opening book database
```

## 🤝 Contributing

Pull requests and ideas are welcome! This is a learning-driven, open project — feel free to fork it and experiment.
## 📝 License

This project is licensed under the MIT License.  

© 2026 Marcin Wiśniewski
