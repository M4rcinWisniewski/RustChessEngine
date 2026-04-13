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
use crossterm::{
    execute,
    style::{
        Attribute, Color as TermColor, Print, ResetColor, SetAttribute, SetBackgroundColor,
        SetForegroundColor,
    },
    terminal::{Clear, ClearType},
};
use std::io::stdout;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Color {
    White,
    Black,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

// i dont know if i really need this
impl PieceType {
    pub fn pieces() -> [PieceType; 6] {
        [
            PieceType::Pawn,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Queen,
            PieceType::King,
        ]
    }
}

#[derive(Default, Debug, Clone)]
pub struct Bitboards {
    pub boards: [[u64; 6]; 2],
    pub en_passant_square: Option<u8>,
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl Bitboards {
    pub fn new() -> Self {
        Self {
            boards: [[0u64; 6]; 2],
            en_passant_square: None,
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }

    pub fn render_board(board: &Bitboards) {
        const PIECES: [[char; 6]; 2] = [
            // White pieces
            ['♟', '♞', '♝', '♜', '♛', '♚'],
            // Black pieces
            ['♟', '♞', '♝', '♜', '♛', '♚'],
        ];

        const SQUARE_WIDTH: usize = 7;
        const SQUARE_HEIGHT: usize = 3;

        let mut squares: [Option<(char, u8)>; 64] = [None; 64];

        for color in 0..2 {
            for piece in 0..6 {
                let mut bb = board.boards[color][piece];
                while bb != 0 {
                    let sq = bb.trailing_zeros() as usize;
                    squares[sq] = Some((PIECES[color][piece], color as u8));
                    bb &= bb - 1;
                }
            }
        }

        let mut out = stdout();
        execute!(out, Clear(ClearType::All)).unwrap();

        for rank in (0..8).rev() {
            for row in 0..SQUARE_HEIGHT {
                if row == SQUARE_HEIGHT / 2 {
                    print!("{} ", rank + 1);
                } else {
                    print!("  ");
                }

                for file in 0..8 {
                    let idx = rank * 8 + file;
                    let is_dark = (rank + file) % 2 == 1;

                    // --- Background colors ---
                    let bg = if is_dark {
                        TermColor::DarkYellow
                    } else {
                        TermColor::Black
                    };

                    if row == SQUARE_HEIGHT / 2 {
                        if let Some((piece, color)) = squares[idx] {
                            let fg = match color {
                                0 => TermColor::White,    // white pieces
                                1 => TermColor::DarkGrey, // black pieces
                                _ => TermColor::White,
                            };
                            let side_padding = (SQUARE_WIDTH - 1) / 2;
                            let cell = format!(
                                "{}{}{}",
                                " ".repeat(side_padding),
                                piece,
                                " ".repeat(side_padding)
                            );
                            execute!(
                                out,
                                SetBackgroundColor(bg),
                                SetForegroundColor(fg),
                                SetAttribute(Attribute::Bold),
                                Print(cell),
                                ResetColor
                            )
                            .unwrap();
                        } else {
                            execute!(
                                out,
                                SetBackgroundColor(bg),
                                Print(" ".repeat(SQUARE_WIDTH)),
                                ResetColor
                            )
                            .unwrap();
                        }
                    } else {
                        execute!(
                            out,
                            SetBackgroundColor(bg),
                            Print(" ".repeat(SQUARE_WIDTH)),
                            ResetColor
                        )
                        .unwrap();
                    }
                }
                println!();
            }
        }

        // --- file letters ---
        print!("   ");
        for file in 0..8 {
            let letter = (b'a' + file as u8) as char;
            let spacing = " ".repeat((SQUARE_WIDTH - 1) / 2);
            print!("{}{}{}", spacing, letter, spacing);
        }
        println!();
    }

    //adds piece to bit board
    pub fn add_piece(bitboards: &mut Bitboards, color: Color, piece: PieceType, square: u8) {
        let bb = &mut bitboards.boards[color as usize][piece as usize];
        *bb |= 1 << square;
    }

    //count pieces on the bitboard
    pub fn count_pieces(bitboard: u64) -> i32 {
        let mut count: i32 = 0;
        for i in 0..64 {
            if (bitboard >> i) & 1 == 1 {
                count += 1
            }
        }
        count
    }

    pub fn return_squares(bitboard: u64) -> Vec<u8> {
        let mut squares = Vec::new();
        for i in 0..64 {
            if (bitboard >> i) & 1 == 1 {
                squares.push(i);
            }
        }
        squares
    }
}
