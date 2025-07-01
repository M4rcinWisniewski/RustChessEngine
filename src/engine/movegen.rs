use crate::board::PieceType;
use crate::board::Color;
const FILE_A: u64 = 0x0101010101010101;
const FILE_B: u64 = 0x0202020202020202;
const FILE_G: u64 = 0x4040404040404040;
const FILE_H: u64 = 0x8080808080808080;
#[derive(Debug)]
pub struct Move {
    from: u8,
    to: u8,
    piece: PieceType,
    promotion_rights: bool,
    is_capture: bool,
    flags: u8,
}


impl Move {


    pub fn generate_moves_for_piece(sq: u8, piece: PieceType, color: Color) -> Vec<Move> {
        match piece {
            PieceType::Pawn => Self::pawn_moves(sq, color),
            PieceType::Knight => Self::knight_moves(sq),
            PieceType::King => Self::king_moves(sq),
            PieceType::Rook => Self::rook_moves(sq),
            PieceType::Bishop => Self::bishop_moves(sq),
            PieceType::Queen => Self::queen_moves(sq),
            _ => Vec::new(), // default empty for unimplemented
        }
    }



    // return board representation of squares that a passed knight can move to
    fn knight_moves(sq: u8) -> Vec<Move> { 
        let knight= 1u64 << sq;
        let mut moves = 0u64;

        moves |= (knight & !(FILE_H))             << 17; // ↑2 →1
        moves |= (knight & !(FILE_A))             << 15; // ↑2 ←1
        moves |= (knight & !(FILE_G | FILE_H))    << 10; // ↑1 →2
        moves |= (knight & !(FILE_A | FILE_B))    << 6;  // ↑1 ←2

        moves |= (knight & !(FILE_A))             >> 17; // ↓2 ←1
        moves |= (knight & !(FILE_H))             >> 15; // ↓2 →1
        moves |= (knight & !(FILE_A | FILE_B))    >> 10; // ↓1 ←2
        moves |= (knight & !(FILE_G | FILE_H))    >> 6;  // ↓1 →2

        Self::moves_from_bitboard(sq, PieceType::Knight, moves, false)
        }

    fn king_moves(sq: u8) -> Vec<Move> {
        let king = 1u64 << sq;
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


        Self::moves_from_bitboard(sq, PieceType::King, moves, false)
    }

    fn pawn_moves(sq: u8, color: Color) -> Vec<Move> {
        let pawn = 1u64 << sq;
        let mut moves = 0u64;
        let promotion: bool;

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
        if sq < 56 && sq > 47 && color == Color::White || color == Color::Black && sq > 7 && sq < 16 {
            promotion = true;
        } else  {
            promotion = false;
        }

        Self::moves_from_bitboard(sq, PieceType::Pawn, moves, promotion)
    }

    fn rook_moves(sq: u8) -> Vec<Move> {
        let mut moves = 0u64;
        let mut next_sq = sq + 8;

        while next_sq <= 63 {
            moves |= 1u64 << next_sq;
            next_sq += 8;
        }
        // Number of squares south

        let mut next_sq = sq.wrapping_sub(8);

        while next_sq <= 63 {
            moves |= 1u64 << next_sq;
            // Break if next subtraction would underflow
            if next_sq < 8 {
                break;
            }
            next_sq -= 8;
        }
        
        // Number of squares east

        let mut next_sq = sq;
        while next_sq % 8 != 0 {
            next_sq -= 1;
            moves |= 1u64 << next_sq;
        }

        let mut next_sq = sq;
        while next_sq % 8 != 7 {
            next_sq += 1;
            moves |= 1u64 << next_sq;

        }
        
        Self::moves_from_bitboard(sq, PieceType::Pawn, moves, false)        
    }


    pub fn bishop_moves(sq: u8) -> Vec<Move> {
        let mut moves = 0u64;

        // Direction masks
        const FILE_A: u64 = 0x0101010101010101;
        const FILE_H: u64 = 0x8080808080808080;

        // Northeast (+9)
        let mut pos = sq;
        while pos < 56 && (1u64 << pos) & FILE_H == 0 {
            pos += 9;
            if pos < 64 {
                moves |= 1u64 << pos;
                if (1u64 << pos) & FILE_H != 0 { break; }
            }
        }

        // Northwest (+7)
        pos = sq;
        while pos < 56 && (1u64 << pos) & FILE_A == 0 {
            pos += 7;
            if pos < 64 {
                moves |= 1u64 << pos;
                if (1u64 << pos) & FILE_A != 0 { break; }
            }
        }

        // Southeast (-7)
        pos = sq;
        while pos >= 7 && (1u64 << pos) & FILE_H == 0 {
            pos -= 7;
            moves |= 1u64 << pos;
            if (1u64 << pos) & FILE_H != 0 || pos < 7 { break; }
        }

        // Southwest (-9)
        pos = sq;
        while pos >= 9 && (1u64 << pos) & FILE_A == 0 {
            pos -= 9;
            moves |= 1u64 << pos;
            if (1u64 << pos) & FILE_A != 0 || pos < 9 { break; }
        }

        Self::moves_from_bitboard(sq, PieceType::Pawn, moves, false)
    }



    fn queen_moves(sq: u8) -> Vec<Move> {
        let mut moves = Move::rook_moves(sq);
        moves.extend(Move::bishop_moves(sq));
        moves
    }


    fn moves_from_bitboard(
        from_sq: u8,
        piece: PieceType,
        destinations: u64,
        promotion_rights: bool 
        
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
                is_capture: false,
                flags: 0,
            };

            moves_vec.push(m);
        }

        moves_vec
    }

}

