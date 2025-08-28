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

    // Reverse only the ranks, not each line's characters
    ranks.reverse();
    let board_representation = ranks.concat();
    board_representation
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
