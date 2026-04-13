//! # Module: `make_move`
//!
//! This module serves as the **Physics Engine** and **Rules Arbiter** of the chess engine.
//! While the move generator suggests potential "pseudo-legal" actions, `make_move` is
//! responsible for executing those moves, updating the global state, and ultimately
//! filtering out illegal positions.
//!
//! ## Core Responsibilities
//!
//! ### 1. Threat Detection ([`is_square_attacked`])
//! Acts as the engine's "Security System." It determines if a specific square is
//! currently under fire by the opponent.
//! * **Efficiency:** It utilizes a high-speed King-distance heuristic (Chebyshev distance)
//!   before falling back to standard move generation for other piece types.
//! * **Logic:** $distance = \max(|sq_{rank} - king_{rank}|, |sq_{file} - king_{file}|)$.
//!
//! ### 2. State Mutation ([`apply_move`])
//! The primary state machine. It surgically alters bitboards to reflect a move's
//! impact on the board.
//! * **Atomic Actions:** Handles piece displacement, regular captures, and resets
//!   ephemeral states like the en passant square.
//! * **Special Rules:** Contains the complex logic for **En Passant** (removing
//!   the ghost pawn), **Pawn Promotion** (currently defaulting to Queen), and
//!   **Castling** (synchronizing the King and Rook movement).
//! * **Persistent State:** Updates castling rights whenever a King or Rook moves
//!   (or is captured), ensuring rules are strictly followed throughout the game.
//!
//! ### 3. Legality Filtering ([`generate_legal_moves`])
//! The bridge between "maybe" and "yes." It converts pseudo-legal moves into
//! strictly legal ones.
//! * **The Trial-and-Error Method:** For every generated move, the engine
//!   clones the board, applies the move, and checks if the King is left in a state
//!   of check.
//! * **The Filter:** If the King is safe, the move is validated and added to the
//!   final move list.
//!
//! ---
//!
//! ## Implementation Philosophy
//!
//! > **Safety over Speed:** This module currently uses a "Clone-and-Apply" strategy.
//! > While this is computationally more expensive than an "Undo/Unmake" system,
//! > it prevents subtle state-corruption bugs that frequently plague complex
//! > chess engines.
//!
//! * **Bitwise Masking:** All updates are performed using 64-bit masks, allowing
//!   for $O(1)$ updates to piece positions.
//! * **Defensive Programming:** Includes validation checks to ensure squares
//!   remain within the 0-63 range, preventing runtime panics during deep searches.

use crate::board::{Bitboards, Color, PieceType};

use crate::engine::board;
use crate::movegen::Move;

/// Determines if a specific square is under attack by a given side.
///
/// This is a fundamental utility for move validation, particularly for:
/// * **Check Detection:** Is the King's square attacked?
/// * **Castling Legality:** Are the squares the King passes through attacked?
/// * **Illegal Moves:** Does a move leave the King in check?
///
/// # Arguments
///
/// * `board` - A reference to the current [`Bitboards`] state.
/// * `sq` - The bit index (0-63) of the square to verify.
/// * `color` - The color of the side *being attacked* (the function will
///   check if the *opposite* color is attacking this square).
///
/// # Returns
///
/// Returns `true` if any enemy piece can legally (pseudo-legally) capture
/// on the target square.
///
/// # Implementation Details
///
/// 1. **Enemy Identification:** Swaps the input `color` to find the attacking side.
/// 2. **Bitboard Iteration:** Loops through all enemy piece types (excluding the King).
///    For each piece found via bit-scanning (`trailing_zeros`), it generates
///    pseudo-legal moves and checks if the `to` square matches `sq`.
/// 3. **King Heuristic:** Since using a full move generator for the King can be
///    heavy, it uses a manual distance check. If the enemy King is exactly one
///    square away (calculated via Chebyshev distance), the square is attacked.
pub fn is_square_attacked(board: &Bitboards, sq: u8, color: Color) -> bool {
    let enemy_color = match color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };
    let piece_types: [PieceType; 5] = [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
    ];

    for &piece_type in &piece_types {
        let bitboard = board.boards[enemy_color as usize][piece_type as usize];
        let mut pieces = bitboard;
        while pieces != 0 {
            let from = pieces.trailing_zeros() as u8;
            pieces &= pieces - 1;
            let moves = Move::generate_moves_for_piece(from, piece_type, enemy_color, board);
            for mv in moves {
                if mv.to == sq {
                    return true;
                }
            }
        }
    }

    // Checking kings seperatly to make castling logic work
    let king_bitboard = board.boards[enemy_color as usize][PieceType::King as usize];
    if king_bitboard != 0 {
        let king_sq = king_bitboard.trailing_zeros() as u8;
        let distance = (sq as i8 - king_sq as i8)
            .abs()
            .max((sq % 8) as i8 - (king_sq % 8) as i8)
            .abs();
        if distance == 1 {
            // King attacks 1 square away
            return true;
        }
    }

    false
}

