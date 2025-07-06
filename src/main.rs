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
    make_move
};
use clap::{Parser};





#[derive(Parser, Debug)]
#[command(name = "greeter")]
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
    let pawns = board.boards[Color::White as usize][PieceType::Pawn as usize];
    let mut pawn_positions = pawns;
    let mut all_moves = Vec::new();

    while pawn_positions != 0 {
        let sq = pawn_positions.trailing_zeros() as u8;
        pawn_positions &= pawn_positions - 1; // clear the least significant 1 bit

        let moves = movegen::Move::generate_moves_for_piece(
            sq,
            PieceType::Pawn,
            Color::White,
            &board,
        );

        all_moves.extend(moves);
    }


    println!("{:#?}", &all_moves[6]);
    make_move::apply_move(&mut board, &all_moves[6], board::Color::White);
    Bitboards::_print_board(board.boards[Color::White as usize][PieceType::Pawn as usize]);

}
