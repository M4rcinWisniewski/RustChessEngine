use crate::engine::board::Color;

pub fn parse_fen(fen: &str) -> String {
    
    let mut ranks: Vec<String> = vec![];

    for rank in fen.split(' ').next().unwrap().split('/') {
        let mut rank_str = String::new();
        for c in rank.chars() {
            if c.is_ascii_digit() {
                let count = c.to_digit(10).unwrap();
                rank_str.extend(std::iter::repeat('.').take(count as usize));
            } else {
                rank_str.push(c);
            }
        }
        ranks.push(rank_str);
    }


    ranks.reverse();
    let board_representation = ranks.concat();
    board_representation
}

pub fn flat_board_to_fen(flat_board: &str) -> String {
    let mut fen = String::new();
    let mut empty_count = 0;

    for (i, c) in flat_board.chars().enumerate() {
        if c == '.' {
            empty_count += 1;
        } else {
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
                empty_count = 0;
            }
            fen.push(c);
        }

        
        if (i + 1) % 8 == 0 {
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
                empty_count = 0;
            }
            if i != flat_board.len() - 1 {
                fen.push('/');
            }
        }
    }

    let mut ranks: Vec<&str> = fen.split('/').collect();
    ranks.reverse();
    ranks.join("/")
}



pub fn side_to_move(fen: &str) -> Option<Color> {
    let fields: Vec<&str> = fen.split_whitespace().collect();
    if fields.len() < 2 {
        return None; // invalid FEN
    }
    match fields[1] {
        "w" => Some(Color::White),
        "b" => Some(Color::Black),
        _ => None, // invalid value
    }
}

pub fn update_fen(from: &str, to: &str, fen: &str) -> String {
    let board_part = fen.split_whitespace().next().unwrap();


    let mut board: Vec<Vec<char>> = board_part
        .split('/')
        .map(|rank| {
            let mut row = Vec::new();
            for c in rank.chars() {
                if c.is_digit(10) {
                    for _ in 0..c.to_digit(10).unwrap() {
                        row.push('.');
                    }
                } else {
                    row.push(c);
                }
            }
            row
        })
        .collect();

    let square_to_idx = |sq: &str| -> (usize, usize) {
        let file = (sq.chars().nth(0).unwrap() as u8 - b'a') as usize;
        let rank = (sq.chars().nth(1).unwrap().to_digit(10).unwrap() - 1) as usize;
        (7 - rank, file) 
    };

    let (from_row, from_col) = square_to_idx(from);
    let (to_row, to_col) = square_to_idx(to);


    let piece = board[from_row][from_col];
    board[from_row][from_col] = '.';
    board[to_row][to_col] = piece;

    let mut new_fen_parts = Vec::new();
    for row in board {
        let mut row_str = String::new();
        let mut empty_count = 0;
        for c in row {
            if c == '.' {
                empty_count += 1;
            } else {
                if empty_count > 0 {
                    row_str.push_str(&empty_count.to_string());
                    empty_count = 0;
                }
                row_str.push(c);
            }
        }
        if empty_count > 0 {
            row_str.push_str(&empty_count.to_string());
        }
        new_fen_parts.push(row_str);
    }

    new_fen_parts.join("/")
}
