use std::sync::mpsc::Sender;

use simulation::playing::compute_weights;

mod simulation;
pub mod board_roll;

/// Randomly simulates the given amount of games to play on the number of given threads.
/// This method writes the best move for each board-roll combination to "best_moves.yml"
pub fn compute(threads: u8, games_to_play: u32, sender: Sender<bool>) {
    compute_weights(threads, games_to_play, sender);
}