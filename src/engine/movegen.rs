use crate::board::{
    PieceType,
    Color,
    Bitboards
};
use crate::make_move;

// masks
const FILE_A: u64 = 0x0101010101010101;
const FILE_B: u64 = 0x0202020202020202;
const FILE_G: u64 = 0x4040404040404040;
const FILE_H: u64 = 0x8080808080808080;


#[derive(Debug, Clone)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub piece: PieceType,
    pub promotion_rights: bool,
    pub is_castling: bool
}



impl Move {
    pub fn generate_moves_for_piece(sq: u8, piece: PieceType, color: Color, boards: &Bitboards) -> Vec<Move> {
        assert!(sq < 64, "Invalid square index: {}", sq);
        match piece {
            PieceType::Pawn => Self::pawn_moves(sq, color, boards),
            PieceType::Knight => Self::knight_moves(sq, color, boards),
            PieceType::King => Self::king_moves(sq, color, boards),
            PieceType::Rook => Self::rook_moves(sq, color, boards),
            PieceType::Bishop => Self::bishop_moves(sq, color, boards),
            PieceType::Queen => Self::queen_moves(sq, color, boards),

        }
    }


    pub fn generate_moves_for_side(color: Color, boards: &Bitboards) -> Vec<Move> {
    let mut moves = Vec::new();

    for piece in [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
        PieceType::King,
    ] {
        let mut bb = boards.boards[color as usize][piece as usize];
        while bb != 0 {
            let sq = bb.trailing_zeros() as u8;
            bb &= bb - 1; // clear least significant bit

            moves.extend(Self::generate_moves_for_piece(sq, piece, color, boards));
        }
    }

    moves
}



    fn is_square_occupied(bitboard: u64, from: u8, offset: i8) -> bool {
        let target = from as i16 + offset as i16;
        if (0..64).contains(&target) {
            (bitboard & (1u64 << target)) != 0
        } else {
            false
        }
    }

    fn get_own_pieces(bitboards: &Bitboards, color: Color) -> u64 {
        bitboards.boards[color as usize]
            .iter()
            .fold(0u64, |acc, &bb| acc | bb)
    }

    fn get_opponent_pieces(bitboards: &Bitboards, color: Color) -> u64 {
        let opponent = match color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        bitboards.boards[opponent as usize]
            .iter()
            .fold(0u64, |acc, &bb| acc | bb)
    }

    // return board representation of squares that a passed knight can move to
    fn knight_moves(sq: u8, color: Color, board: &Bitboards) -> Vec<Move> {
        let knight= 1u64 << sq;
        let mut moves = 0u64;
        let own_pieces = Self::get_own_pieces(board, color);

        if sq <= 46 && !Self::is_square_occupied(own_pieces, sq, 17) {
            moves |= (knight & !FILE_H) << 17;
        }
        if sq <= 48 && !Self::is_square_occupied(own_pieces, sq,15) {
            moves |= (knight & !(FILE_A))             << 15; // ↑2 ←1
        }
        if sq <= 53 && !Self::is_square_occupied(own_pieces, sq,10) {
            moves |= (knight & !(FILE_G | FILE_H))    << 10; // ↑1 →2
        }
        if sq <= 57 && !Self::is_square_occupied(own_pieces, sq, 6) {
            moves |= (knight & !(FILE_A | FILE_B))    << 6;  // ↑1 ←2
        }
        if sq >= 17 && !Self::is_square_occupied(own_pieces, sq,-17) {
            moves |= (knight & !(FILE_A)) >> 17; // ↓2 ←1;
        }
        if sq >= 15 && !Self::is_square_occupied(own_pieces, sq,-15) {
            moves |= (knight & !(FILE_H))             >> 15; // ↓2 →1
        }
        if sq >= 10 && !Self::is_square_occupied(own_pieces, sq,-10) {
            moves |= (knight & !(FILE_A | FILE_B))    >> 10; // ↓1 ←2
        }
        if sq >= 6 && !Self::is_square_occupied(own_pieces, sq, -6) {
            moves |= (knight & !(FILE_G | FILE_H))    >> 6;  // ↓1 →2
        }

        Self::moves_from_bitboard(sq, PieceType::Knight, moves, false, false)
    }




