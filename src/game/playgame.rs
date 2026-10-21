use regex::Regex;
use std::io;
use std::collections::HashMap;
use crate::engine::board::{Bitboards, Color};
use crate::opening_book::book::MoveEntry;
use crate::engine::game_over;
use crate::engine::make_move::apply_move;
use crate::engine::movegen::Move;
use crate::engine::parse_fen;
use crate::engine::search;
pub fn play_game(
    mut board: Bitboards, 
    mut color_to_move: Color, 
    user_color: Color, 
    mut fen: String, 
    book: &HashMap<String, HashMap<String, MoveEntry>>
) {
    let re = Regex::new(r"^[a-h][1-8][a-h][1-8][qrbn]?$").unwrap();
    let mut move_count = 0;

    while !game_over::checkmate(&board, color_to_move) {
        if color_to_move != user_color {
            // engines move 
            let mv = search::best_move(&mut board, 5, color_to_move, &fen, book, move_count).unwrap();
            apply_move(&mut board, &mv, color_to_move);
            color_to_move = search::opposite(color_to_move);
            let uci = Move::move_to_uci(&mv);
            let (from, to) = uci.split_at(2);
            fen = parse_fen::update_fen(from, to, &fen);
        } else {
            // users move
            let mut input_const;
            let mv = loop {
                let mut input = String::new();
                println!("Play your next move! Your color is {:?},", color_to_move);

                io::stdin().read_line(&mut input).expect("Failed to read input");
                let input = input.trim();

                if !re.is_match(input) {
                    println!("'{}' is not a valid UCI format, please try again.", input);
                    continue;
                }
                input_const = input.to_string().clone();
                
                match Move::uci_user_parser(input, &board, color_to_move) {
                    Some(mv) => {
                        println!("'{}' is a valid and legal move", input);
                        break mv; 
                    }
                    None => {
                        println!("'{}' has correct format but is not legal here, try again.", input);
                        continue;
                    }
                }
            };
            
            apply_move(&mut board, &mv, color_to_move);
            color_to_move = search::opposite(color_to_move);
            let (from, to) = input_const.split_at(2);
            fen = parse_fen::update_fen(from, to, &fen);
            move_count += 1;
        }
    }
    println!("checkmate");
}
