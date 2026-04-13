//! # Module: `movegen`
//!
//! This module is the engine's "engine room." It calculates every pseudolegal 
//! move for a given position using **Bitboard-based** calculations.
//!
//! ## Performance Goals
//! - **Throughput:** Aiming for 50M+ nodes per second in Perft testing.
//!
//! ## Correctness
//! - Must pass all standard **Perft** test suites (standard, kiwipete, etc.).

use crate::board::{
    PieceType,
    Color,
    Bitboards
};
use crate::make_move;

/// Bitmask representing all squares on the A-file.
/// Used to prevent pieces from "wrapping around" to the H-file when shifting left.
const FILE_A: u64 = 0x0101010101010101;

/// Bitmask representing all squares on the B-file.
/// Used primarily for Knight move generation to prevent two-file wrap-around bugs.
const FILE_B: u64 = 0x0202020202020202;

/// Bitmask representing all squares on the G-file.
const FILE_G: u64 = 0x4040404040404040;

/// Bitmask representing all squares on the H-file.
/// Used to prevent pieces from wrapping to the A-file when shifting right.
const FILE_H: u64 = 0x8080808080808080;

/// Represents a single chess move with all necessary metadata for 
/// making/unmaking and move ordering.
#[derive(Debug, Clone)]
pub struct Move {
    /// The starting square index (0-63).
    pub from: u8,
    
    /// The destination square index (0-63).
    pub to: u8,
    
    /// The type of piece that is performing the move.
    pub piece: PieceType,
    
    /// Indicates if this move results in a pawn promotion.
    pub promotion_rights: bool,
    
    /// Set to `true` if the move is a King-side or Queen-side castle.
    pub is_castling: bool,
    
    /// Set to `true` if the move involves capturing an opponent's piece.
    pub is_capture: bool,
}



impl Move {
    /// Generates a vector of all pseudo-legal moves for a specific piece on a given square.
    ///
    /// # Arguments
    ///
    /// * `sq` - The 0-63 index of the square where the piece is located.
    /// * `piece` - The type of piece being moved.
    /// * `color` - The color of the piece.
    /// * `boards` - A reference to the current game state bitboards.
    ///
    /// # Returns
    ///
    /// Returns a [`Vec<Move>`] containing all possible moves, including captures.
    ///
    /// # Panics
    ///
    /// Panics if `sq` is greater than or equal to 64.
    pub fn generate_moves_for_piece(sq: u8, piece: PieceType, color: Color, boards: &Bitboards) -> Vec<Move> {
        assert!(sq < 64, "Invalid square index: {}", sq);
        match piece {
            PieceType::Pawn => Self::pawn_moves(sq, color, boards),
            PieceType::Knight => Self::knight_moves(sq, color, boards),
            PieceType::King => Self::king_moves(sq, color, boards),
            PieceType::Rook => Self::rook_moves(sq, color, boards),
            PieceType::Bishop => Self::bishop_moves(sq, color, boards),
            PieceType::Queen => Self::queen_moves(sq, color, boards),

        }
    }
    /// Assigns numerical values to every piece
    ///
    /// # Arguments
    ///
    /// * `piece` - The type of piece to evaluate
    ///
    /// # Returns
    ///
    /// Returns an [`i32`] representing the value of the piece in **centipawns**
    fn get_piece_value(piece: PieceType) -> i32 {
        match piece {
            PieceType::Pawn => 100,
            PieceType::Knight => 300,
            PieceType::Bishop => 300,
            PieceType::Rook => 500,
            PieceType::Queen => 900,
            PieceType::King => 10000,
        }
    }


    /// Identifies which opponent piece occupies a square and returns its value.
    ///
    /// This is used during move ordering to prioritize captures of high-value pieces
    ///
    /// # Arguments
    ///
    /// * `boards` - The current bitboard state.
    /// * `square` - The target square index (0-63).
    /// * `opponent_color` - The color of the piece being captured.
    ///
    /// # Returns
    ///
    /// Returns the [`i32`] value of the captured piece. Returns `0` if the square is empty.
    ///
    /// # Performance Note
    ///
    /// Iterates through piece types from most valuable (Queen) to least valuable (Pawn)
    /// to find the occupant efficiently.
    fn get_captured_piece_value(boards: &Bitboards, square: u8, opponent_color: Color) -> i32 {
        // Look at the destination square and see what opponent piece is there
        for piece_type in [PieceType::Queen, PieceType::Rook, PieceType::Bishop, 
                        PieceType::Knight, PieceType::Pawn] {
            let piece_bb = boards.boards[opponent_color as usize][piece_type as usize];
            if (piece_bb >> square) & 1 == 1 {
                return Self::get_piece_value(piece_type);
            }
        }
        0
    }


