use std::sync::OnceLock;
use std::time::SystemTime;

use rand::prelude::SliceRandom;
use rand::thread_rng;

use crate::board::Board;

mod roll;
mod board;
mod playing;

/// Stores all the computed boards
static BOARDS: OnceLock<Vec<Board>> = OnceLock::new();

/// Gets the pre-computed boards.
pub fn get_boards() -> &'static Vec<Board> {
    // Gets the pre-computed boards, or if they haven't been computed before, they are computed, cached, & returned.
    BOARDS.get_or_init(|| {
        let mut possible_boards = Vec::with_capacity(512);

        // Iterates though every possible board.
        // From 0b000000000 to 0b111111111.
        for index in 0..512 {
            possible_boards.push(Board::new(index));
        }

        possible_boards
    })
}

/// Gets the board at the given index.
/// If the index is out of bounds, then None will be returned.
pub fn get_board(binary_board: usize) -> Option<&'static Board> {
    get_boards().get(binary_board)
}

/// Gets a random board.
pub fn get_rand_board() -> &'static Board {
    get_boards().choose(&mut thread_rng()).expect("The vec will never be empty.")
}

fn main() {
    let before = SystemTime::now();
    playing::compute_weights(8, 10000000);

    let duration = SystemTime::now().duration_since(before).unwrap();
    println!("Time taken to simulate: {:?}", duration);
}