use crate::board::Bitboards;

//Very simple evaluation function. Will be improved in the future
pub fn evaluation(board: Bitboards) -> i32 {
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

    friendly_score - enemy_score
}
