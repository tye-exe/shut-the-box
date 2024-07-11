use std::sync::Arc;

use fastrand::Rng;

/// Contains the value of a roll & the possible boards it could lead to in reference to the board containing this roll instance.
#[derive(Debug)]
pub struct Roll {
    pub roll_value: u8,
    pub boards: Arc<[u16]>,
}

impl Roll {
    /// Simulates every valid board combination for the given roll from the given alive pieces.
    pub fn new(rolled_value: u8, board: u16) -> Roll {
        let mut boards = Vec::new();
        // The amount of unique boards is all the alive pieces as the alive pieces are stored in binary.
        // Counting up to the max value of a binary number with the same number of digits as alive pieces
        // will iterate though every possible combination of the alive pieces, using the binary value of the
        // counter.
        let unique_board_amount = 2u16.pow(board.count_ones());
        let numeric_board = Self::pieces(board);

        // Finds every valid move.
        for unique_board in 1..unique_board_amount {
            // Checks if the simulated move would add up to the rolled value.
            // If it doesn't, then it's not a valid move.
            let possible_move = Self::sum_move(unique_board, numeric_board.clone());
            if possible_move != rolled_value {
                continue;
            }

            boards.push(Self::preform_move(unique_board, numeric_board.clone()));
        }

        Roll {
            roll_value: rolled_value,
            boards: boards.into(),
        }
    }

    /// Converts the binary bored into an arc containing the numeric value of each piece.
    fn pieces(alive_pieces: u16) -> Arc<[u8]> {
        let mut numeric_value = Vec::with_capacity(9);
        // If the higher pieces are dead, then don't iterate over them.
        let max_iterations = (16 - alive_pieces.leading_zeros()) as u8;

        // Iterates over every (see above comment) piece & gets its numeric value.
        for index in 0..max_iterations {
            let shifted = alive_pieces >> index;

            if (shifted & 1) != 0 {
                numeric_value.push(index + 1)
            }
        }

        numeric_value.into()
    }

    /// Converts the binary encoded board combination to its numeric value.
    /// For example, 0101 would become the value of the numbers at index 2 + index 0 of the given vector.
    fn sum_move(move_to_sum: u16, alive_pieces: Arc<[u8]>) -> u8 {
        let mut numeric_move_sum: u8 = 0;

        for piece_index in 0..alive_pieces.len() {
            let shifted = move_to_sum >> piece_index;

            // Checks if the piece at the current index is alive.
            if (shifted & 1) == 1 {
                numeric_move_sum += alive_pieces.get(piece_index).unwrap()
            }
        }

        numeric_move_sum
    }

    /// Returns the board after the given move has been performed.
    fn preform_move(move_to_perform: u16, alive_pieces: Arc<[u8]>) -> u16 {
        // This function can't be a negative bitmask, as the pieces in the move to perform don't correlate
        // to the pieces in the board given in the new() function.

        let mut resultant_board = 0u16;

        for piece_index in 0..alive_pieces.len() {
            let shifted = move_to_perform >> piece_index;

            // If the piece isn't one being killed by the move set the bit representing the alive piece
            // high in the resultant board
            if (shifted & 1) != 1 {
                let value = alive_pieces.get(piece_index).unwrap() - 1;
                resultant_board |= (1 << value) as u16;
            }
        }

        resultant_board
    }

    /// Gets a random valid board within this roll.
    /// If there are no valid boards then None is returned.
    pub fn get_rand_board(&self, rng: &mut Rng) -> Option<u16> {
        if self.boards.len() == 0 {
            return None;
        }

        // Not sure why it needs to be forked, but it doesn't work without it (as far as I can tell).
        let mut forked = Rng::fork(rng);
        let index = forked.usize(..self.boards.len());

        Some(
            *self
                .boards
                .get(index)
                .expect("The rng is limited by the length"),
        )
    }
}
