use rust_chess::engine;
use rust_chess::game;
use rust_chess::engine::board::{self, Bitboards};
use rust_chess::engine::movegen::Move;
use rust_chess::opening_book::book;
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
};
use std::io::stdout;
use clap::Parser;
use rust_chess::engine::parse_fen;
use indicatif::{ProgressBar, ProgressStyle};
mod opening_book;
use std::collections::HashMap;

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

fn _clear_terminal() {
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
    let color_to_move = parse_fen::side_to_move(&args.fen).unwrap();
    let fen = parse_fen::flat_board_to_fen(&fen_position);
    let _moves = Move::generate_moves_for_side(board::Color::White, &board);
    
    //prints all found moves (debug tool)
    //for m in &moves {
    //    println!("Found Move: {:?}", m);
    //}

    game::playgame::play_game(board, color_to_move, user_color, fen, &book);

    println!("checkmate");
}
