mod engine;
use engine::{
    board::{
        self,
        Bitboards,
        PieceType,
        Color
    },
    parse_fen,
    movegen,
    make_move,
    evaluation
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

    let king_move = movegen::Move::generate_moves_for_piece(60, PieceType::King, Color::Black, &board);
    println!("{:#?}", king_move);


    let eval = evaluation::evaluation(board);

    println!("{:?}", eval);
}
