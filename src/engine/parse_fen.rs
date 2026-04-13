//! # Module: `parse_fen`
//!
//! This module serves as the **Rosetta Stone** of the engine. While the core bitboards
//! handle the "machine math" and the move generator provides the "muscles,"
//! `parse_fen` bridges the gap between human-readable chess notation and the
//! engine's internal 64-bit architecture.
//!
//! ## Core Responsibilities
//!
//! ### 1. Board Decompression ([`parse_fen`])
//! FEN strings are naturally compressed (using digits like `8` to represent empty rows).
//! This function "unfolds" that string into a flattened, 64-character map.
//! * **The "FEN Paradox":** FEN starts at Rank 8 (top), while internal bitboards start
//!   at Rank 1 (index 0). This module handles the vertical flip using functional
//!   iterators and `.rev()`.
//!
//!
//!
//! ### 2. Board Compression ([`flat_board_to_fen`])
//! The inverse of the parser. It takes the engine's internal 64-character representation
//! and "folds" it back into a standard FEN string using **Run-Length Encoding** //! (e.g., `....` becomes `4`). This allows the engine to "talk" to external GUIs
//! like **En Croissant**.
//!
//! ### 3. Metadata Extraction ([`side_to_move`])
//! Chess is more than just piece positions. This function extracts the **Active Color** //! field from the FEN string, ensuring the engine knows exactly whose turn it is
//! before starting a search.
//!
//!
//!
//! ### 4. Surgical Updates ([`update_fen`])
//! A high-level utility for performing "surgical strikes" on a board state. By expanding
//! a FEN into a 2D grid, moving a piece, and re-compressing it, this function allows
//! for board manipulation without touching low-level bitboard logic.
//!
//! ---
//!
//! ## Implementation Philosophy
//! This module prioritizes **Declarative Logic**. By replacing nested `for` loops with
//! Rust’s functional pipelines (`.split()`, `.map()`, `.rev()`), the code remains
//! immutable, easy to test, and resistant to "off-by-one" string parsing errors.

use crate::engine::board::Color;

/// Transforms a FEN piece-placement string into a flattened 64-character board map.
///
/// This function expands FEN digits (representing empty squares) into periods (`.`)
/// and removes rank separators. It also reverses the rank order to align with
/// standard bitboard indexing (Rank 1 to Rank 8).
///
/// # Arguments
///
/// * `fen` - A full FEN string or just the piece-placement component.
///
/// # Returns
///
/// Returns a [`String`] of 64 characters where:
/// * Uppercase letters are White pieces.
/// * Lowercase letters are Black pieces.
/// * `.` represents an empty square.
///
/// # Implementation Details
///
/// Uses a functional approach with iterators:
/// 1. **Split:** Isolates the board part and individual ranks.
/// 2. **Map:** Converts digits like '3' into "..." via `to_digit` and `repeat`.
/// 3. **Reverse:** Uses `.rev()` because FEN starts at Rank 8, but internal
///    indexing typically starts at Rank 1 (index 0).
pub fn parse_fen(fen: &str) -> String {
    //replaced for loop with functional approach
    let ranks: Vec<String> = fen
        .split(' ')
        .next()
        .unwrap()
        .split('/')
        .map(|rank| {
            rank.chars()
                .map(|c| {
                    if let Some(count) = c.to_digit(10) {
                        ".".repeat(count as usize)
                    } else {
                        c.to_string()
                    }
                })
                .collect::<String>()
        })
        .rev()
        .collect();

    let board_representation = ranks.concat();
    board_representation
}