/// Validates that a square index falls within the legal 0-63 range.
///
/// This is a low-level safety check used primarily by [`apply_move`] to prevent
/// out-of-bounds indexing before performing bitwise shifts.
///
/// # Arguments
///
/// * `sq` - The bit index to check.
///
/// # Returns
///
/// Returns `true` if the square is between 0 and 63 (inclusive),
/// `false` otherwise.
///
/// # Why This Matters
///
/// Since bitboards are `u64` integers, shifting a 1 by more than 63 bits
/// results in undefined behavior or a panic in Rust (in debug mode). This
/// function ensures the engine remains "panic-proof" during high-speed searches.
fn is_valid_square(sq: u8) -> bool {
    sq < 64
}

/// Checks if the King of the specified color is currently in check.
///
/// This function acts as the high-level "Checkmate/Stalemate" precursor. It
/// locates the King on the board and queries the [`is_square_attacked`]
/// function to see if the opponent has a direct line of fire to that square.
///
/// # Arguments
///
/// * `board` - A reference to the current [`Bitboards`] state.
/// * `color` - The side whose King we are checking (e.g., `Color::White`
///   checks if the White King is in check).
///
/// # Returns
///
/// Returns `true` if the King is under attack, `false` otherwise. If no
/// King is found on the board (an invalid state in standard chess),
/// it returns `false`.
///
/// # Implementation Details
///
/// * **King Retrieval:** Accesses the King bitboard using the color index (0 for White,
///   1 for Black) and the King piece index (5).
/// * **Square Mapping:** Converts the bitboard into a vector of indices. Since
///   there is only one King, it extracts the first element as the target square.
/// * **Attack Verification:** delegates the heavy lifting to `is_square_attacked`.
pub fn is_check(board: &Bitboards, color: Color) -> bool {
    if color == Color::White {
        let king_squares = board::Bitboards::return_squares(board.boards[0][5]);
        if king_squares.is_empty() {
            return false;
        }
        let king = king_squares[0];
        return is_square_attacked(board, king, color);
    } else {
        let king_squares = board::Bitboards::return_squares(board.boards[1][5]);
        if king_squares.is_empty() {
            return false;
        }
        let king = king_squares[0];
        return is_square_attacked(board, king, color);
    }
}