    /// Generates all pseudo-legal moves for the given color and sorts them by heuristic strength.
    ///
    /// This function iterates through all bitboards for the active side, generates individual
    /// piece moves, and applies a scoring heuristic to assist with move ordering in the search tree.
    ///
    /// # Arguments
    ///
    /// * `color` - The side to generate moves for.
    /// * `boards` - The current game state.
    ///
    /// # Returns
    ///
    /// Returns a [`Vec<Move>`] sorted in descending order of strength (e.g., captures and 
    /// promotions first).
    ///
    /// # Heuristics Used
    ///
    /// * **MVV-LVA:** Prioritizes "Most Valuable Victim - Least Valuable Attacker" captures.
    /// * **Promotion:** Prioritizes moves that result in a piece promotion.
    /// * **Center Control:** Rewards moves that target the central squares (d4, d5, e4, e5).
    pub fn generate_moves_for_side(color: Color, boards: &Bitboards) -> Vec<Move> {
        let mut moves = Vec::new();
        let oposite_color = match color {
            Color::White => Color::Black,
            Color::Black => Color::White
        };
        for piece in [
            PieceType::Pawn,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Queen,
            PieceType::King,
        ] {
            let mut bb = boards.boards[color as usize][piece as usize];
            while bb != 0 {
                let sq = bb.trailing_zeros() as u8;
                bb &= bb - 1; // clear least significant bit

                moves.extend(Self::generate_moves_for_piece(sq, piece, color, boards));
            }
        }

        moves.sort_by_key(|m| {
                let mut score = 0;
                
                if m.is_capture {
                    // Try to figure out what piece was captured at m.to
                    let victim_value = Self::get_captured_piece_value(boards, m.to, oposite_color);
                    let attacker_value = Self::get_piece_value(m.piece);
                    score += victim_value * 10 - attacker_value; // Prefer QxP over PxQ
                }
                
                if m.promotion_rights {
                    score += 800;
                }
                
                // Prefer center squares
                if [27, 28, 35, 36].contains(&m.to) { // e4, e5, d4, d5
                    score += 30;
                }
                
                -score // Sort descending
            });
            
            moves

    }

    
    /// Checks if a specific bit is set in a bitboard, relative to a starting square.
    ///
    /// This helper handles the coordinate math and boundary checking to ensure 
    /// that offsets (e.g., +8 for North) don't result in out-of-bounds bit shifts.
    ///
    /// # Arguments
    ///
    /// * `bitboard` - The 64-bit representation of piece locations to check.
    /// * `from` - The base square index (0-63).
    /// * `offset` - The relative move distance (can be negative).
    ///
    /// # Returns
    ///
    /// Returns `true` if the target square is within the 0-63 range and the 
    /// corresponding bit is set to 1. Returns `false` otherwise.
    fn is_square_occupied(bitboard: u64, from: u8, offset: i8) -> bool {
        let target = from as i16 + offset as i16;
        if (0..64).contains(&target) {
            (bitboard & (1u64 << target)) != 0
        } else {
            false
        }
    }
    /// Aggregates all individual piece bitboards for a given color into a single bitboard.
    ///
    /// This provides a "global occupancy" mask for one side, which is essential for 
    /// collision detection and determining if a square is occupied by a friendly piece.
    ///
    /// # Arguments
    ///
    /// * `bitboards` - A reference to the current game state.
    /// * `color` - The color of the pieces to aggregate.
    ///
    /// # Returns
    ///
    /// A [`u64`] where each set bit represents the presence of any piece of the specified color.
    fn get_own_pieces(bitboards: &Bitboards, color: Color) -> u64 {
        bitboards.boards[color as usize]
            .iter()
            .fold(0u64, |acc, &bb| acc | bb)
    }