/// Converts a 64-character flat board string into a FEN piece-placement string.
///
/// This function compresses empty squares (represented by `.`) into digits and
/// inserts rank separators (`/`). It also reverses the rank order to ensure the
/// output follows the FEN standard (Rank 8 to Rank 1).
///
/// # Arguments
///
/// * `flat_board` - A 64-character string where each character represents a piece
///   or an empty square (`.`), ordered from Rank 1 to Rank 8.
///
/// # Returns
///
/// A [`String`] containing the compressed FEN representation of the board state.
///
/// # Implementation Details
///
/// * **Compression:** Iterates through the board, incrementing a counter for
///   consecutive empty squares and appending the count to the string when a
///   piece or rank boundary is encountered.
/// * **Rank Separation:** Inserts a `/` every 8 characters.
/// * **Coordinate Correction:** Splits the generated string by `/`, reverses
///   the resulting vector, and joins it back. This converts the internal
///   "bottom-up" storage (Rank 1-8) to the FEN "top-down" format (Rank 8-1).
pub fn flat_board_to_fen(flat_board: &str) -> String {
    let mut fen = String::new();
    let mut empty_count = 0;

    for (i, c) in flat_board.chars().enumerate() {
        if c == '.' {
            empty_count += 1;
        } else {
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
                empty_count = 0;
            }
            fen.push(c);
        }

        if (i + 1) % 8 == 0 {
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
                empty_count = 0;
            }
            if i != flat_board.len() - 1 {
                fen.push('/');
            }
        }
    }

    let mut ranks: Vec<&str> = fen.split('/').collect();
    ranks.reverse();
    ranks.join("/")
}

/// Parses a FEN string to determine which side is currently active (has the next move).
///
/// In the FEN standard, the second field (index 1) indicates the active color.
/// This function extracts that field and maps it to the internal [`Color`] enum.
///
/// # Arguments
///
/// * `fen` - A full FEN string (e.g., "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").
///
/// # Returns
///
/// * `Some(Color::White)` if the field is "w".
/// * `Some(Color::Black)` if the field is "b".
/// * `None` if the FEN is malformed or the color field is invalid.
///
/// # FEN Structure Note
///
/// A FEN string contains six space-separated fields. This function targets the second:
/// `[Piece Placement] [Side to Move] [Castling Rights] [En Passant] [Halfmove] [Fullmove]`
pub fn side_to_move(fen: &str) -> Option<Color> {
    let fields: Vec<&str> = fen.split_whitespace().collect();
    if fields.len() < 2 {
        return None; // invalid FEN
    }
    match fields[1] {
        "w" => Some(Color::White),
        "b" => Some(Color::Black),
        _ => None, // invalid value
    }
}

/// Updates the piece-placement portion of a FEN string based on a move.
///
/// This function performs a "lightweight" move update by expanding the FEN
/// into a 2D grid, moving the character from the source square to the
/// destination, and re-compressing the grid into FEN format.
///
/// # Arguments
///
/// * `from` - The starting UCI square (e.g., "e2").
/// * `to` - The destination UCI square (e.g., "e4").
/// * `fen` - The current full FEN string.
///
/// # Returns
///
/// Returns a [`String`] containing the updated **piece-placement** part of the FEN.
///
/// # Note
///
/// This function currently only updates the board layout. It does not update
/// side-to-move, castling rights, en passant squares, or move counters.
pub fn update_fen(from: &str, to: &str, fen: &str) -> String {
    let board_part = fen.split_whitespace().next().unwrap();

    let mut board: Vec<Vec<char>> = board_part
        .split('/')
        .map(|rank| {
            let mut row = Vec::new();
            for c in rank.chars() {
                if c.is_digit(10) {
                    for _ in 0..c.to_digit(10).unwrap() {
                        row.push('.');
                    }
                } else {
                    row.push(c);
                }
            }
            row
        })
        .collect();

    let square_to_idx = |sq: &str| -> (usize, usize) {
        let file = (sq.chars().nth(0).unwrap() as u8 - b'a') as usize;
        let rank = (sq.chars().nth(1).unwrap().to_digit(10).unwrap() - 1) as usize;
        (7 - rank, file)
    };

    let (from_row, from_col) = square_to_idx(from);
    let (to_row, to_col) = square_to_idx(to);

    let piece = board[from_row][from_col];
    board[from_row][from_col] = '.';
    board[to_row][to_col] = piece;

    let mut new_fen_parts = Vec::new();
    for row in board {
        let mut row_str = String::new();
        let mut empty_count = 0;
        for c in row {
            if c == '.' {
                empty_count += 1;
            } else {
                if empty_count > 0 {
                    row_str.push_str(&empty_count.to_string());
                    empty_count = 0;
                }
                row_str.push(c);
            }
        }
        if empty_count > 0 {
            row_str.push_str(&empty_count.to_string());
        }
        new_fen_parts.push(row_str);
    }

    new_fen_parts.join("/")
}
