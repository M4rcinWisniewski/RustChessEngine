use crate::board::{
    Color,
    Bitboards
};
use crate::engine::game_over;


use crate::evaluation::evaluation;
use crate::movegen::Move;
use crate::make_move;



pub fn opposite(color: Color) -> Color {
        let opposite = match color {
        Color::White => Color::Black,
        Color::Black => Color::White
    };
    opposite
}
fn negamax(board: &Bitboards, depth: u32, color: Color) -> i32 {
    if depth == 0 {
        return evaluation(board, color);
    }
    if game_over::checkmate(board, color) {
        return -10_000
    }
    let moves = make_move::generate_legal_moves(board, color);
    if moves.is_empty() {
        return 0; // Stalemate
    }
    else {
        let mut best = i32::MIN;
        for mv in moves {
            let mut clone = board.clone();
            make_move::apply_move(&mut clone, &mv, color);
            let score = -negamax(&clone, depth - 1, opposite(color));
            best = best.max(score);
        }
        best
    }
    
}

pub fn best_move(board: &mut Bitboards, depth: u32, color: Color) -> Option<(Move, i32)> {
    let moves = Move::generate_moves_for_side(color, board);
    if moves.is_empty() {
        return None;
    }
    
    let mut best_score = i32::MIN;
    let mut best_move = None;
    
    for mv in moves {
        let mut clone = board.clone();
        make_move::apply_move(&mut clone, &mv, color);
        let score = -negamax(&clone, depth - 1, opposite(color));
        
        println!("{:?}: {}", mv, score);
        
        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
    }
    let mut board_clone = board.clone();
    let move_clone = best_move.clone();
    make_move::apply_move(&mut board_clone, &move_clone.unwrap(), color);
    println!("Board before move:");
    Bitboards::render_board(&board);
    println!("Engines choice:");
    Bitboards::render_board(&board_clone);
    best_move.map(|m| (m, best_score))
}