    /// Aggregates all individual piece bitboards for the opponent's color.
    ///
    /// This is a convenience wrapper around [`Self::get_own_pieces`] that automatically 
    /// calculates the opposing side's occupancy mask. It is primarily used to 
    /// identify potential capture targets.
    ///
    /// # Arguments
    ///
    /// * `bitboards` - A reference to the current game state.
    /// * `color` - The color of the *current* player (the function finds the opposite).
    ///
    /// # Returns
    ///
    /// A [`u64`] representing all squares occupied by the opponent.
    fn get_opponent_pieces(bitboards: &Bitboards, color: Color) -> u64 {
        let opponent = match color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        bitboards.boards[opponent as usize]
            .iter()
            .fold(0u64, |acc, &bb| acc | bb)
    }

    
    /// Generates all pseudo-legal moves for a Knight at a given square.
    ///
    /// This function uses bit-shifting to calculate the 8 possible "L-shapes" a knight 
    /// can move in. It employs specific file masks (A, B, G, H) to prevent the knight 
    /// from wrapping around the edges of the board.
    ///
    /// # Arguments
    ///
    /// * `sq` - The 0-63 index of the knight's current position.
    /// * `color` - The color of the knight.
    /// * `board` - A reference to the current game state.
    ///
    /// # Returns
    ///
    /// Returns a [`Vec<Move>`] of all squares the knight can legally move to (excluding 
    /// squares occupied by friendly pieces).
    ///
    /// # Mathematical Logic
    ///
    /// The offsets represent the "L" jumps on a 64-slot bitboard:
    /// * **17/15:** Vertical-heavy jumps (Up 2, Left/Right 1).
    /// * **10/6:** Horizontal-heavy jumps (Up 1, Left/Right 2).
    ///
    /// Each jump is masked by the destination file to ensure logical consistency.
    fn knight_moves(sq: u8, color: Color, board: &Bitboards) -> Vec<Move> {
        let knight= 1u64 << sq;
        let mut moves = 0u64;
        let own_pieces = Self::get_own_pieces(board, color);
        let opponent_pieces = Self::get_opponent_pieces(board, color);

        if sq <= 46 && !Self::is_square_occupied(own_pieces, sq, 17) {
            
            moves |= (knight & !FILE_H) << 17;
        }
        if sq <= 48 && !Self::is_square_occupied(own_pieces, sq,15) {

            moves |= (knight & !(FILE_A))             << 15; // ↑2 ←1
        }
        if sq <= 53 && !Self::is_square_occupied(own_pieces, sq,10) {

            moves |= (knight & !(FILE_G | FILE_H))    << 10; // ↑1 →2
        }
        if sq <= 57 && !Self::is_square_occupied(own_pieces, sq, 6) {

            moves |= (knight & !(FILE_A | FILE_B))    << 6;  // ↑1 ←2
        }
        if sq >= 17 && !Self::is_square_occupied(own_pieces, sq,-17) {

            moves |= (knight & !(FILE_A)) >> 17; // ↓2 ←1;
        }
        if sq >= 15 && !Self::is_square_occupied(own_pieces, sq,-15) {

            moves |= (knight & !(FILE_H))             >> 15; // ↓2 →1
        }
        if sq >= 10 && !Self::is_square_occupied(own_pieces, sq,-10) {

            moves |= (knight & !(FILE_A | FILE_B))    >> 10; // ↓1 ←2
        }
        if sq >= 6 && !Self::is_square_occupied(own_pieces, sq, -6) {

            moves |= (knight & !(FILE_G | FILE_H))    >> 6;  // ↓1 →2
        }
        Self::moves_from_bitboard(sq, PieceType::Knight, moves, false, false, opponent_pieces)
    }



