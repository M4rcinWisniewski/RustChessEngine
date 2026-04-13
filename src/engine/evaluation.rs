//! # Module: `evaluation`
//!
//! This module acts as the **Judgment Center** or the "Brain" of the engine. While the
//! move generator identifies what *can* be done, the evaluation module determines
//! what *should* be done by assigning a numerical score to any given board state.
//!
//! The score is calculated from the perspective of the side to move, where a
//! positive value indicates an advantage and a negative value indicates a disadvantage.
//!
//! ## Evaluation Components
//!
//! The final score is a weighted sum of several distinct heuristics:
//!
//! ### 1. Material Balance
//! The most fundamental metric. Each piece is assigned a static value:
//! * **Pawn:** 100 | **Knight:** 320 | **Bishop:** 330 | **Rook:** 500 | **Queen:** 900.
//!
//! ### 2. Piece-Square Tables (PST)
//! Positional value based on where pieces are placed. For example, Knights are
//! rewarded for being in the center and penalized for being on the "rim,"
//! while Pawns are rewarded for advancing toward promotion.
//! * **King Safety:** The King uses different PSTs depending on the game phase.
//!   In the **Middlegame**, it seeks safety in the corners; in the **Endgame**,
//!   it is encouraged to move to the center to become an active attacker.
//!
//! [Image of piece-square tables in computer chess]
//!
//! ### 3. Mobility Score
//! Measures "spatial control" by counting the number of pseudo-legal moves
//! available to each piece, weighted by piece type. An engine with more
//! mobility is generally considered to have a more flexible and dominant position.
//!
//! ### 4. Development & Heuristics
//! Specific tactical bonuses and penalties:
//! * **Center Control:** Extra points for pawns occupying d4, e4, d5, and e5.
//! * **Castling Bonus:** Significant rewards for successfully castling the King.
//! * **Knight Rim Penalty:** Discourages placing Knights on the edges of the
//!   board ("A Knight on the rim is dim").
//! * **Early Flank Pawn Penalty:** Discourages pushing edge pawns too early
//!   in the opening, which can weaken the King's future home.
//!
//! [Image of center control and pawn structure in chess evaluation]
//!
//! ## Game Phase Detection
//! The engine uses [`is_endgame`] to dynamically shift its priorities.
//! If the Queens are off the board (or very few pieces remain), the engine
//! transitions into "Endgame Mode," fundamentally changing how it values
//! King placement.

use crate::board::{Bitboards, Color, PieceType};
use crate::movegen::Move;

// mobility weight gives mobility a proper weigth in final eval

const PST_WEIGHT: i32 = 5;

const CENTRAL_SQUARES: [u8; 4] = [27, 28, 35, 36]; // d4, e4, d5, e5

const RIM_SQUARES: [u8; 16] = [0, 8, 16, 24, 32, 40, 48, 56, 7, 15, 23, 31, 39, 47, 55, 63];

// Piece-square table for pawn
// White Pawn PST (Encourages advancement and slight center control)
const WHITE_PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 8 (Promotion handled elsewhere)
    50, 50, 50, 50, 50, 50, 50, 50, // Rank 7 (Almost promoted!)
    10, 10, 20, 30, 30, 20, 10, 10, // Rank 6
    5, 5, 10, 25, 25, 10, 5, 5, // Rank 5
    0, 0, 0, 20, 20, 0, 0, 0, // Rank 4
    5, -5, -10, 0, 0, -10, -5, 5, // Rank 3
    5, 10, 10, -20, -20, 10, 10, 5, // Rank 2 (Defending the King)
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 1
];

// Black Pawn PST (Flipped version of White)
const BLACK_PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 1
    5, 10, 10, -20, -20, 10, 10, 5, // Rank 2
    5, -5, -10, 0, 0, -10, -5, 5, // Rank 3
    0, 0, 0, 20, 20, 0, 0, 0, // Rank 4
    5, 5, 10, 25, 25, 10, 5, 5, // Rank 5
    10, 10, 20, 30, 30, 20, 10, 10, // Rank 6
    50, 50, 50, 50, 50, 50, 50, 50, // Rank 7
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 8
];

