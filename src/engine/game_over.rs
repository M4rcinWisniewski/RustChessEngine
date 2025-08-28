use crate::{board::{
    Bitboards, Color
}, engine::make_move::is_check};
use crate::movegen::Move;
use crate::make_move;


pub fn checkmate(board: &Bitboards, color: Color) -> bool {
    if !is_check(board, color) {
        return false;
    }
    let moves = Move::generate_moves_for_side(color, board);

    for mv in moves {
        let mut clone = (*board).clone();       
        make_move::apply_move(&mut clone, &mv, color);
        if !is_check(&clone, color) {
            return false; 
        }
    }
    // 4. No legal moves left, checkmate
    true
}