    /// Validates and generates legal Castling moves for both sides.
    ///
    /// Castling is a complex move with multiple requirements:
    /// 1. The King and chosen Rook must not have moved (Castling Rights).
    /// 2. The path between the King and Rook must be empty.
    /// 3. The King must not be in check, and must not pass through or land on a square 
    ///    attacked by an opponent piece.
    ///
    /// # Arguments
    /// * `king_sq` - The current square of the King.
    /// * `rook_sq` - The square of the Rook involved in the check (used for validation).
    /// * `color` - The side attempting to castle.
    /// * `board` - The current game state, including FEN castling flags.
    ///
    /// # Returns
    /// A [`Vec<Move>`] containing the King's destination square if castling is legal.
    ///
    /// # TO DO
    /// Refactor this function to improve its **readability** 
    fn castling(king_sq: u8, rook_sq: u8, color: Color, board: &Bitboards) -> Vec<Move>{
        let king = 1u64 << king_sq;
        let mut moves = 0u64;
        let my_pieces: u64 = Self::get_own_pieces(board, color);
        let opponent_pieces = Self::get_opponent_pieces(board, color);
        
        if color == Color::White {
            if !Self::is_square_occupied(my_pieces, 5, 0)        // f1 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 5, 0)   // f1 has no enemy pieces
            && !Self::is_square_occupied(my_pieces, 6, 0)        // g1 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 6, 0)   // g1 has no enemy pieces
            && !make_move::is_square_attacked(board, 4, color)  // e1 not attacked
            && !make_move::is_square_attacked(board, 5, color)  // f1 not attacked
            && !make_move::is_square_attacked(board, 6, color)  // g1 not attacked
            && board.white_kingside
            && rook_sq == 7{
                moves |= king << 2;
            }
            if !Self::is_square_occupied(my_pieces, 1, 0)        // b1 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 1, 0)   // b1 has no enemy pieces
            && !Self::is_square_occupied(my_pieces, 2, 0)        // c1 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 2, 0)   // c1 has no enemy pieces
            && !Self::is_square_occupied(my_pieces, 3, 0)        // d1 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 3, 0)   // d1 has no enemy pieces
            && !make_move::is_square_attacked(board, 4, color)  // e1 not attacked
            && !make_move::is_square_attacked(board, 3, color)  // d1 not attacked
            && !make_move::is_square_attacked(board, 2, color)  // c1 not attacked
            && board.white_queenside
            && rook_sq == 0 {
                moves |= king >> 2;

            }
        } else  {
            if !Self::is_square_occupied(my_pieces, 62, 0)        // g8 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 62, 0)   // g8 has no enemy pieces
            && !Self::is_square_occupied(my_pieces, 61, 0)        // f8 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 61, 0)   // f8 has no enemy pieces
            && !make_move::is_square_attacked(board, 60, color)  // e8 not attacked
            && !make_move::is_square_attacked(board, 61, color)  // f8 not attacked
            && !make_move::is_square_attacked(board, 62, color)  // g8 not attacked
            && board.black_kingside
            && rook_sq == 63{
                moves |= king << 2;
            }
            if !Self::is_square_occupied(my_pieces, 59, 0)        // d8 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 59, 0)   // d8 has no enemy pieces
            && !Self::is_square_occupied(my_pieces, 58, 0)        // c8 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 58, 0)   // c8 has no enemy pieces
            && !Self::is_square_occupied(my_pieces, 57, 0)        // b8 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 57, 0)   // b8 has no enemy pieces
            && !make_move::is_square_attacked(board, 60, color)  // e8 not attacked
            && !make_move::is_square_attacked(board, 59, color)  // d8 not attacked
            && !make_move::is_square_attacked(board, 58, color)  // c8 not attacked
            && board.black_queenside
            && rook_sq == 56{
                moves |= king >> 2;
            }
        }
        //from_sq is a kings position before move as castling is kings move that involves a rook
        Self::moves_from_bitboard(king_sq, PieceType::King, moves, false, true, opponent_pieces)

    }
    /// Generates all pseudo-legal moves for the King at a given square.
    ///
    /// This includes standard one-square moves in all 8 directions and checks 
    /// for potential castling opportunities if the King is on its starting square.
    ///
    /// # Arguments
    ///
    /// * `sq` - The 0-63 index of the King's current position.
    /// * `color` - The color of the King.
    /// * `board` - A reference to the current game state.
    ///
    /// # Returns
    ///
    /// Returns a [`Vec<Move>`] containing both standard moves and valid castling moves.
    ///
    /// # Implementation Details
    ///
    /// * **Movement:** Uses bit-shifts combined with `FILE_A` and `FILE_H` masks 
    ///   to prevent "teleporting" across the board edges.
    /// * **Collision:** Automatically filters out destination squares occupied 
    ///   by friendly pieces using a bitwise AND-NOT (`& !own_pieces`).
    /// * **Castling:** Specifically checks squares 4 (White) and 60 (Black) 
    ///   to trigger the castling logic.
    fn king_moves(sq: u8, color: Color, board: &Bitboards) -> Vec<Move> {
        let king = 1u64 << sq;
        let own_pieces = Self::get_own_pieces(board, color);
        let opponent_pieces = Self::get_opponent_pieces(board, color);
        let mut all_moves = Vec::new();


        // Generate all possible king moves (8 directions)
        let mut possible_moves = 0u64;
        possible_moves |= (king & !FILE_H) << 1;  // East
        possible_moves |= (king & !FILE_A) >> 1;  // West
        possible_moves |= king << 8;              // North
        possible_moves |= king >> 8;              // South
        possible_moves |= (king & !FILE_H) << 9;  // North-East
        possible_moves |= (king & !FILE_A) << 7;  // North-West
        possible_moves |= (king & !FILE_H) >> 7;  // South-East
        possible_moves |= (king & !FILE_A) >> 9;  // South-West

        // Remove moves to squares occupied by own pieces
        let moves = possible_moves & !own_pieces;


        // Add normal king moves
        all_moves.extend(Self::moves_from_bitboard(sq, PieceType::King, moves, false, false, opponent_pieces));

        // Add castling moves separately
        if color == Color::White && sq == 4 {
            all_moves.extend(Self::castling(4, 7, color, board));
            all_moves.extend(Self::castling(4, 0, color, board));
        } else if color == Color::Black && sq == 60 {
            all_moves.extend(Self::castling(60, 63, color, board));
            all_moves.extend(Self::castling(60, 56, color, board));
        }

        all_moves
    }


