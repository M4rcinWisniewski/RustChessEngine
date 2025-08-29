use indicatif::{ProgressBar, ProgressStyle};

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
fn negamax(board: &Bitboards, depth: u32, color: Color, pb: &ProgressBar) -> i32 {
    if depth == 0 {
        pb.inc(1); // increment for this node
        return evaluation(board, color);
    }
    if game_over::checkmate(board, color) {
        pb.inc(1);
        return -10_000;
    }

    let moves = make_move::generate_legal_moves(board, color);
    if moves.is_empty() {
        pb.inc(1);
        return 0; // Stalemate
    }

    let mut best = i32::MIN;
    for mv in moves {
        let mut clone = board.clone();
        make_move::apply_move(&mut clone, &mv, color);

        // Flip alpha & beta, and flip color
        let score = -negamax(&clone, depth - 1, color, pb);

        best = best.max(score);
  
    }
    
    
    best
}

pub fn best_move(board: &mut Bitboards, depth: u32, color: Color) -> Option<(Move, i32)> {
    let moves = Move::generate_moves_for_side(color, board);
    if moves.is_empty() {
        return None;
    }

    // total nodes for progress bar = moves^depth (rough estimate)
    let total_nodes = moves.len().pow(depth);
    let pb = ProgressBar::new(total_nodes as u64);
    pb.set_style(
        ProgressStyle::with_template("{msg} [{wide_bar:.green}] {percent}% ({eta_precise})")
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏  ")
    );

    let mut best_score = i32::MIN;
    let mut best_move = None;

    for mv in moves {
        let mut clone = board.clone();
        make_move::apply_move(&mut clone, &mv, color);
        let score = -negamax(&clone, depth - 1, opposite(color), &pb);


        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
    }

    pb.finish_with_message("Search complete");

    // print board preview for chosen move
    let mut board_clone = board.clone();
    if let Some(ref mv) = best_move {
        make_move::apply_move(&mut board_clone, mv, color);
        println!("Board before move:");
        Bitboards::render_board(&board);
        println!("Engine's choice:");
        Bitboards::render_board(&board_clone);
    }

    best_move.map(|m| (m, best_score))
}
