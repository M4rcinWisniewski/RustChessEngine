use crate::board::PieceType;
use crate::movegen::Move;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct MoveEntry {
    pub count: u32,
    pub piece: String, // We'll convert this to PieceType
}

fn piece_from_str(s: &str) -> PieceType {
    match s {
        "P" | "p" => PieceType::Pawn,
        "N" | "n" => PieceType::Knight,
        "B" | "b" => PieceType::Bishop,
        "R" | "r" => PieceType::Rook,
        "Q" | "q" => PieceType::Queen,
        "K" | "k" => PieceType::King,
        _ => panic!("Invalid piece symbol: {}", s),
    }
}

// I know there is probably better way, but i like the simplest
// This is used for calculating length of kings move (either one normal move, or two, castle)
fn absolute_subtruct(a: u8, b: u8) -> u8 {
    if a > b {
        return a - b;
    }
    b - a
}

// Select a move from the opening book randomly, weighted by count
pub fn get_uci_move(
    book: &HashMap<String, HashMap<String, MoveEntry>>,
    fen: &str,
) -> Option<String> {
    if let Some(moves) = book.get(fen) {
        let total: u32 = moves.values().map(|entry| entry.count).sum();
        let mut rng = rand::rng();
        let mut r = rand::Rng::random_range(&mut rng, 0..total);

        for (mv, entry) in moves {
            if r < entry.count {
                return Some(mv.clone());
            }
            r -= entry.count;
        }
    }
    None
}

// Convert UCI string from opening book into a Move struct
pub fn opening(book: &HashMap<String, HashMap<String, MoveEntry>>, fen: &str) -> Option<Move> {
    let uci = get_uci_move(book, fen)?;

    assert!(uci.len() >= 4, "UCI move string too short");
    let (from_uci, to_uci) = uci.split_at(2);

    let from_file = (from_uci.chars().nth(0).unwrap() as u8) - b'a';
    let from_rank = (from_uci.chars().nth(1).unwrap() as u8) - b'1';
    let to_file = (to_uci.chars().nth(0).unwrap() as u8) - b'a';
    let to_rank = (to_uci.chars().nth(1).unwrap() as u8) - b'1';

    let from = from_rank * 8 + from_file;
    let to = to_rank * 8 + to_file;

    // Get the piece type directly from the MoveEntry
    let piece_entry = &book.get(fen).unwrap()[&uci];
    let piece = piece_from_str(&piece_entry.piece);
    let mut castling: bool = false;
    if piece == PieceType::King && absolute_subtruct(from, to) == 2 {
        castling = true;
    }
    Some(Move {
        from,
        to,
        piece,
        promotion_rights: uci.len() == 5, // if promotion indicated, e.g. "e7e8q"
        is_castling: castling,            // can implement castling detection if needed
        is_capture: false,                // can implement capture detection if needed
    })
}