    /// Generates all pseudo-legal moves for a Pawn at a given square.
    ///
    /// Pawns have the most complex move generation logic due to:
    /// 1. Asymmetric movement (pushes) vs. captures.
    /// 2. Double-push capability from the starting rank.
    /// 3. The En Passant capture rule.
    /// 4. Promotion logic upon reaching the final rank.
    ///
    /// # Arguments
    ///
    /// * `sq` - The 0-63 index of the pawn's current position.
    /// * `color` - The color of the pawn (White moves "Up" +8, Black moves "Down" -8).
    /// * `board` - A reference to the current game state, including En Passant flags.
    ///
    /// # Returns
    ///
    /// Returns a [`Vec<Move>`] of all possible pawn actions.
    ///
    /// # Implementation Highlights
    ///
    /// * **Collision Detection:** Pushes are only valid if the target square is empty. 
    ///   Captures are only valid if the target square contains an opponent piece.
    /// * **En Passant:** Checks the `en_passant_square` from the board state to allow 
    ///   captures on pawns that just performed a double-push.
    /// * **Promotion:** Sets the `promotion_rights` flag if the pawn is currently 
    ///   on the 7th rank (White) or 2nd rank (Black).
    fn pawn_moves(sq: u8, color: Color, board: &Bitboards) -> Vec<Move> {
        //Make a bitboard representing all the pieces in the board
        let all_pieces: u64 = Self::get_own_pieces(board, color);
        let opponent_pieces = Self::get_opponent_pieces(board, color);

        let pawn = 1u64 << sq;
        let mut moves = 0u64;
        let promotion: bool;


        if color == Color::White {
            let all_pieces_combined = all_pieces | opponent_pieces;
            if !Self::is_square_occupied(all_pieces_combined, sq, 8) {
                moves |= pawn << 8;  // FIXED: Left shift to move UP
                // FIXED: White pawns start on rank 2 (squares 8-15)
                if !Self::is_square_occupied(all_pieces_combined, sq, 16)
                    && sq >= 8 && sq <= 15 {  // FIXED: Rank 2, not rank 8
                    moves |= pawn << 16;  // FIXED: Left shift for two squares up
                }
            }
            // Diagonal captures (these look correct)
            if Self::is_square_occupied(opponent_pieces, sq, 9) {
                moves |= (pawn & !FILE_H) << 9;
            }
            if Self::is_square_occupied(opponent_pieces, sq, 7) {
                moves |= (pawn & !FILE_A) << 7;
            }
        }    else if color == Color::Black {
            let all_pieces_combined = all_pieces | opponent_pieces; // Combine both colors
            if !Self::is_square_occupied(all_pieces_combined, sq, -8) {
                moves |= pawn >> 8;
                if !Self::is_square_occupied(all_pieces_combined, sq, -16)
                    && sq > 47 && sq < 56 {
                    moves |= pawn >> 16;
                }
            }
            if Self::is_square_occupied(opponent_pieces, sq, -9) {
                moves |= (pawn & !FILE_A) >> 9; // Takes on diagonal to the east
            }
            if Self::is_square_occupied(opponent_pieces, sq, -7) {
                moves |= (pawn & !FILE_H) >> 7; //Takes on diagonal to the west
            }

        }



        if let Some(ep_square) = board.en_passant_square {
            let sq_i = sq as i16;
            let ep_i = ep_square as i16;

            if (ep_i - sq_i).abs() == 1 {
                let ep_capture_square = if color == Color::White {
                    ep_square - 8
                } else {
                    ep_square + 8
                };
                if ep_capture_square < 64 {
                    moves |= 1u64 << ep_capture_square;
                }
            }
        }



        if  (color == Color::White && sq >= 48 && sq <= 55) ||
            (color == Color::Black && sq >= 8 && sq <= 15) {
                promotion = true;
            } else {
                promotion = false;
            }

        Self::moves_from_bitboard(sq, PieceType::Pawn, moves, promotion, false, opponent_pieces)
    }



