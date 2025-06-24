use crate::board::PieceType;
use crate::board::Color;

pub struct Move {
    from: u8,
    to: u8,
    piece: PieceType,
    promotion: Option<PieceType>,
    is_capture: bool,
    flags: u8,
}


impl Move {

    pub fn generate_moves_for_piece(sq: u8, piece: PieceType, color: Color) -> Vec<Move> {
    match piece {
        PieceType::Pawn => Self::pawn_moves(sq, color),
        PieceType::Knight => Self::knight_moves(sq),
        PieceType::King => Self::king_moves(sq),
        _ => Vec::new(), // default empty for unimplemented
    }
    }



    // return board representation of squares that a passed knight can move to
    fn knight_moves(sq: u8) -> Vec<Move> { 
        let knight= 1u64 << sq;

        const FILE_A: u64 = 0x0101010101010101;
        const FILE_B: u64 = 0x0202020202020202;
        const FILE_G: u64 = 0x4040404040404040;
        const FILE_H: u64 = 0x8080808080808080;

        let mut moves = 0u64;

        moves |= (knight & !(FILE_H))             << 17; // ↑2 →1
        moves |= (knight & !(FILE_A))             << 15; // ↑2 ←1
        moves |= (knight & !(FILE_G | FILE_H))    << 10; // ↑1 →2
        moves |= (knight & !(FILE_A | FILE_B))    << 6;  // ↑1 ←2

        moves |= (knight & !(FILE_A))             >> 17; // ↓2 ←1
        moves |= (knight & !(FILE_H))             >> 15; // ↓2 →1
        moves |= (knight & !(FILE_A | FILE_B))    >> 10; // ↓1 ←2
        moves |= (knight & !(FILE_G | FILE_H))    >> 6;  // ↓1 →2

        Self::moves_from_bitboard(sq, PieceType::Knight, moves)
        }

    fn king_moves(sq: u8) -> Vec<Move> {
        let king = 1u64 << sq;

        const FILE_A: u64 = 0x0101010101010101;
        const FILE_H: u64 = 0x8080808080808080;
        
        let mut moves = 0u64;

        // Horizontal moves
        moves |= (king & !FILE_H) << 1;  // East
        moves |= (king & !FILE_A) >> 1;  // West

        // Vertical moves
        moves |= king << 8;             // North
        moves |= king >> 8;             // South

        // Diagonal moves
        moves |= (king & !FILE_H) << 9;  // North-East
        moves |= (king & !FILE_A) << 7;  // North-West
        moves |= (king & !FILE_H) >> 7;  // South-East
        moves |= (king & !FILE_A) >> 9;  // South-West


        Self::moves_from_bitboard(sq, PieceType::King, moves)
    }

    fn pawn_moves(sq: u8, color: Color) -> Vec<Move> {
        let pawn = 1u64 << sq;
        const FILE_A: u64 = 0x0101010101010101;
        const FILE_H: u64 = 0x8080808080808080;

        let mut moves = 0u64;

        // Generate pawn moves as before, into bitboard:
        if color == Color::White {
            if sq >= 8 && sq <= 15 {
                moves |= pawn << 16; // two-square move
            }
            moves |= pawn << 8; // one-square move
            moves |= (pawn & !FILE_H) << 9; // capture west diagonal
            moves |= (pawn & !FILE_A) << 7; // capture east diagonal
        } else {
            if sq >= 48 && sq <= 55 {
                moves |= pawn >> 16; // two-square move
            }
            moves |= pawn >> 8; // one-square move
            moves |= (pawn & !FILE_H) >> 7; // capture west diagonal
            moves |= (pawn & !FILE_A) >> 9; // capture east diagonal
        }

        Self::moves_from_bitboard(sq, PieceType::Pawn, moves)
    }


 fn moves_from_bitboard(
    from_sq: u8,
    piece: PieceType,
    destinations: u64,
    // You can add more parameters like captures, promotions, flags here later
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
            promotion: None,
            is_capture: false,
            flags: 0,
        };

        moves_vec.push(m);
    }

    moves_vec
}
}

