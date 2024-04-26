use std::hash::Hash;

use weighted_rand::builder::*;

use crate::board::Board;
use crate::playing::Result::{DRAW, LOOSE, WIN};

pub struct Games {
    pub moves: Vec<u16>,
    pub result: Result
}

impl Games {
    pub fn new(moves: Vec<u16>, result: Result) -> Games {
        Games{ moves, result }
    }
}

pub enum Result {
    WIN,
    LOOSE,
    DRAW
}

pub fn run_game(board: &Board) -> (Games, Games) {
    let first_moves = rand(board, Vec::new());
    let second_moves = rand(board, Vec::new());

    let mut first = Games::new(first_moves.1, WIN);
    let mut second = Games::new(second_moves.1, LOOSE);

    if first_moves.0 > second_moves.0 {
        first.result = WIN;
        second.result = LOOSE

    } else if second_moves.0 > first_moves.0 {
        first.result = LOOSE;
        second.result = WIN

    } else {
        first.result = DRAW;
        second.result = DRAW
    }

    (first, second)
}

fn rand(board: &Board, mut moves: Vec<u16>) -> (u8, Vec<u16>) {
    let rand_move = board.get_rand_roll();
    moves.push(rand_move.1);

    return match rand_move.0.get_rand_board() {
        None => {
            (board.calculate_value(), moves)
        }
        Some(board) => {
            moves.push(board.1);
            rand(board.0, moves)
        }
    }

}