    /// Generates sliding moves for a Rook along ranks and files.
    /// 
    /// It "casts a ray" in four directions (North, South, East, West), 
    /// stopping when it encounters a piece or the board edge.
    /// 
    /// # Arguments
    /// * `sq` - Starting square index.
    /// * `color` - Color of the Rook.
    /// * `board` - Current bitboard state.
    pub fn rook_moves(sq: u8, color: Color, board: &Bitboards) -> Vec<Move> {
        let mut moves = 0u64;
        let own_pieces = Self::get_own_pieces(board, color);
        let opponent_pieces = Self::get_opponent_pieces(board, color);

        // NORTH
        let mut next_sq = sq + 8;
        while next_sq <= 63 {
            if Self::is_square_occupied(own_pieces, next_sq, 0) {
                break;
            }
            moves |= 1u64 << next_sq;
            if Self::is_square_occupied(opponent_pieces, next_sq, 0) {
                break;
            }
            next_sq += 8;
        }

        // SOUTH
        let mut next_sq = sq.wrapping_sub(8);
        while next_sq <= 63 {
            if Self::is_square_occupied(own_pieces, next_sq, 0) {
                break;
            }
            moves |= 1u64 << next_sq;
            if Self::is_square_occupied(opponent_pieces, next_sq, 0) {
                break;
            }
            if next_sq < 8 { break; }
            next_sq -= 8;
        }

        // EAST

        let mut next_sq = sq + 1;
        while next_sq % 8 != 0 {
            if Self::is_square_occupied(own_pieces, next_sq, 0) {
                break;
            }
            moves |= 1u64 << next_sq;
            if Self::is_square_occupied(opponent_pieces, next_sq, 0) {
                break;
            }
            next_sq += 1;
        }

        // WEST
        let mut next_sq = sq.wrapping_sub(1);
        while sq % 8 != 0 && next_sq % 8 != 7 {
            if Self::is_square_occupied(own_pieces, next_sq, 0) {
                break;
            }
            moves |= 1u64 << next_sq;
            if Self::is_square_occupied(opponent_pieces, next_sq, 0) {
                break;
            }
            if next_sq == 0 { break; }
            next_sq -= 1;
        }

        Self::moves_from_bitboard(sq, PieceType::Rook, moves, false, false, opponent_pieces)
    }


