import struct
from collections import Counter
import requests
import zstandard as zstd
import chess.pgn
import chess.polyglot
import io
import sys
import os


DB_URL = "https://database.lichess.org/standard/lichess_db_standard_rated_2024-12.pgn.zst"
MIN_ELO = 2200
MAX_GAMES = 5000  
BOOK_FILE = "./src/opening_book/book.bin"

def build_book():
    print(f"Streaming from {DB_URL}...")

    session = requests.Session()
    response = session.get(DB_URL, stream=True)
    
    if response.status_code != 200:
        print(f"Failed to connect to Lichess Database: {response.status_code}")
        return

    dctx = zstd.ZstdDecompressor()
    entries = Counter()
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

                        key = chess.polyglot.zobrist_hash(board)
                        
                        
                        move_int = (move.to_square & 0x3F) | \
                                   ((move.from_square & 0x3F) << 6)
                        
                        if move.promotion:
                            promo_code = {chess.KNIGHT: 1, chess.BISHOP: 2, 
                                          chess.ROOK: 3, chess.QUEEN: 4}
                            move_int |= (promo_code.get(move.promotion, 0) << 12)

                        entries[(key, move_int)] += 1
                        board.push(move)
                    
                    games_found += 1
                    if games_found % 100 == 0:
                        print(f"Progress: {games_found}/{MAX_GAMES} games processed...")
        except Exception as e:
            print(f"Stream interrupted: {e}")

 
    print(f"Saving {len(entries)} moves to {BOOK_FILE}...")
    with open(BOOK_FILE, "wb") as f:

        for (key, move_int), weight in sorted(entries.items()):

            if weight < 2: 
                continue
                
            f.write(struct.pack(">QHHI", key, move_int, min(weight, 0xFFFF), 0))

    print("Success. Opening book generated.")

if __name__ == "__main__":
    if os.path.exists(BOOK_FILE):
        print("Book already exist!")
    else:
        build_book()