// White Knight PST (Index 0 = a8)
const WHITE_KNIGHT_PST: [i32; 64] = [
    -30, -15, -10, -10, -10, -10, -15, -30, // Rank 8
    -15, 0, 0, 5, 5, 0, 0, -15, // Rank 7
    -10, 5, 10, 15, 15, 10, 5, -10, // Rank 6
    -10, 10, 15, 20, 20, 15, 10, -10, // Rank 5
    -10, 5, 15, 20, 20, 15, 5, -10, // Rank 4
    -10, 0, 10, 15, 15, 10, 0, -10, // Rank 3
    -15, -5, 0, 0, 0, 0, -5, -15, // Rank 2
    -30, -15, -10, -10, -10, -10, -15, -30, // Rank 1
];

// Black Knight PST (Vertical Flip)
const BLACK_KNIGHT_PST: [i32; 64] = [
    -30, -15, -10, -10, -10, -10, -15, -30, -15, -5, 0, 0, 0, 0, -5, -15, -10, 0, 10, 15, 15, 10,
    0, -10, -10, 5, 15, 20, 20, 15, 5, -10, -10, 10, 15, 20, 20, 15, 10, -10, -10, 5, 10, 15, 15,
    10, 5, -10, -15, 0, 0, 5, 5, 0, 0, -15, -30, -15, -10, -10, -10, -10, -15, -30,
];

// White Bishop (Index 0 = a8)
const WHITE_BISHOP_PST: [i32; 64] = [
    -10, -10, -10, -10, -10, -10, -10, -10, -10, 5, 0, 0, 0, 0, 5, -10, -10, 0, 5, 10, 10, 5, 0,
    -10, -10, 5, 10, 15, 15, 10, 5, -10, -10, 0, 10, 15, 15, 10, 0, -10, -10, 5, 5, 10, 10, 5, 5,
    -10, -10, 0, 0, 0, 0, 0, 0, -10, -10, -10, -10, -10, -10, -10, -10, -10,
];

const BLACK_BISHOP_PST: [i32; 64] = [
    -10, -10, -10, -10, -10, -10, -10, -10, // Rank 8 (Home)
    -10, 0, 0, 0, 0, 0, 0, -10, // Rank 7
    -10, 5, 5, 10, 10, 5, 5, -10, // Rank 6
    -10, 0, 10, 15, 15, 10, 0, -10, // Rank 5
    -10, 5, 10, 15, 15, 10, 5, -10, // Rank 4
    -10, 0, 5, 10, 10, 5, 0, -10, // Rank 3
    -10, 5, 0, 0, 0, 0, 5, -10, // Rank 2
    -10, -10, -10, -10, -10, -10, -10, -10, // Rank 1 (Enemy)
];

// Piece-square table for rook
const WHITE_ROOK_PST: [i32; 64] = [
    0, 0, 0, 5, 5, 0, 0, 0, 10, 15, 15, 15, 15, 15, 15, 10, // Reward for 7th rank!
    -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0,
    0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 0, 0, 0, 5, 5, 0, 0, 0, // Centralize on start
];

const BLACK_ROOK_PST: [i32; 64] = [
    0, 0, 0, 5, 5, 0, 0, 0, // Rank 8 (Home)
    -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0,
    0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 10, 15, 15, 15, 15, 15, 15,
    10, // Rank 2 (Attacking White pawns)
    0, 0, 0, 5, 5, 0, 0, 0, // Rank 1
];

//Piece-square table for queen
const WHITE_QUEEN_PST: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 5, 0, 0, 0, 0, -10, -10, 5, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 0, 5, 5, 5, 5, 0, -10, -10, 0, 0, 0, 0,
    0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
];

const BLACK_QUEEN_PST: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, // Rank 8
    -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10, 0, 0, 5, 5, 5, 5, 0, -5, -5, 0, 5, 5,
    5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0, 5, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5,
    -10, -10, -20, // Rank 1
];

//Piece-square table for king
const WHITE_KING_MIDDLEGAME_PST: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40,
    -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40, -40, -30,
    -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 10, 10, -5, -10, -10, -5, 10, 10, 20, 30, 10,
    0, 0, 10, 30, 20, // Values are lower (max 30)
];

const BLACK_KING_MIDDLEGAME_PST: [i32; 64] = [
    20, 30, 10, 0, 0, 10, 30, 20, // Rank 8 (Safe Home)
    10, 10, -5, -10, -10, -5, 10, 10, // Rank 7
    -10, -20, -20, -20, -20, -20, -20, -10, -20, -30, -30, -40, -40, -30, -30, -20, -30, -40, -40,
    -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40,
    -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, // Rank 1 (Danger Zone)
];

const WHITE_KING_ENDGAME_PST: [i32; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50, -30, -20, -10, 0, 0, -10, -20, -30, -30, -10, 20, 30,
    30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30,
    -10, 20, 30, 30, 20, -10, -30, -30, -30, 0, 0, 0, 0, -30, -30, -50, -30, -30, -30, -30, -30,
    -30, -50,
];

