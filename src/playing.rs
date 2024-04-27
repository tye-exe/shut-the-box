use std::hash::Hash;

use weighted_rand::builder::*;
use serde::Serialize;
use std::ops::Div;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::fs::File;
use std::io::BufWriter;

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


#[derive(Debug, Copy, Clone, Serialize)]
pub struct Weight {
    total: u128,
    used: u32,
}

impl Weight {
    pub fn new() -> Weight {
        Weight { total: 0, used: 0 }
    }

    pub fn inc(&mut self, amount: u128) {
        self.total += amount;
        self.used += 1;
    }

    pub fn combine(&mut self, other: &Weight) {
        self.total += other.total;
        self.used += other.used;
    }

    pub fn calculate(&self) -> u16 {
        self.total.div(self.used as u128) as u16
    }
}


#[derive(Copy, Clone)]
pub enum Result {
    WIN = 10,
    LOOSE = 5,
    DRAW = 0
}


pub fn run_game(board: &Board) -> (Games, Games) {
    let first_moves = rand(board, Vec::new());
    let second_moves = rand(board, Vec::new());

    let mut first = Games::new(first_moves.1, DRAW);
    let mut second = Games::new(second_moves.1, DRAW);

    if first_moves.0 > second_moves.0 {
        first.result = WIN;
        second.result = LOOSE

    } else if second_moves.0 > first_moves.0 {
        first.result = LOOSE;
        second.result = WIN

    }
    // If it's a draw then it can just use the default values.

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


pub fn compute_weights() {
    let mut win_weights: HashMap<String, Weight> = HashMap::new();
    let (tx, rx) = mpsc::channel();

    // Creates 8 threads to compute random simulations of the game.
    for _ in 0..8 {
        let tx_thread = tx.clone();

        thread::spawn(move || {
            let board = Board::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9], 0);
            let mut win_weights: HashMap<String, Weight> = HashMap::new();

            for _ in 0..100000 {
                let (game_one, game_two) = run_game(&board);

                update_weights(&game_one, game_one.result as u128, &mut win_weights);
                update_weights(&game_two, game_two.result as u128, &mut win_weights);
            }

            tx_thread.send(win_weights).expect("Should be able to send.");
        });
    }

    // Waits for each thread to finish & merges its results into the main map.
    for finished_threads in 0..8 {
        let mut thread_map = rx.recv().expect("Should always receive a value");

        for key in thread_map.keys() {
            if !win_weights.contains_key(key) {
                win_weights.insert(key.clone(), *thread_map.get(key).expect("Will exist."));
                continue;
            }

            let existing_weight = win_weights.get_mut(key).expect("Will exist.");
            let thread_weight = thread_map.get(key).expect("Will exist.");
            existing_weight.combine(thread_weight);
        }

        println!("{}", finished_threads);
    }

    //
    // let file = File::create("computed_weights").expect("Should be able to create file.");
    // let writer = BufWriter::new(file);
    //
    // serde_yaml::to_writer(writer, &win_weights).expect("Should be able to write data to file.");
}

fn update_weights(game: &Games, value: u128, win_weights: &mut HashMap<String, Weight>) {
    let mut move_sequence = String::new();
    for game_move in &game.moves {
        move_sequence += game_move.to_string().as_str();

        let mut weight;
        if !win_weights.contains_key(&move_sequence) {
            weight = Weight::new()
        } else {
            weight = *win_weights.get(&move_sequence).expect("The map will contain this value");
        }

        weight.inc(value);
        win_weights.insert(move_sequence.clone(), weight);
    }
}

