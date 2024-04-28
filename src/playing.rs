use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::BufWriter;
use std::ops::Div;
use std::sync::mpsc;
use std::thread;

use fastrand::Rng;
use serde::{Serialize, Serializer};

use crate::{get_board, get_rand_board};
use crate::board::Board;
use crate::playing::Result::{DRAW, LOSS, WIN};

/// A wrapper struct to store the moves taken in a game & the result of the game.
pub struct Games {
    pub moves: Vec<Choice>,
    pub result: Result,
}

impl Games {
    pub fn new(moves: Vec<Choice>, result: Result) -> Games {
        Games { moves, result }
    }
}


/// Stores the total value of a choice & the amount of times it was taken.
/// This allows for the division to be performed after, since division is very intensive.
#[derive(Debug, Copy, Clone, Serialize)]
pub struct Weight {
    total: u32,
    used: u32,
}

impl Weight {

    /// Adds the given amount to the weight.
    pub fn inc(&mut self, amount: u32) {
        self.total += amount;
        self.used += 1;
    }

    /// Adds the given weight to this weight.
    pub fn combine(&mut self, other: &Weight) {
        self.total += other.total;
        self.used += other.used;
    }

    /// Calculates the average chance of
    pub fn calculate(&self) -> u16 {
        self.total.div(self.used) as u16
    }
}

/// Stores a possible board that could be "made" from one board state according to a certain roll.
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct Choice {
    root_board: u16,
    roll: u8,
    chosen_board: Option<u16>,
}

impl Choice {
    /// Sets the value of the chosen board
    pub fn set_chosen_board(&mut self, chosen_board: u16) {
        self.chosen_board = Some(chosen_board);
    }

    /// Returns true if the move this choice represents would lead to a game over.
    pub fn is_dying_choice(&self) -> bool {
        self.chosen_board == None
    }
}

impl Serialize for Choice {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
        serializer.collect_str(
            &format!("{}-{}-{:?}", self.root_board, self.roll, self.chosen_board)
        )
    }
}


/// Represents the weight of each simulation outcome.
/// Win = 10
/// Draw = 5
/// Loss = 0
#[derive(Copy, Clone)]
pub enum Result {
    WIN = 10,
    DRAW = 5,
    LOSS = 0,
}


/// The amount of threads to simulate games with.
const THREADS: u8 = 8;

/// The amount of games each thread should simulate.
const GAMES_TO_PLAY: u32 = 500000;

/// Simulates games of shut the box & write the win rates of each move to "computed_weights.yml"
pub fn compute_weights() {
    let mut win_weights: HashMap<Choice, Weight> = HashMap::new();
    let (tx, rx) = mpsc::channel();

    // Creates threads to compute random simulations of the game.
    for _ in 0..THREADS {
        let tx_thread = tx.clone();

        thread::spawn(move || {
            // Each simulation will start from a random board to get an even distribution
            let mut win_weights: HashMap<Choice, Weight> = HashMap::new();

            for _ in 0..GAMES_TO_PLAY {
                let board = get_rand_board();
                let (game_one, game_two) = run_game(&board);

                let one = game_one.result as u32;
                let two = game_two.result as u32;

                update_weights(game_one, one, &mut win_weights);
                update_weights(game_two, two, &mut win_weights);
            }

            // Send the results of the games to the main thread for merging.
            tx_thread.send(win_weights).expect("Should be able to send.");
        });
    }

    // Waits for each thread to finish & merges its results into the main map.
    for finished_threads in 0..THREADS {
        let thread_map = rx.recv().expect("Should always receive a value");

        for choice in thread_map.keys() {
            // If it doesn't contain a value for this choice, add it.
            if !win_weights.contains_key(choice) {
                win_weights.insert(choice.clone(), *thread_map.get(choice).expect("Will exist."));
                continue;
            }

            // if the choice would lead to a death, don't combine it.
            if choice.is_dying_choice() {
                continue;
            }

            // Combine the existing weight with the thread weight.
            let existing_weight = win_weights.get_mut(choice).expect("Will exist.");
            let thread_weight = thread_map.get(choice).expect("Will exist.");
            existing_weight.combine(thread_weight);
        }

        println!("{}", finished_threads + 1);
    }

    let file = File::create("computed_weights.yml").expect("Should be able to create file.");
    let writer = BufWriter::new(file);

    serde_yaml::to_writer(writer, &win_weights).expect("Should be able to write data to file.");
}


/// Simulates two games with the given board state.
pub fn run_game(board: &Board) -> (Games, Games) {
    // Ensures that each game has the same roll pattern
    let rand_seed = fastrand::u64(..);

    // Simulates the games
    // Each game has a different board rng.
    let mut rng = Rng::with_seed(fastrand::u64(..));
    let first_game = rand(board, Vec::new(), &mut Rng::with_seed(rand_seed), &mut rng);

    let mut rng1 = Rng::with_seed(fastrand::u64(..));
    let second_game = rand(board, Vec::new(), &mut Rng::with_seed(rand_seed), &mut rng1);

    // Uses the wrapper to store the game data
    let mut first = Games::new(first_game.1, DRAW);
    let mut second = Games::new(second_game.1, DRAW);

    // Assigns the correct win/loss values to each game
    if first_game.0 > second_game.0 {
        first.result = WIN;
        second.result = LOSS
    } else if second_game.0 > first_game.0 {
        first.result = LOSS;
        second.result = WIN
    }

    // If it's a draw then it can just use the default values.
    (first, second)
}

/// Performs a random move on the given board recursively, until there are no valid moves.
/// The returned u8 is the finial value of the board
fn rand(board: &Board, mut choices: Vec<Choice>, rng_roll: &mut Rng, rng_board: &mut Rng) -> (u8, Vec<Choice>) {
    let rand_move = board.get_rand_roll(rng_roll);

    let mut choice = Choice {
        root_board: board.get_raw(),
        roll: rand_move.roll_value,
        chosen_board: None,
    };

    return match rand_move.get_rand_board(rng_board) {
        None => {
            choices.push(choice);
            (board.calculate_value(), choices)
        }
        Some(rand_board) => {
            choice.set_chosen_board(rand_board);
            choices.push(choice);
            let board = get_board(rand_board as usize).expect("Will exist");
            rand(board, choices, rng_roll, rng_board)
        }
    };
}

/// Updates the HashMap with the outcome of the choices in the game.
fn update_weights(game: Games, value: u32, win_weights: &mut HashMap<Choice, Weight>) {
    for game_move in game.moves {
        // If the move caused a death, don't even consider it.
        if game_move.chosen_board == None {
            continue;
        }

        // If the move hasn't been chosen before create a new weight for it.
        if !win_weights.contains_key(&game_move) {
            let weight = Weight {
                total: value,
                used: 1,
            };

            win_weights.insert(game_move, weight);
            continue;
        }

        // Update the existing weight with the outcome of the game
        let weight = win_weights.get_mut(&game_move).expect("The map will contain this value");
        weight.inc(value);
    }
}