const BLACK_KING_ENDGAME_PST: [i32; 64] = [
    -50, -30, -30, -30, -30, -30, -30, -50, -30, -30, 0, 0, 0, 0, -30, -30, -30, -10, 20, 30, 30,
    20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10,
    20, 30, 30, 20, -10, -30, -30, -20, -10, 0, 0, -10, -20, -30, -50, -40, -30, -20, -20, -30,
    -40, -50,
];

pub fn evaluation(board: &Bitboards, color: Color) -> i32 {
    // Decide which side is "friendly" and which is "enemy"
    let (friendly_idx, enemy_idx, friendly_color, enemy_color) = match color {
        Color::White => (0, 1, Color::White, Color::Black),
        Color::Black => (1, 0, Color::Black, Color::White),
    };

    /* MATERIAL SCORE */
    let mut friendly_score = 0i32;
    let mut enemy_score = 0i32;

    // pawn, rook, knight, bishop, queen
    let piece_values: [i32; 5] = [100, 500, 320, 330, 900];
    for i in 0..piece_values.len() {
        let friendly_pieces = Bitboards::count_pieces(board.boards[friendly_idx][i]);
        let enemy_pieces = Bitboards::count_pieces(board.boards[enemy_idx][i]);
        friendly_score += friendly_pieces * piece_values[i];
        enemy_score += enemy_pieces * piece_values[i];
    }

    let material_score = friendly_score - enemy_score;

    /* MOBILITY SCORE */
    let mut friendly_moves = 0i32;
    let mut enemy_moves = 0i32;
    let piece_mobility_weights = [
        1, // Pawn
        4, // Knight
        4, // Bishop
        5, // Rook
        9, // Queen
        0, // King
    ];

    for i in 0..PieceType::pieces().len() {
        let piece = PieceType::pieces()[i];

        // Friendly
        let friendly_squares = Bitboards::return_squares(board.boards[friendly_idx][i]);
        for sq in friendly_squares {
            let possible_moves = Move::generate_moves_for_piece(sq, piece, friendly_color, board);
            friendly_moves += piece_mobility_weights[i] * possible_moves.len() as i32;
        }

        // Enemy
        let enemy_squares = Bitboards::return_squares(board.boards[enemy_idx][i]);
        for sq in enemy_squares {
            let possible_moves = Move::generate_moves_for_piece(sq, piece, enemy_color, board);
            enemy_moves += piece_mobility_weights[i] * possible_moves.len() as i32;
        }
    }

    let mobility_score = friendly_moves - enemy_moves;

    /* PIECE-SQUARE TABLE SCORE */
    let mut pst_score = 0;
    let endgame = is_endgame(board);

    // Friendly PST
    for piece in PieceType::pieces() {
        let pst = pst_for(piece, friendly_color, endgame);
        for sq in Bitboards::return_squares(board.boards[friendly_idx][piece_type_index(piece)]) {
            pst_score += pst[sq as usize];
        }
    }

    // Enemy PST
    for piece in PieceType::pieces() {
        let pst = pst_for(piece, enemy_color, endgame);
        for sq in Bitboards::return_squares(board.boards[enemy_idx][piece_type_index(piece)]) {
            pst_score -= pst[sq as usize];
        }
    }

    /* DEVELOPMENT BONUS */
    let mut dev_bonus = 0;
    for sq in
        Bitboards::return_squares(board.boards[friendly_idx][piece_type_index(PieceType::Pawn)])
    {
        dev_bonus += center_pawns(sq);
        dev_bonus += pawn_development(sq, color);
        dev_bonus += early_flank_pawn_penalty(sq, enemy_color)
    }
    for sq in Bitboards::return_squares(board.boards[enemy_idx][piece_type_index(PieceType::Pawn)])
    {
        dev_bonus -= center_pawns(sq);
        dev_bonus -= pawn_development(sq, enemy_color);
        dev_bonus -= early_flank_pawn_penalty(sq, enemy_color)
    }
    for sq in
        Bitboards::return_squares(board.boards[enemy_idx][piece_type_index(PieceType::Knight)])
    {
        dev_bonus -= knight_penalty(sq);
    }
    for sq in
        Bitboards::return_squares(board.boards[friendly_idx][piece_type_index(PieceType::Knight)])
    {
        dev_bonus += knight_penalty(sq);
    }
    for sq in
        Bitboards::return_squares(board.boards[friendly_idx][piece_type_index(PieceType::King)])
    {
        dev_bonus += castle_bonus(sq, color) * 2;
    }
    for sq in Bitboards::return_squares(board.boards[enemy_idx][piece_type_index(PieceType::King)])
    {
        dev_bonus -= castle_bonus(sq, enemy_color) * 2;
    }
    // println!("{}", dev_bonus);
    /* FINAL SCORE */
    let eval = material_score + pst_score * PST_WEIGHT + mobility_score + dev_bonus;
    eval
}

