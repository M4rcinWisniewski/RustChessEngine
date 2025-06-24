mod board;
mod parse_fen;
mod movegen;
use board::Bitboards;
use movegen::Move;
use clap::Parser;

/// Simple program that greets a person
#[derive(Parser, Debug)]
#[command(name = "greeter")]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")]
    fen: String,
}

fn main() {
    let args = Args::parse();    
    let fen_position = parse_fen::parse_fen(&args.fen);
    
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


let sq = 15u8; // c3
// Move::knight_moves(sq);
    let _moves = Move::generate_moves_for_piece(
        sq, 
        board::PieceType::King, 
        board::Color::White);
 

}
