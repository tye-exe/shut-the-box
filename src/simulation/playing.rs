use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::BufWriter;
use std::ops::Div;
use std::sync::mpsc;
use std::thread;

use fastrand::Rng;
use crate::board_roll::BoardRoll;

use crate::simulation::board::{Board, get_board, get_rand_board};
use crate::simulation::playing::Result::{DRAW, LOSS, WIN};


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
#[derive(Debug, Copy, Clone)]
pub struct Weight {
    total: u32,
    used: u32,
}

impl Weight {
    /// Adds the given amount to this weight.
    pub fn inc(&mut self, amount: u32) {
        self.total += amount;
        self.used += 1;
    }

    /// Adds the given weight to this weight.
    pub fn combine(&mut self, other: &Weight) {
        self.total += other.total;
        self.used += other.used;
    }

    /// Calculates the average of if choosing a move would result in a win.
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


/// Represents the weight of each simulation outcome.
/// Win = 1000
/// Draw = 500
/// Loss = 0
///
/// The values are big as it results in higher accuracy during the division for the average win calculation.
#[derive(Copy, Clone)]
pub enum Result {
    WIN = 1000,
    DRAW = 500,
    LOSS = 0,
}


/// Randomly simulates the given amount of games to play on the number of given threads.
/// This method writes the best move for each board-roll combination to "best_moves.yml"
pub fn compute_weights(threads: u8, games_to_play: u32) {
    let mut win_weights: HashMap<Choice, Weight> = HashMap::new();
    let (tx, rx) = mpsc::channel();

    // Creates threads to compute random simulations of the game.
    for _ in 0..threads {
        let tx_thread = tx.clone();

        thread::spawn(move || {
            // Each simulation will start from a random board to get an even distribution
            let mut win_weights: HashMap<Choice, Weight> = HashMap::new();

            for _ in 0..games_to_play {
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
    for finished_threads in 0..threads {
        let thread_map = rx.recv().expect("Should always receive a value");

        for choice in thread_map.keys() {
            // If it doesn't contain a value for this choice, add it.
            if !win_weights.contains_key(choice) {
                win_weights.insert(choice.clone(), *thread_map.get(choice).expect("Will exist."));
                continue;
            }

            // Combine the existing weight with the thread weight.
            let existing_weight = win_weights.get_mut(choice).expect("Will exist.");
            let thread_weight = thread_map.get(choice).expect("Will exist.");
            existing_weight.combine(thread_weight);
        }

        println!("Games simulated: {}", (finished_threads + 1) as u32 * games_to_play);
    }


    // Contains the best choice for each roll for each board.
    let mut choice_map = HashMap::new();
    // Contains the win % of the current best choice
    let mut weight_map = HashMap::new();

    // Calculates the best choice for each roll for each board.
    for choice in win_weights.keys() {
        let weight = win_weights.get(choice).expect("Iterating over every key so the kye must be in the map.");
        let win_average = weight.calculate();

        let board_roll = BoardRoll {
            board: choice.root_board,
            roll: choice.roll,
        };

        // If the map contains a choice that looses more often discard this choice.
        if let Some(existing) = weight_map.get(&board_roll) {
            if *existing < win_average { continue; }
        }

        weight_map.insert(
            board_roll,
            win_average,
        );

        choice_map.insert(
            board_roll,
            choice.chosen_board.expect("None boards are removed before this function."),
        );
    }


    // Writes the data to the file to be referenced later.
    let file = File::create("best_moves.yml").expect("Should be able to create file.");
    let writer = BufWriter::new(file);
    serde_yaml::to_writer(writer, &choice_map).expect("Should be able to write data to file.");
}


/// Simulates two random games with the given board state.
pub fn run_game(board: &Board) -> (Games, Games) {
    // Ensures that each game has the same roll rng.
    let rand_seed = fastrand::u64(..);

    // Simulates the games.
    // Each game has a different board rng.
    let mut rng_1 = Rng::with_seed(fastrand::u64(..));
    let first_game = rand(board, Vec::new(), &mut Rng::with_seed(rand_seed), &mut rng_1);

    let mut rng_2 = Rng::with_seed(fastrand::u64(..));
    let second_game = rand(board, Vec::new(), &mut Rng::with_seed(rand_seed), &mut rng_2);

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
fn rand(board: &Board, mut choices: Vec<Choice>, roll_rng: &mut Rng, board_rng: &mut Rng) -> (u8, Vec<Choice>) {
    let rand_roll = board.get_rand_roll(roll_rng);

    let mut choice = Choice {
        root_board: board.get_raw(),
        roll: rand_roll.roll_value,
        chosen_board: None,
    };

    // If there are no more valid moves return the board value & the moves leading to the last valid board.
    // If there are more valid moves randomly simulate them.
    return match rand_roll.get_rand_board(board_rng) {
        None => {
            choices.push(choice);
            (board.calculate_value(), choices)
        }
        Some(rand_board) => {
            choice.set_chosen_board(rand_board);
            choices.push(choice);

            let board = get_board(rand_board as usize).expect("Will exist");
            rand(board, choices, roll_rng, board_rng)
        }
    };
}

/// Updates the HashMap with the outcome of the choices in the game.
fn update_weights(game: Games, value: u32, win_weights: &mut HashMap<Choice, Weight>) {
    for game_move in game.moves {
        // If the move caused a death, don't even consider it.
        if game_move.is_dying_choice() {
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

