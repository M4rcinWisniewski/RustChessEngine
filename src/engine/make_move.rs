use crate::board::{
    PieceType,
    Color,
    Bitboards
};

use crate::movegen::Move;



pub fn is_square_attacked(board: &Bitboards, sq: u8, color: Color) -> bool {
    let enemy_color = match color {
        Color::White => Color::Black,
        Color::Black => Color::White
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
        let distance = (sq as i8 - king_sq as i8).abs().max((sq % 8) as i8 - (king_sq % 8) as i8).abs();
        if distance == 1 {  // King attacks 1 square away
            return true;
        }
    }

    false
}

fn is_valid_square(sq: u8) -> bool {
    sq < 64
}

fn apply_move(board: &mut Bitboards, mv: &Move, color: Color) {
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
        if rank_diff == 16 { // Two-square pawn move
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
            (4, 6) => {  // White kingside: e1→g1
                board.boards[Color::White as usize][PieceType::Rook as usize] &= !(1u64 << 7); // Remove rook from h1
                board.boards[Color::White as usize][PieceType::Rook as usize] |= 1u64 << 5;    // Place rook on f1
            }
            (4, 2) => {  // White queenside: e1→c1
                board.boards[Color::White as usize][PieceType::Rook as usize] &= !(1u64 << 0); // Remove rook from a1
                board.boards[Color::White as usize][PieceType::Rook as usize] |= 1u64 << 3;    // Place rook on d1
            }
            (60, 62) => { // Black kingside: e8→g8
                board.boards[Color::Black as usize][PieceType::Rook as usize] &= !(1u64 << 63); // Remove rook from h8
                board.boards[Color::Black as usize][PieceType::Rook as usize] |= 1u64 << 61;    // Place rook on f8
            }
            (60, 58) => { // Black queenside: e8→c8
                board.boards[Color::Black as usize][PieceType::Rook as usize] &= !(1u64 << 56); // Remove rook from a8
                board.boards[Color::Black as usize][PieceType::Rook as usize] |= 1u64 << 59;    // Place rook on d8
            }
            _ => {}
        }
    }
    //update castle rights when king moves
    if mv.piece == PieceType::King {
        if color == Color::White {
            board.white_kingside = false;
            board.white_queenside = false;
        }
        else {
            board.black_kingside = false;
            board.black_queenside = false
        }
    }
    // update castle rights when rook moves/is captured
    if mv.from == 0 || mv.to == 0 { board.white_queenside = false; }
    if mv.from == 7 || mv.to == 7 { board.white_kingside = false; }
    if mv.from == 56 || mv.to == 56 { board.black_queenside = false; }
    if mv.from == 63 || mv.to == 63 { board.black_kingside = false; }
}