    /// Generates sliding moves for a Bishop along diagonals.
    /// 
    /// # Arguments
    /// * `sq` - Starting square index.
    /// * `color` - Color of the Rook.
    /// * `board` - Current bitboard state
    ///
    /// Uses directional increments (+9, +7, -7, -9) to simulate diagonal 
    /// movement. Includes checks for edge-wrapping to prevent the ray 
    /// from jumping between the A and H files.
    pub fn bishop_moves(sq: u8, color: Color, board: &Bitboards) -> Vec<Move> {
        let mut moves = 0u64;

        let own_pieces = Self::get_own_pieces(board, color);
        let opponent_pieces = Self::get_opponent_pieces(board, color);

        // Northeast (+9)
        let mut pos = sq;
        while pos < 56 && (1u64 << pos) & FILE_H == 0 {
            pos += 9;
            if pos >= 64 {
                break;
            }
            if Self::is_square_occupied(own_pieces, pos, 0) {
                break;
            }
            moves |= 1u64 << pos;
            if Self::is_square_occupied(opponent_pieces, pos, 0) {
                break;
            }
            if pos % 8 == 7 { break; } // prevent edge wrapping
        }

        // Northwest (+7)
        pos = sq;
        while pos < 56 && (1u64 << pos) & FILE_A == 0 {
            pos += 7;
            if pos >= 64 {
                break;
            }
            if Self::is_square_occupied(own_pieces, pos, 0) {
                break;
            }
            moves |= 1u64 << pos;
            if Self::is_square_occupied(opponent_pieces, pos, 0) {
                break;
            }
            if pos % 8 == 0 { break; } // prevent edge wrapping
        }

        // Southeast (-7)
        pos = sq;
        while pos >= 7 && (1u64 << pos) & FILE_H == 0 {
            pos = pos.wrapping_sub(7);
            if Self::is_square_occupied(own_pieces, pos, 0) {
                break;
            }
            moves |= 1u64 << pos;
            if Self::is_square_occupied(opponent_pieces, pos, 0) {
                break;
            }
            if pos % 8 == 7 { break; }
            if pos < 7 { break; }
        }

        // Southwest (-9)
        pos = sq;
        while pos >= 9 && (1u64 << pos) & FILE_A == 0 {
            pos = pos.wrapping_sub(9);
            if Self::is_square_occupied(own_pieces, pos, 0) {
                break;
            }
            moves |= 1u64 << pos;
            if Self::is_square_occupied(opponent_pieces, pos, 0) {
                break;
            }
            if pos % 8 == 0 { break; }
            if pos < 9 { break; }
        }

        Self::moves_from_bitboard(sq, PieceType::Bishop, moves, false, false, opponent_pieces)
    }


    /// Generates all Queen moves by combining the move sets of a Rook and a Bishop.
    /// 
    /// This function effectively "re-labels" the moves generated by the Rook 
    /// and Bishop as Queen moves before collecting them.
    fn queen_moves(sq: u8, color: Color, board: &Bitboards) -> Vec<Move> {
        let rook_moves = Move::rook_moves(sq, color, board)
            .into_iter()
            .map(|mut m| {
                m.piece = PieceType::Queen;
                m
            });

        let bishop_moves = Move::bishop_moves(sq, color, board)
            .into_iter()
            .map(|mut m| {
                m.piece = PieceType::Queen;
                m
            });

        rook_moves.chain(bishop_moves).collect()
    }

    /// Converts algebraic notation (e.g., "e2") into a bitboard index (0-63).
    /// 
    /// This is a utility function used primarily for parsing user input 
    /// or UCI commands.
    /// 
    /// # Returns
    /// A bit representation of a move in u8
    ///
    /// # Examples
    /// "a1" -> 0
    /// "h8" -> 63
    fn move_coordinates_to_bit(mv: &str) -> u8 {
        let file = (mv.chars().nth(0).unwrap() as u8) - b'a';
        let rank = (mv.chars().nth(1).unwrap() as u8) - b'1';

        rank * 8 + file
    }


    /// Parses a UCI-style string (e.g., "e2e4") and returns the corresponding legal Move.
    ///
    /// This function acts as a safety layer: it converts coordinate strings into internal 
    /// bit indices, generates all legal moves for the side, and verifies that the 
    /// requested move is actually valid.
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice representing the move in UCI format (e.g., "e2e4").
    /// * `board` - The current game state bitboards.
    /// * `color` - The side making the move.
    ///
    /// # Returns
    ///
    /// Returns `Some(Move)` if the input corresponds to a valid legal move. 
    /// Returns `None` if the move is illegal or the input is malformed.
    ///
    /// # Note on Promotions
    ///
    /// Currently, this parser handles standard 4-character moves. Support for 
    /// 5-character promotion strings (e.g., "a7a8q") is a planned UCI enhancement.
    pub fn uci_user_parser(
        input: &str,
        board: &Bitboards,
        color: Color
    ) -> Option<Move> {
        let (from_uci, to_uci) = input.split_at(2);
        let from = Self::move_coordinates_to_bit(from_uci);
        let to = Self::move_coordinates_to_bit(to_uci);

        let moves = Self::generate_moves_for_side(color, board);
        moves.into_iter().find(|m| m.from == from && m.to == to)
    }