fn pst_for(piece: PieceType, color: Color, endgame: bool) -> &'static [i32; 64] {
    match (piece, color) {
        (PieceType::Pawn, Color::White) => &WHITE_PAWN_PST,
        (PieceType::Pawn, Color::Black) => &BLACK_PAWN_PST,
        (PieceType::Knight, Color::White) => &WHITE_KNIGHT_PST,
        (PieceType::Knight, Color::Black) => &BLACK_KNIGHT_PST,
        (PieceType::Bishop, Color::White) => &WHITE_BISHOP_PST,
        (PieceType::Bishop, Color::Black) => &BLACK_BISHOP_PST,
        (PieceType::Rook, Color::White) => &WHITE_ROOK_PST,
        (PieceType::Rook, Color::Black) => &BLACK_ROOK_PST,
        (PieceType::Queen, Color::White) => &WHITE_QUEEN_PST,
        (PieceType::Queen, Color::Black) => &BLACK_QUEEN_PST,
        (PieceType::King, Color::White) => {
            if endgame {
                &WHITE_KING_ENDGAME_PST
            } else {
                &WHITE_KING_MIDDLEGAME_PST
            }
        }
        (PieceType::King, Color::Black) => {
            if endgame {
                &BLACK_KING_ENDGAME_PST
            } else {
                &BLACK_KING_MIDDLEGAME_PST
            }
        }
    }
}

fn center_pawns(square: u8) -> i32 {
    if CENTRAL_SQUARES.contains(&square) {
        40
    } else {
        0
    }
}

fn pawn_development(square: u8, color: Color) -> i32 {
    let rank = square / 8;
    match color {
        Color::White if rank == 3 || rank == 4 => 40, // encourage 2nd → 3rd/4th rank
        Color::Black if rank == 4 || rank == 3 => 40,
        _ => 0,
    }
}

fn early_flank_pawn_penalty(square: u8, color: Color) -> i32 {
    let file = square % 8;
    let rank = square / 8;
    match color {
        Color::White => {
            if (file == 6 || file == 7) && rank <= 3 {
                -15
            } else {
                0
            }
        }
        Color::Black => {
            if (file == 1 || file == 0) && rank >= 4 {
                -15
            } else {
                0
            }
        }
    }
}

fn knight_penalty(sq: u8) -> i32 {
    if RIM_SQUARES.contains(&sq) { -20 } else { 5 }
}

fn castle_bonus(sq: u8, color: Color) -> i32 {
    if color == Color::White {
        if sq == 2 {
            40
        } else if sq == 6 {
            50
        } else {
            0
        }
    } else {
        if sq == 62 {
            40
        } else if sq == 58 {
            50
        } else {
            0
        }
    }
}

fn piece_type_index(piece: PieceType) -> usize {
    match piece {
        PieceType::Pawn => 0,
        PieceType::Knight => 1,
        PieceType::Bishop => 2,
        PieceType::Rook => 3,
        PieceType::Queen => 4,
        PieceType::King => 5,
    }
}

fn is_endgame(board: &Bitboards) -> bool {
    let queens = Bitboards::count_pieces(board.boards[0][piece_type_index(PieceType::Queen)])
        + Bitboards::count_pieces(board.boards[1][piece_type_index(PieceType::Queen)]);
    let other_pieces = Bitboards::count_pieces(board.boards[0][piece_type_index(PieceType::Rook)])
        + Bitboards::count_pieces(board.boards[1][piece_type_index(PieceType::Rook)])
        + Bitboards::count_pieces(board.boards[0][piece_type_index(PieceType::Bishop)])
        + Bitboards::count_pieces(board.boards[1][piece_type_index(PieceType::Bishop)])
        + Bitboards::count_pieces(board.boards[0][piece_type_index(PieceType::Knight)])
        + Bitboards::count_pieces(board.boards[1][piece_type_index(PieceType::Knight)]);

    queens == 0 || (queens == 1 && other_pieces <= 1)
}