/// Executes a move on the provided bitboard state, updating all relevant board metadata.
///
/// This function is the core "state mutator" of the engine. It transforms a pseudo-legal
/// [`Move`] into a concrete change in the board's bitboards. It is designed to be
/// atomic; if the move involves multiple steps (like castling), they are all
/// handled within this single call.
///
/// # Arguments
///
/// * `board` - A mutable reference to the [`Bitboards`] state.
/// * `mv` - The [`Move`] struct containing source, destination, and metadata.
/// * `color` - The color of the side currently making the move.
///
/// # Special Move Handling
///
/// * **En Passant:** Detects moves into the `en_passant_square` and removes the
///   "ghost" pawn from the rank behind/ahead of the destination.
/// * **Castling:** Moves the corresponding Rook to its new position based on
///   hardcoded starting and destination squares.
/// * **Promotion:** Currently transforms any pawn reaching the final rank into
///   a Queen (promotion to other pieces is a future enhancement).
/// * **State Updates:** Automatically invalidates castling rights if a King or Rook
///   is moved or captured and calculates the next potential en passant square.
pub fn apply_move(board: &mut Bitboards, mv: &Move, color: Color) {
    let from_mask = 1u64 << mv.from;
    let to_mask = 1u64 << mv.to;

    if !is_valid_square(mv.from) || !is_valid_square(mv.to) {
        panic!("Invalid move: {:?}", mv);
    }
    // opponents color
    let enemy_color = match color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };

    // Handle en passant capture
    let mut is_en_passant = false;
    if mv.piece == PieceType::Pawn {
        if let Some(ep_sq) = board.en_passant_square {
            if mv.to == ep_sq {
                is_en_passant = true;
                let captured_pawn_sq = if color == Color::White {
                    ep_sq - 8
                } else {
                    ep_sq + 8
                };
                let captured_mask = 1u64 << captured_pawn_sq;
                board.boards[enemy_color as usize][PieceType::Pawn as usize] &= !captured_mask;
            }
        }
    }

    // Reset en passant square
    board.en_passant_square = None;

    // Remove piece from source square
    board.boards[color as usize][mv.piece as usize] &= !from_mask;

    // Handle regular captures
    if !is_en_passant {
        for piece_type in 0..6 {
            if board.boards[enemy_color as usize][piece_type] & to_mask != 0 {
                board.boards[enemy_color as usize][piece_type] &= !to_mask;
                break;
            }
        }
    }

    // Place piece at destination (handle promotion)
    if mv.promotion_rights {
        board.boards[color as usize][PieceType::Queen as usize] |= to_mask; // For simplicity only promotes to queen for now
    } else {
        board.boards[color as usize][mv.piece as usize] |= to_mask;
    }

    // Check if this pawn move creates a new en passant opportunity
    if mv.piece == PieceType::Pawn {
        let rank_diff = (mv.to as i8 - mv.from as i8).abs();
        if rank_diff == 16 {
            // Two-square pawn move
            board.en_passant_square = Some(if color == Color::White {
                mv.from + 8
            } else {
                mv.from - 8
            });
        }
    }

    // Castling
    if mv.is_castling {
        match (mv.from, mv.to) {
            (4, 6) => {
                // White kingside: e1→g1
                board.boards[Color::White as usize][PieceType::Rook as usize] &= !(1u64 << 7); // Remove rook from h1
                board.boards[Color::White as usize][PieceType::Rook as usize] |= 1u64 << 5; // Place rook on f1
            }
            (4, 2) => {
                // White queenside: e1→c1
                board.boards[Color::White as usize][PieceType::Rook as usize] &= !(1u64 << 0); // Remove rook from a1
                board.boards[Color::White as usize][PieceType::Rook as usize] |= 1u64 << 3; // Place rook on d1
            }
            (60, 62) => {
                // Black kingside: e8→g8
                board.boards[Color::Black as usize][PieceType::Rook as usize] &= !(1u64 << 63); // Remove rook from h8
                board.boards[Color::Black as usize][PieceType::Rook as usize] |= 1u64 << 61; // Place rook on f8
            }
            (60, 58) => {
                // Black queenside: e8→c8
                board.boards[Color::Black as usize][PieceType::Rook as usize] &= !(1u64 << 56); // Remove rook from a8
                board.boards[Color::Black as usize][PieceType::Rook as usize] |= 1u64 << 59; // Place rook on d8
            }
            _ => {}
        }
    }
    //update castle rights when king moves
    if mv.piece == PieceType::King {
        if color == Color::White {
            board.white_kingside = false;
            board.white_queenside = false;
        } else {
            board.black_kingside = false;
            board.black_queenside = false
        }
    }
    // update castle rights when rook moves/is captured
    if mv.from == 0 || mv.to == 0 {
        board.white_queenside = false;
    }
    if mv.from == 7 || mv.to == 7 {
        board.white_kingside = false;
    }
    if mv.from == 56 || mv.to == 56 {
        board.black_queenside = false;
    }
    if mv.from == 63 || mv.to == 63 {
        board.black_kingside = false;
    }
}

/// Filters pseudo-legal moves to return a vector of strictly legal moves.
///
/// In chess, a move is only legal if it does not leave the player's own King
/// in check. This function performs the final "verification" by simulating
/// every potential move on a temporary board state.
///
/// # Arguments
///
/// * `board` - A reference to the current [`Bitboards`] state.
/// * `color` - The side for which to generate moves.
///
/// # Returns
///
/// A [`Vec<Move>`] containing only the moves that comply with the laws of chess.
///
/// # Implementation Strategy: Search-and-Verify
///
/// This function acts as the final arbiter using a three-step pipeline:
/// 1. **Generation:** Obtains all pseudo-legal moves (moves that match piece
///    movement rules but may ignore existing or resulting checks).
/// 2. **Simulation:** For each move, it creates a `clone` of the current board
///    and executes the move using [`apply_move`].
/// 3. **Validation:** Calls [`is_check`] on the resulting board state. If the
///    moving side's King is safe, the move is pushed to the `legal_moves` vector.
pub fn generate_legal_moves(board: &Bitboards, color: Color) -> Vec<Move> {
    let mut legal_moves = Vec::new();

    for mv in Move::generate_moves_for_side(color, board) {
        let mut clone = board.clone();
        apply_move(&mut clone, &mv, color);
        if !is_check(&clone, color) {
            legal_moves.push(mv);
        }
    }

    legal_moves
}
