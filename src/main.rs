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

    

    let cloned_boards = board.boards;
    for mv in all_moves {

        

        if make_move::make_safe_move(&mut board, &mv, board::Color::White) {
            println!("Legal move!");
            let white_pawns_bb = board.get_single_bit_board(PieceType::Pawn, board::Color::White);
            Bitboards::_print_board(white_pawns_bb);
        } else {
            println!("Illegal move - leaves king in check");
        }
        let white_queens = board.get_single_bit_board(PieceType::Queen, board::Color::White);
        println!("Pawn after promotion to Queen");
        Bitboards::_print_board(white_queens);
        // Reset board for next move
        board.boards = cloned_boards;
    }
}
