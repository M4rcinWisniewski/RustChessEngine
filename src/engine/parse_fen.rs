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
