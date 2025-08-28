mod engine;
use engine::{
    board::{
        self,
        Bitboards,
    },
    parse_fen,
    movegen,
    make_move,
    evaluation,
    search
};
use clap::{Parser};
#[derive(Parser, Debug)]
struct Args {
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

    let color_to_move = parse_fen::side_to_move(&args.fen).unwrap();
    let best_next_move = search::best_move(&mut board, 3, color_to_move);
    println!("{:#?}", best_next_move);
    



}