    /// Converts a 0-63 bit index back into a UCI-standard square string (e.g., "e2").
    ///
    /// This is the inverse of [`Self::move_coordinates_to_bit`]. It decomposes a linear 
    /// index into its file (column) and rank (row) components using modulo and 
    /// division operators.
    ///
    /// # Arguments
    ///
    /// * `bit` - The 0-63 index of the square.
    ///
    /// # Returns
    ///
    /// Returns a [`String`] containing the two-character algebraic notation.
    ///
    /// # Implementation Details
    ///
    /// * **File Calculation:** `bit % 8` yields 0-7, which is mapped to 'a' through 'h'.
    /// * **Rank Calculation:** `bit / 8` yields 0-7, which is mapped to '1' through '8'.
    /// * **ASCII Offset:** Uses byte literals (`b'a'`, `b'1'`) to perform efficient 
    ///   character arithmetic.
    fn bit_to_uci(bit: u8) -> String {
        let file = (bit % 8) as u8;       // 0..7 → a..h
        let rank = (bit / 8) as u8;       // 0..7 → 1..8
        let file_char = (b'a' + file) as char;
        let rank_char = (b'1' + rank) as char;
        format!("{}{}", file_char, rank_char)
    }

    /// Converts a [`Move`] struct into a UCI-standard coordinate string (e.g., "e2e4").
    ///
    /// This function serves as the primary output for the engine's move selection, 
    /// combining the starting and destination squares into a single string.
    ///
    /// # Arguments
    ///
    /// * `mv` - A reference to the [`Move`] to be converted.
    ///
    /// # Returns
    ///
    /// Returns a [`String`] like "g1f3". 
    ///
    /// # Note on Promotions
    ///
    /// This current implementation does not append the promotion piece character 
    /// (e.g., 'q' for "a7a8q"). This is a targeted enhancement for full UCI compliance.
    pub fn move_to_uci(mv: &Move) -> String {
        let from_str = Self::bit_to_uci(mv.from);
        let to_str = Self::bit_to_uci(mv.to);
        format!("{}{}", from_str, to_str)
    }


    /// Converts a destination bitboard into a list of discrete [`Move`] objects.
    ///
    /// This function "unpacks" a 64-bit integer where each set bit represents a 
    /// potential destination for a piece. It performs a bit-scan to find piece 
    /// indices and identifies captures on the fly.
    ///
    /// # Arguments
    ///
    /// * `from_sq` - The starting square index of the piece.
    /// * `piece` - The type of piece being moved.
    /// * `destinations` - A [`u64`] bitboard containing all valid target squares.
    /// * `promotion_rights` - Flag indicating if these moves result in a promotion.
    /// * `is_castling` - Flag indicating if this is a castling move.
    /// * `opponent_pieces` - Bitboard of all opponent pieces for capture detection.
    ///
    /// # Returns
    ///
    /// Returns a [`Vec<Move>`] containing one struct for every set bit in `destinations`.
    ///
    /// # Performance Note
    ///
    /// Uses `trailing_zeros()` (Bit Scan Forward) and `bb &= bb - 1` to iterate 
    /// only over set bits, avoiding an expensive 64-iteration loop.
    fn moves_from_bitboard(
        from_sq: u8,
        piece: PieceType,
        destinations: u64,
        promotion_rights: bool,
        is_castling: bool,
        opponent_pieces: u64,  
    ) -> Vec<Move> {
        let mut moves_vec = Vec::new();
        let mut bitboard_copy = destinations;

        while bitboard_copy != 0 {
            let to_square = bitboard_copy.trailing_zeros() as u8;
            bitboard_copy &= bitboard_copy - 1;

            let is_capture = (opponent_pieces >> to_square) & 1 != 0;

            let m = Move {
                from: from_sq,
                to: to_square,
                piece,
                promotion_rights,
                is_castling,
                is_capture,
            };

            moves_vec.push(m);
        }

        moves_vec
    }
}
