use crate::board::{
    Bitboards,
    Color,
    PieceType
};
use crate::movegen::Move;

// mobility weight gives mobility a proper weigth in final eval
const MOBILITY_WEIGHT: i32= 5;

// Piece-square table for knight
const KNIGHT_PST: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];


fn mirror_square(square: u8) -> u8 {
    let rank = square / 8;
    let file = square % 8;
    (7 - rank) * 8 + file
}


//Very simple evaluation function. Will be improved in the future
pub fn evaluation(board: &Bitboards, color: Color) -> i32 {

    /* MATERIAL SCORE */
    let mut friendly_score = 0i32;
    let mut enemy_score = 0i32;

    // following: pawn, rook, knight, bishop, queen
    let piece_values = [100, 500, 320, 330, 900];
    for i in 0..piece_values.len() {
        let friendly_pieces = Bitboards::count_pieces(board.boards[0][i]);
        let enemy_pieces = Bitboards::count_pieces(board.boards[1][i]);
        friendly_score += friendly_pieces * piece_values[i];
        enemy_score +=  enemy_pieces * piece_values[i];

    }

    let material_score = friendly_score - enemy_score;


    /* MOBILITY SCORE */
    let mut friendly_moves = 0i32;
    let mut enemy_moves = 0i32;
    let piece_mobility_weights = [
        0,   // Pawn
        4,   // Knight
        4,   // Bishop
        5,   // Rook
        9,   // Queen
        0,   // King
    ];

    for i in 0..PieceType::pieces().len() {
        let piece = PieceType::pieces()[i];
        let friendly_squares = Bitboards::return_squares(board.boards[0][i]);
        for sq in friendly_squares {
           let possible_moves = Move::generate_moves_for_piece(sq, piece, Color::White, board);
           friendly_moves += piece_mobility_weights[i] * possible_moves.len() as i32 ;
        }
    
        let enemy_squares = Bitboards::return_squares(board.boards[1][i]);
        for sq in enemy_squares {
            let possible_moves = Move::generate_moves_for_piece(sq, piece, Color::Black, board);
            enemy_moves += piece_mobility_weights[i] * possible_moves.len() as i32 ;
        }
    }

    let mobility_score = friendly_moves - enemy_moves;
    // println!("{}", friendly_moves);
    // println!("{}", enemy_moves);

   
    // Knights PST
    let mut pst_score = 0;

    for sq in Bitboards::return_squares(board.boards[0][1]) {
        pst_score += KNIGHT_PST[sq as usize];
        
    }

    for sq in Bitboards::return_squares(board.boards[1][1]) {
        let mirrored = mirror_square(sq);
        pst_score -= KNIGHT_PST[mirrored as usize];
    }

    let eval = material_score + mobility_score * MOBILITY_WEIGHT + pst_score;
    eval
}