    fn castling(king_sq: u8, rook_sq: u8, color: Color, board: &Bitboards) -> Vec<Move>{
        let king = 1u64 << king_sq;
        let mut moves = 0u64;
        let my_pieces: u64 = Self::get_own_pieces(board, color);
        let opponent_pieces = Self::get_opponent_pieces(board, color);
        //The approach is not clean and i repeat my self here but as of now i just want it to work.
        //🚨 FIX: Optimize this piece of junk (i know it looks terrible)🚨
        if color == Color::White {
            if !Self::is_square_occupied(my_pieces, 5, 0)        // f1 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 5, 0)   // f1 has no enemy pieces
            && !Self::is_square_occupied(my_pieces, 6, 0)        // g1 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 6, 0)   // g1 has no enemy pieces
            && !make_move::is_square_attacked(board, 4, color)  // e1 not attacked
            && !make_move::is_square_attacked(board, 5, color)  // f1 not attacked
            && !make_move::is_square_attacked(board, 6, color)  // g1 not attacked
            && board.white_kingside
            && rook_sq == 7{
                moves |= king << 2;
            }
            if !Self::is_square_occupied(my_pieces, 1, 0)        // b1 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 1, 0)   // b1 has no enemy pieces
            && !Self::is_square_occupied(my_pieces, 2, 0)        // c1 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 2, 0)   // c1 has no enemy pieces
            && !Self::is_square_occupied(my_pieces, 3, 0)        // d1 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 3, 0)   // d1 has no enemy pieces
            && !make_move::is_square_attacked(board, 4, color)  // e1 not attacked
            && !make_move::is_square_attacked(board, 3, color)  // d1 not attacked
            && !make_move::is_square_attacked(board, 2, color)  // c1 not attacked
            && board.white_queenside
            && rook_sq == 0 {
                moves |= king >> 2;

            }
        } else  {
            if !Self::is_square_occupied(my_pieces, 62, 0)        // g8 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 62, 0)   // g8 has no enemy pieces
            && !Self::is_square_occupied(my_pieces, 61, 0)        // f8 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 61, 0)   // f8 has no enemy pieces
            && !make_move::is_square_attacked(board, 60, color)  // e8 not attacked
            && !make_move::is_square_attacked(board, 61, color)  // f8 not attacked
            && !make_move::is_square_attacked(board, 62, color)  // g8 not attacked
            && board.black_kingside
            && rook_sq == 63{
                moves |= king << 2;
            }
            if !Self::is_square_occupied(my_pieces, 59, 0)        // d8 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 59, 0)   // d8 has no enemy pieces
            && !Self::is_square_occupied(my_pieces, 58, 0)        // c8 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 58, 0)   // c8 has no enemy pieces
            && !Self::is_square_occupied(my_pieces, 57, 0)        // b8 has no friendly pieces
            && !Self::is_square_occupied(opponent_pieces, 57, 0)   // b8 has no enemy pieces
            && !make_move::is_square_attacked(board, 60, color)  // e8 not attacked
            && !make_move::is_square_attacked(board, 59, color)  // d8 not attacked
            && !make_move::is_square_attacked(board, 58, color)  // c8 not attacked
            && board.black_queenside
            && rook_sq == 56{
                moves |= king >> 2;
            }
        }
        //from_sq is a kings position before move as castling is kings move that involves a rook
        Self::moves_from_bitboard(king_sq, PieceType::King, moves, false, true)

    }

    fn king_moves(sq: u8, color: Color, board: &Bitboards) -> Vec<Move> {
        let king = 1u64 << sq;
        let own_pieces = Self::get_own_pieces(board, color);
        let mut all_moves = Vec::new();


        // Generate all possible king moves (8 directions)
        let mut possible_moves = 0u64;
        possible_moves |= (king & !FILE_H) << 1;  // East
        possible_moves |= (king & !FILE_A) >> 1;  // West
        possible_moves |= king << 8;              // North
        possible_moves |= king >> 8;              // South
        possible_moves |= (king & !FILE_H) << 9;  // North-East
        possible_moves |= (king & !FILE_A) << 7;  // North-West
        possible_moves |= (king & !FILE_H) >> 7;  // South-East
        possible_moves |= (king & !FILE_A) >> 9;  // South-West

        // Remove moves to squares occupied by own pieces
        let moves = possible_moves & !own_pieces;


        // Add normal king moves
        all_moves.extend(Self::moves_from_bitboard(sq, PieceType::King, moves, false, false));

        // Add castling moves separately
        if color == Color::White && sq == 4 {
            // all_moves.extend(Self::castling(4, 7, color, board));
            all_moves.extend(Self::castling(4, 0, color, board));
        } else if color == Color::Black && sq == 60 {
            all_moves.extend(Self::castling(60, 63, color, board));
            all_moves.extend(Self::castling(60, 56, color, board));
        }

        all_moves
    }



