### Script to generate opening book
### Run ./.venv/bin/python ./scripts/make_book.py to create opening book
### The JSON file will be created in ./src/opening_book directory
### After that the engine will be able to look up first moves from there
### instead of evaluating position and choosing move on their own
### Games are downloaded from lichess database

import json
import requests
import zstandard as zstd
import chess.pgn
import chess.polyglot
import io
import os

DB_URL = "https://database.lichess.org/standard/lichess_db_standard_rated_2024-12.pgn.zst"
MIN_ELO = 2200
MAX_GAMES = 5000
BOOK_FILE = "./src/opening_book/book.json"

PIECE_SYMBOLS = {
    chess.PAWN: "P",
    chess.KNIGHT: "N",
    chess.BISHOP: "B",
    chess.ROOK: "R",
    chess.QUEEN: "Q",
    chess.KING: "K",
}

def build_book():
    print(f"Streaming from {DB_URL}...")
    session = requests.Session()
    response = session.get(DB_URL, stream=True)

    if response.status_code != 200:
        print(f"Failed to connect to Lichess Database: {response.status_code}")
        return

    dctx = zstd.ZstdDecompressor()
    # fen -> uci -> {count, piece}
    entries = {}
    games_found = 0

    with dctx.stream_reader(response.raw) as reader:
        text_stream = io.TextIOWrapper(reader, encoding="utf-8")
        try:
            while games_found < MAX_GAMES:
                game = chess.pgn.read_game(text_stream)
                if game is None:
                    break

                white_elo = int(game.headers.get("WhiteElo", 0))
                black_elo = int(game.headers.get("BlackElo", 0))

                if white_elo >= MIN_ELO or black_elo >= MIN_ELO:
                    board = game.board()
                    for move in game.mainline_moves():
                        if board.fullmove_number > 15:
                            break

                        fen = board.fen().split(" ")[0]  # position only, no clocks
                        uci = move.uci()
                        piece_type = board.piece_type_at(move.from_square)
                        piece_symbol = PIECE_SYMBOLS.get(piece_type, "P")

                        if fen not in entries:
                            entries[fen] = {}
                        if uci not in entries[fen]:
                            entries[fen][uci] = {"count": 0, "piece": piece_symbol}
                        entries[fen][uci]["count"] += 1

                        board.push(move)

                    games_found += 1
                    if games_found % 100 == 0:
                        print(f"Progress: {games_found}/{MAX_GAMES} games processed...")

        except Exception as e:
            print(f"Stream interrupted: {e}")

    # filter out moves seen only once
    filtered = {
        fen: {
            uci: data for uci, data in moves.items() if data["count"] >= 2
        }
        for fen, moves in entries.items()
    }
    filtered = {fen: moves for fen, moves in filtered.items() if moves}

    print(f"Saving {sum(len(m) for m in filtered.values())} moves to {BOOK_FILE}...")
    with open(BOOK_FILE, "w") as f:
        json.dump(filtered, f)

    print("Success. Opening book generated.")

if __name__ == "__main__":
    if os.path.exists(BOOK_FILE):
        print("Book already exists!")
    else:
        build_book()