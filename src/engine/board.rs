#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Color {
    White,
    Black,
}


#[derive(Copy, Clone, Debug)]
pub enum PieceType {
    Pawn = 0,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Default)]
pub struct Bitboards {
    pub boards: [[u64; 6]; 2], 
}

    /*the Bitboards structs single bitboard representation:
    a8 b8 c8 d8 e8 f8 g8 h8   ← bits 56–63
    a7 b7 c7 d7 e7 f7 g7 h7   ← bits 48–55
    a6 b6 c6 d6 e6 f6 g6 h6   ← bits 40–47
    a5 b5 c5 d5 e5 f5 g5 h5   ← bits 32–39
    a4 b4 c4 d4 e4 f4 g4 h4   ← bits 24–31
    a3 b3 c3 d3 e3 f3 g3 h3   ← bits 16–23
    a2 b2 c2 d2 e2 f2 g2 h2   ← bits 8–15
    a1 b1 c1 d1 e1 f1 g1 h1   ← bits 0–7
    
    Numbers equivalent to spcific bit:

    56  57  58  59  60  61  62  63
    48  49  50  51  52  53  54  55 
    40  41  42  43  44  45  46  47  
    32  33  34  35  36  37  38  39  
    24  25  26  27  28  29  30  31
    16  17  18  19  20  21  22  23
    8   9   10  11  12  13  14  15  
    0   1   2   3   4   5   6   7

    so a1 is 0 and f6 is 45
    these are made for all white and black pieces
    */


impl Bitboards {
    pub fn new() -> Self {
        Self {
            boards: [[0u64; 6]; 2],  // boards[color][piece] keeps 12 bit boards
        }
    }
    //returns a single selected bitboard
    pub fn _get_single_bit_board(&self, piece: PieceType, color: Color) -> u64 {
        self.boards[color as usize][piece as usize]

    }
    //adds piece to bit board
    pub fn add_piece(bitboards: &mut Bitboards, color: Color, piece: PieceType, square: u8) {
        let bb = &mut bitboards.boards[color as usize][piece as usize];
        *bb |= 1 << square;
    }
    //prints bit board
    pub fn _print_board(bitboard: u64) {
        println!("   a b c d e f g h");
        for rank in (0..8).rev() {
            print!("{}  ", rank + 1);
            for file in 0..8 {
                let square = rank * 8 + file;
                let bit = (bitboard >> square) & 1;
                print!("{} ", if bit == 1 { "1" } else { "." });
            }
            println!();
        }
        println!();
    }
    //Returns all squares occupied by a selected piece 
    pub fn _get_piece_squares(bitboard: u64) -> Vec<u8> {
        let mut squares = Vec::new();
        for i in 0..64 {
            if (bitboard >> i) & 1 == 1 {
                squares.push(i);
            }
        }
        squares
    }
    // Displayes all bit boards
    pub fn _display(&self) {
        for color_index in 0..2 {
            for piece_index in 0..6 {
                let bb = self.boards[color_index][piece_index];
                let color = if color_index == 0 { "White" } else { "Black" };
                let piece = match piece_index {
                    0 => "Pawn",
                    1 => "Rook",
                    2 => "Knight",
                    3 => "Bishop",
                    4 => "Queen",
                    5 => "King",
                    _ => unreachable!(),
                };

                println!("{} {}:", color, piece);
                Self::_print_board(bb);
            }
        }
        
    }

}