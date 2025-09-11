use indicatif::{ProgressBar, ProgressStyle};
use crate::book;
use crate::board::{
    Color,
    Bitboards
};
use crate::engine::game_over;
use crate::evaluation::evaluation;
use crate::movegen::Move;
use crate::make_move;
use std::collections::HashMap;


pub fn opposite(color: Color) -> Color {
        let opposite = match color {
        Color::White => Color::Black,
        Color::Black => Color::White
    };
    opposite
}
fn negamax(board: &Bitboards, depth: u32, color: Color, mut alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
        let eval = evaluation(board, color);
        return eval

    }
    if game_over::checkmate(board, color) {
        return -10_000;
    }

    let moves = make_move::generate_legal_moves(board, color);
    if moves.is_empty() {
        return 0; // Stalemate
    }

    let mut best = i32::MIN;
    for mv in moves {
        let mut clone = board.clone();
        make_move::apply_move(&mut clone, &mv, color);

        // Flip alpha & beta, and flip color
        let score = -negamax(&clone, depth - 1, opposite(color), -beta, -alpha);

        best = best.max(score);
        alpha = alpha.max(score);

        if alpha >= beta {
            break;
        }

    }

    
    best
}

pub fn best_move(
    board: &mut Bitboards,
    depth: u32,
    color: Color,
    fen: &str,
    book: &HashMap<String, HashMap<String, book::MoveEntry>>,
    move_count: u8
) -> Option<Move> {
    let moves = Move::generate_moves_for_side(color, board);
    if moves.is_empty() {
        return None;
    }
    
    if move_count < 11 {
        println!("{}", fen);
        if let Some(opening_move) = book::opening(book, fen) {
            let mut board_clone = board.clone();
            make_move::apply_move(&mut board_clone, &opening_move, color);
            println!("Board before move:");
            Bitboards::render_board(&board);
            println!("After move:");
            Bitboards::render_board(&board_clone);
            return Some(opening_move);
        }
    }

    // Search path
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner} {msg}")
            .unwrap()
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb.set_message("Evaluating position...");

    let mut best_score = i32::MIN;
    let mut best_move_search = None;

    for mv in moves {
        let mut clone = board.clone();
        make_move::apply_move(&mut clone, &mv, color);
        let score = -negamax(
            &clone,
            depth - 1,
            opposite(color),
            i32::MIN + 1,
            i32::MAX,
        );
        if score > best_score {
            best_score = score;
            best_move_search = Some(mv);
        }
        if score == 10_000 {
            break;
        }
        pb.inc(1);
    }
    pb.finish_with_message("Search completed!");
    // print board preview for chosen move
    let mut board_clone = board.clone();
    if let Some(ref mv) = best_move_search {
        make_move::apply_move(&mut board_clone, mv, color);
        println!("Board before move:");
        Bitboards::render_board(&board);
        println!("Engine's choice:");
        Bitboards::render_board(&board_clone);
    }
    best_move_search
}




