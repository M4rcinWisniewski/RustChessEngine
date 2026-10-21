use rust_chess::engine::board::{self, Bitboards};
use rust_chess::engine::movegen::Move;
use rust_chess::engine::make_move::apply_move;
use rust_chess::engine::search; 

//Perft function
fn perft(board: &mut Bitboards, color: board::Color, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let moves = rust_chess::engine::make_move::generate_legal_moves(board, color);

    for mv in moves {
        let mut next_board = board.clone();
        apply_move(&mut next_board, &mv, color);

        nodes += perft(&mut next_board, search::opposite(color), depth - 1);    
    }
    nodes
}

#[test]
fn test_perft_initial_position() {
    let mut board = Bitboards::new(); 

    let default_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let fen_position = rust_chess::engine::parse_fen::parse_fen(default_fen);

    for (i, n) in fen_position.chars().enumerate() {
        if !n.is_ascii_alphabetic() {
    continue;
}        let square = i as u8;
        let color = if n.is_lowercase() { board::Color::Black } else { board::Color::White };
        let piece_type = match n.to_ascii_lowercase() {
            'p' => board::PieceType::Pawn,
            'r' => board::PieceType::Rook,
            'n' => board::PieceType::Knight,
            'b' => board::PieceType::Bishop,
            'q' => board::PieceType::Queen,
            'k' => board::PieceType::King,
            _ => unreachable!(),
        };         Bitboards::add_piece(&mut board, color, piece_type, square);
    }

    // 3. Teraz test zadziała, bo plansza ma pionki
    let color = board::Color::White;
    assert_eq!(perft(&mut board, color, 1), 20);
    assert_eq!(perft(&mut board, color, 2), 400);
    assert_eq!(perft(&mut board, color, 3), 8902);
    assert_eq!(perft(&mut board, color, 4), 197281);
}