    fn pawn_moves(sq: u8, color: Color, board: &Bitboards) -> Vec<Move> {
        //Make a bitboard representing all the pieces in the board
        let all_pieces: u64 = Self::get_own_pieces(board, color);
        let opponet_pieces = Self::get_opponent_pieces(board, color);

        let pawn = 1u64 << sq;
        let mut moves = 0u64;
        let promotion: bool;


        if color == Color::White {
            if !Self::is_square_occupied(all_pieces, sq, 8) { //check if the square above a pawn is not occupied
                moves |= pawn << 8; // one-square move
                //check if pawn is on the starting posision to make two-square move
                if !Self::is_square_occupied(all_pieces, sq,16)
                && sq > 7 && sq < 16 {
                    moves |= pawn << 16; // does two-square move
                }
            }
            if Self::is_square_occupied(opponet_pieces, sq, 9) {
                moves |= (pawn & !FILE_H) << 9;
            }
            if Self::is_square_occupied(opponet_pieces, sq, 7) {
                moves |= (pawn & !FILE_A) << 7;
            }


        } else {
            if !Self::is_square_occupied(all_pieces, sq, -8) { //check if the square above a pawn is not occupied
                moves |= pawn >> 8; // one-square move
                //check if pawn is on the starting posision to make two-square move
                if !Self::is_square_occupied(all_pieces, sq, -16)
                && sq > 47 && sq < 56 {
                    moves |= pawn >> 16; // does two-square move
                }
            }
            if Self::is_square_occupied(opponet_pieces, sq, -9) {
                moves |= (pawn & !FILE_H) >> 9; // Takes on diagonal to the east
            }
            if Self::is_square_occupied(opponet_pieces, sq, -7) {
                moves |= (pawn & !FILE_A) >> 7; //Takes on diagonal to the west
            }

        }



        if let Some(ep_square) = board.en_passant_square {
            let sq_i = sq as i16;
            let ep_i = ep_square as i16;

            if (ep_i - sq_i).abs() == 1 {
                let ep_capture_square = if color == Color::White {
                    ep_square - 8
                } else {
                    ep_square + 8
                };
                if ep_capture_square < 64 {
                    moves |= 1u64 << ep_capture_square;
                }
            }
        }



        if  (color == Color::White && sq >= 48 && sq <= 55) ||
            (color == Color::Black && sq >= 8 && sq <= 15) {
                promotion = true;
            } else {
                promotion = false;
            }

        Self::moves_from_bitboard(sq, PieceType::Pawn, moves, promotion, false)
    }

    pub fn rook_moves(sq: u8, color: Color, board: &Bitboards) -> Vec<Move> {
        let mut moves = 0u64;
        let own_pieces = Self::get_own_pieces(board, color);
        let enemy_pieces = Self::get_opponent_pieces(board, color);

        // NORTH
        let mut next_sq = sq + 8;
        while next_sq <= 63 {
            if Self::is_square_occupied(own_pieces, next_sq, 0) {
                break;
            }
            moves |= 1u64 << next_sq;
            if Self::is_square_occupied(enemy_pieces, next_sq, 0) {
                break;
            }
            next_sq += 8;
        }

        // SOUTH
        let mut next_sq = sq.wrapping_sub(8);
        while next_sq <= 63 {
            if Self::is_square_occupied(own_pieces, next_sq, 0) {
                break;
            }
            moves |= 1u64 << next_sq;
            if Self::is_square_occupied(enemy_pieces, next_sq, 0) {
                break;
            }
            if next_sq < 8 { break; }
            next_sq -= 8;
        }

        // EAST

        let mut next_sq = sq + 1;
        while next_sq % 8 != 0 {
            if Self::is_square_occupied(own_pieces, next_sq, 0) {
                break;
            }
            moves |= 1u64 << next_sq;
            if Self::is_square_occupied(enemy_pieces, next_sq, 0) {
                break;
            }
            next_sq += 1;
        }

        // WEST
        let mut next_sq = sq.wrapping_sub(1);
        while sq % 8 != 0 && next_sq % 8 != 7 {
            if Self::is_square_occupied(own_pieces, next_sq, 0) {
                break;
            }
            moves |= 1u64 << next_sq;
            if Self::is_square_occupied(enemy_pieces, next_sq, 0) {
                break;
            }
            if next_sq == 0 { break; }
            next_sq -= 1;
        }

        Self::moves_from_bitboard(sq, PieceType::Rook, moves, false, false)
    }



