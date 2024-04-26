use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::ops::Div;
use serde::Serialize;

use crate::board::Board;
use crate::playing::Games;
use crate::playing::Result::{DRAW, LOOSE, WIN};

mod roll;
mod board;
mod playing;

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

    pub fn calculate(&self) -> u16 {
        self.total.div(self.used as u128) as u16
    }
}

fn main() {
    let mut win_weights: HashMap<String, Weight> = HashMap::new();
    let board = Board::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9], 0);

    let mut counter = 0u32;
    loop {
        let (game_one, game_two) = playing::run_game(&board);

        let game_one_value;
        match game_one.result {
            WIN => { game_one_value = 1000 }
            LOOSE => { game_one_value = 0 }
            DRAW => { game_one_value = 500 }
        }

        let game_two_value;
        match game_two.result {
            WIN => { game_two_value = 1000 }
            LOOSE => { game_two_value = 0 }
            DRAW => { game_two_value = 500 }
        }

        update_weights(game_one, game_one_value, &mut win_weights);
        update_weights(game_two, game_two_value, &mut win_weights);

        counter += 1;
        if counter % 100000 == 0 {
            println!("{}", counter);
        }

        if counter % 10000000 == 0 {
            let file = File::create("computed_weights").expect("Should be able to create file.");
            let writer = BufWriter::new(file);

            serde_yaml::to_writer(writer, &win_weights).expect("Should be able to write data to file.");
            return;
        }
    }
}

fn update_weights(game: Games, value: u128, win_weights: &mut HashMap<String, Weight>) {
    let mut move_sequence = String::new();
    for game_move in game.moves {
        move_sequence += game_move.to_string().as_str();

        let mut weight;
        if !win_weights.contains_key(&move_sequence) {
            weight = Weight::new()
        }
        else {
            weight = *win_weights.get(&move_sequence).expect("The map will contain this value");
        }

        weight.inc(value);
        win_weights.insert(move_sequence.clone(), weight);
    }
}