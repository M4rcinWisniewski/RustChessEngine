mod engine;
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
};
use regex::Regex;
use std::io;
use std::io::stdout;

use clap::Parser;
use engine::{
    board::{self, Bitboards},
    evaluation, make_move, movegen, parse_fen, search,
};
use indicatif::{ProgressBar, ProgressStyle};
mod opening_book;
use opening_book::book;

use std::collections::HashMap;

use crate::engine::{game_over, make_move::apply_move, movegen::Move};
#[derive(Parser, Debug)]
struct Args {
    #[arg(
        short,
        long,
        default_value = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    )]
    fen: String,
    #[arg(short, long, default_value = "w")]
    color: char, //either w or b
}

fn clear_terminal() {
    execute!(stdout(), Clear(ClearType::All)).unwrap();
}

fn main() {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::with_template("{spinner} {msg}").unwrap());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb.set_message("Loading opening book...");

    let opening_book_data = std::fs::read_to_string("src/opening_book/book.json").unwrap();
    let book: HashMap<String, HashMap<String, book::MoveEntry>> =
        serde_json::from_str(&opening_book_data).unwrap();

    pb.finish_with_message("Opening book loaded successfully!");

    let args = Args::parse();
    let fen_position = parse_fen::parse_fen(&args.fen);
    let user_color = &args.color;
    let user_color = match user_color {
        'w' => board::Color::White,
        'b' => board::Color::Black,
        _ => unreachable!(),
    };
    let mut board = Bitboards::new();
    for (i, n) in fen_position.chars().enumerate() {
        if n == '.' {
            continue;
        }
        let square = i as u8;

        let color = if n.is_lowercase() {
            board::Color::Black
        } else {
            board::Color::White
        };

        let piece_type = match n.to_ascii_lowercase() {
            'p' => board::PieceType::Pawn,
            'r' => board::PieceType::Rook,
            'n' => board::PieceType::Knight,
            'b' => board::PieceType::Bishop,
            'q' => board::PieceType::Queen,
            'k' => board::PieceType::King,
            _ => unreachable!(),
        }; 

        Bitboards::add_piece(&mut board, color, piece_type, square);
    }
    // Validate UCI move syntax (simple) input like e2e4 or b1c3 legal but e8e9, illegal
    let re = Regex::new(r"^[a-h][1-8][a-h][1-8][qrbn]?$").unwrap();
    let mut color_to_move = parse_fen::side_to_move(&args.fen).unwrap();
    let mut move_count = 0;
    let mut fen = parse_fen::flat_board_to_fen(&fen_position);

    let moves = Move::generate_moves_for_side(board::Color::White, &board);

    for m in &moves {
        println!("Found Move: {:?}", m);
    }

    if user_color == board::Color::White {
        // Bitboards::render_board(&board);
        while !game_over::checkmate(&board, color_to_move) {
            if color_to_move == board::Color::Black {
                // Bitboards::render_board(&board);
                let mv = search::best_move(&mut board, 5, color_to_move, &fen, &book, move_count)
                    .unwrap();
                apply_move(&mut board, &mv, color_to_move);
                color_to_move = search::opposite(color_to_move);
                let uci = Move::move_to_uci(&mv);
                let (from, to) = uci.split_at(2);
                fen = parse_fen::update_fen(from, to, &fen);
            } else if color_to_move == board::Color::White {
                let mut input_const;

                let mv = loop {
                    let mut input = String::new();
                    println!("Play your next move! Your color is {:?},", color_to_move);

                    io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to read input");

                    let input = input.trim();

                    if !re.is_match(input) {
                        println!("'{}' is not a valid UCI format, please try again.", input);
                        continue;
                    }
                    input_const = input.to_string().clone();
                    match Move::uci_user_parser(input, &board, color_to_move) {
                        Some(mv) => {
                            println!("'{}' is a valid and legal move", input);
                            break mv; // success, exit loop
                        }
                        _none => {
                            println!(
                                "'{}' has correct format but is not legal here, try again.",
                                input
                            );

                            continue;
                        }
                    }
                };
                // clear_terminal();

                // Bitboards::render_board(&board);
                apply_move(&mut board, &mv, color_to_move);
                color_to_move = search::opposite(color_to_move);
                println!("{:?}", input_const);
                let (from, to) = input_const.split_at(2);
                fen = parse_fen::update_fen(from, to, &fen);
                move_count += 1
            }
        }
    } else if user_color == board::Color::Black {
        while !game_over::checkmate(&board, color_to_move) {
            if color_to_move == board::Color::White {
                // Bitboards::render_board(&board);
                let mv = search::best_move(&mut board, 5, color_to_move, &fen, &book, move_count)
                    .unwrap();
                apply_move(&mut board, &mv, color_to_move);
                color_to_move = search::opposite(color_to_move);
                let uci = Move::move_to_uci(&mv);
                let (from, to) = uci.split_at(2);
                fen = parse_fen::update_fen(from, to, &fen);
            } else if color_to_move == board::Color::Black {
                let mut input_const;

                let mv = loop {
                    let mut input = String::new();
                    println!("Play your next move! Your color is {:?},", color_to_move);

                    io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to read input");

                    let input = input.trim();

                    if !re.is_match(input) {
                        println!("'{}' is not a valid UCI format, please try again.", input);
                        continue;
                    }
                    input_const = input.to_string().clone();
                    match Move::uci_user_parser(input, &board, color_to_move) {
                        Some(mv) => {
                            println!("'{}' is a valid and legal move", input);
                            break mv; // success, exit loop
                        }
                        _none => {
                            println!(
                                "'{}' has correct format but is not legal here, try again.",
                                input
                            );

                            continue;
                        }
                    }
                };
                // clear_terminal();

                // Bitboards::render_board(&board);
                apply_move(&mut board, &mv, color_to_move);
                color_to_move = search::opposite(color_to_move);
                println!("{:?}", input_const);
                let (from, to) = input_const.split_at(2);
                fen = parse_fen::update_fen(from, to, &fen);
                move_count += 1
            }
        }
    }

    println!("checkmate");
}
