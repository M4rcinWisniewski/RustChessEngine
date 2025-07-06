use crate::board::{
    PieceType,
    Color,
    Bitboards  
};
use crate::movegen::Move;

pub fn apply_move(board: &mut Bitboards, mv: &Move, color: Color) {
    let from_mask = 1u64 << mv.from;
    let to_mask = 1u64 << mv.to;

    // Remove piece from source square
    board.boards[color as usize][mv.piece as usize] &= !from_mask;

    // If it's a capture, remove enemy piece from destination square
    let enemy_color = match color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };
    for piece_type in 0..6 {
        if board.boards[enemy_color as usize][piece_type] & to_mask != 0 {
            board.boards[enemy_color as usize][piece_type] &= !to_mask;
            break;
        }
    }

    // Handle promotion
    if mv.promotion_rights {
        board.boards[color as usize][PieceType::Queen as usize] |= to_mask; //for simplicity it can only promote to Queen for now
    } else {
        // Regular move
        board.boards[color as usize][mv.piece as usize] |= to_mask;
    }

    /* !! TODO: Castiling !! */

}
