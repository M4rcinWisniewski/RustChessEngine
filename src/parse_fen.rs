pub fn parse_fen(fen: &str) -> String {
    let mut board_representation: String = String::new();
    for n in fen.chars(){
        if n == ' ' {
            break
        }
        if n == '/' {
           continue;
        }
        if n.is_ascii_digit() {
            let count = n.to_digit(10).unwrap();
            for _ in 0..count {
                board_representation.push('.');
            }
            continue;
        }
        board_representation.push(n);
    }
    board_representation
}