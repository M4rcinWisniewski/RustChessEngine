use crate::board::{
    PieceType,
    Color,
    Bitboards  
};
use crate::movegen::Move;


fn is_square_attacked(board: &Bitboards, sq: u8, color: Color)-> bool{
    let enemy_color = match color {
    Color::White => Color::Black,
    Color::Black => Color:: White
    };

    let piece_types: [PieceType; 6] = [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
        PieceType::King,
    ];


    for &piece_type in &piece_types {
        let bitboard = board.boards[enemy_color as usize][piece_type as usize];
        let mut pieces = bitboard;

            while pieces != 0 {
                let from = pieces.trailing_zeros() as u8;
                pieces &= pieces - 1;

                let moves = Move::generate_moves_for_piece(from, piece_type, enemy_color, board);
                for mv in moves {
                    if mv.to == sq {
                        return true;
                    }
                }
            }
    }
    false
}

fn is_valid_square(sq: u8) -> bool {
    sq < 64
}

fn apply_move(board: &mut Bitboards, mv: &Move, color: Color) {
    assert!(mv.from < 64, "mv.from out of bounds: {}", mv.from);
    assert!(mv.to < 64, "mv.to out of bounds: {}", mv.to);
    let from_mask = 1u64 << mv.from;
    let to_mask = 1u64 << mv.to;
    if !is_valid_square(mv.from) || !is_valid_square(mv.to) {
        panic!("Invalid move: {:?}", mv);
    }

    
    // Remove piece from source square
    board.boards[color as usize][mv.piece as usize] &= !from_mask;

    // If it's a capture, remove enemy piece from destination square
    let enemy_color = match color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };
    for piece_type in 0..6 {
        if board.boards[enemy_color as usize][piece_type] & to_mask != 0 {
            board.boards[enemy_color as usize][piece_type] &= !to_mask;
            break;
        }
    }

    // Handle promotion
    if mv.promotion_rights {
        board.boards[color as usize][PieceType::Queen as usize] |= to_mask; //for simplicity it can only promote to Queen for now
    } else {
        // Regular move
        board.boards[color as usize][mv.piece as usize] |= to_mask;
    }

    /* !! TODO: Castiling !! */

}


pub fn make_safe_move(board: &mut Bitboards, mv: &Move, color: Color) -> bool {
    // Store the original board state
    let original_boards = board.boards;
    
    apply_move(board, mv, color);
    let king = Bitboards::get_piece_squares(
        Bitboards::get_single_bit_board(board, PieceType::King, color)
    )[0];
    
    if !is_square_attacked(board, king, color) {
        // Move is legal, keep the new board state
        true
    } else {
        // Move is illegal, restore original board state
        board.boards = original_boards;
        false
    }
}