    pub fn bishop_moves(sq: u8, color: Color, board: &Bitboards) -> Vec<Move> {
        let mut moves = 0u64;

        let own_pieces = Self::get_own_pieces(board, color);
        let enemy_pieces = Self::get_opponent_pieces(board, color);

        // Northeast (+9)
        let mut pos = sq;
        while pos < 56 && (1u64 << pos) & FILE_H == 0 {
            pos += 9;
            if pos >= 64 {
                break;
            }
            if Self::is_square_occupied(own_pieces, pos, 0) {
                break;
            }
            moves |= 1u64 << pos;
            if Self::is_square_occupied(enemy_pieces, pos, 0) {
                break;
            }
            if pos % 8 == 7 { break; } // prevent edge wrapping
        }

        // Northwest (+7)
        pos = sq;
        while pos < 56 && (1u64 << pos) & FILE_A == 0 {
            pos += 7;
            if pos >= 64 {
                break;
            }
            if Self::is_square_occupied(own_pieces, pos, 0) {
                break;
            }
            moves |= 1u64 << pos;
            if Self::is_square_occupied(enemy_pieces, pos, 0) {
                break;
            }
            if pos % 8 == 0 { break; } // prevent edge wrapping
        }

        // Southeast (-7)
        pos = sq;
        while pos >= 7 && (1u64 << pos) & FILE_H == 0 {
            pos = pos.wrapping_sub(7);
            if Self::is_square_occupied(own_pieces, pos, 0) {
                break;
            }
            moves |= 1u64 << pos;
            if Self::is_square_occupied(enemy_pieces, pos, 0) {
                break;
            }
            if pos % 8 == 7 { break; }
            if pos < 7 { break; }
        }

        // Southwest (-9)
        pos = sq;
        while pos >= 9 && (1u64 << pos) & FILE_A == 0 {
            pos = pos.wrapping_sub(9);
            if Self::is_square_occupied(own_pieces, pos, 0) {
                break;
            }
            moves |= 1u64 << pos;
            if Self::is_square_occupied(enemy_pieces, pos, 0) {
                break;
            }
            if pos % 8 == 0 { break; }
            if pos < 9 { break; }
        }

        Self::moves_from_bitboard(sq, PieceType::Bishop, moves, false, false)
    }



    fn queen_moves(sq: u8, color: Color, board: &Bitboards) -> Vec<Move> {
        let rook_moves = Move::rook_moves(sq, color, board)
            .into_iter()
            .map(|mut m| {
                m.piece = PieceType::Queen;
                m
            });

        let bishop_moves = Move::bishop_moves(sq, color, board)
            .into_iter()
            .map(|mut m| {
                m.piece = PieceType::Queen;
                m
            });

        rook_moves.chain(bishop_moves).collect()
    }



    fn moves_from_bitboard(
        from_sq: u8,
        piece: PieceType,
        destinations: u64,
        promotion_rights: bool,
        is_castling: bool
        ) -> Vec<Move> {

        let mut moves_vec = Vec::new();
        let mut bitboard_copy = destinations;

        while bitboard_copy != 0 {
            let to_square = bitboard_copy.trailing_zeros() as u8;
            bitboard_copy &= bitboard_copy - 1;

            let m = Move {
                from: from_sq,
                to: to_square,
                piece,
                promotion_rights: promotion_rights,
                is_castling: is_castling

            };

            moves_vec.push(m);
        }

        moves_vec
    }



}
