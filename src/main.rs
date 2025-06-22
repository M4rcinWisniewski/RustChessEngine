mod board;
mod parse_fen;
use board::Bitboards;

/* 
Fen notation example for reference:
rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
*/

fn main() {
    let mut board = Bitboards::new();

    let fen_position = parse_fen::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    
    for (i, n) in fen_position.chars().enumerate() {
        if n == '.' {
            continue;
        }

        let square = i as u8;

        let color = if n.is_uppercase() {
            board::Color::White
        } else {
            board::Color::Black
        };

        let piece_type = match n.to_ascii_uppercase() {
            'P' => board::PieceType::Pawn,
            'R' => board::PieceType::Rook,
            'N' => board::PieceType::Knight,
            'B' => board::PieceType::Bishop,
            'Q' => board::PieceType::Queen,
            'K' => board::PieceType::King,
            _ => unreachable!(),
        };

        Bitboards::add_piece(&mut board, color, piece_type, square);
}

    println!("{}", fen_position);
    board.display();

}
