use rand::Rng;
use rand::seq::SliceRandom;
use crate::board::Board;

#[derive(Debug)]
pub struct Roll {
    pub roll_value: u8,
    pub boards: Vec<Board>,

    depth: u16,
}

impl Roll {
    pub fn new(rolled_value: u8, alive_pieces: &Vec<u8>, depth: u16) -> Roll {
        let mut boards = Vec::new();
        let unique_board_amount = 2u16.pow((alive_pieces.len() - 1) as u32);

        for unique_board in 0..unique_board_amount {
            let possible_move = Self::sum_move(unique_board, &alive_pieces);
            if possible_move != rolled_value { continue; }

            let new_alive = Self::preform_move(unique_board, &alive_pieces);
            boards.push(Board::new(new_alive, depth + 1));
        }

        Roll {
            roll_value: rolled_value,
            boards,
            depth,
        }
    }


    /// Converts the binary encoded combination to its numeric value.
    /// For example, 0101 would become the value of the numbers at index 2 + index 0 of the given vector.
    fn sum_move(move_to_sum: u16, alive_pieces: &Vec<u8>) -> u8 {
        let mut numeric_move_sum: u8 = 0;

        for piece_index in 0..alive_pieces.len() {
            let shifted = move_to_sum >> piece_index;

            if (shifted & 1) == 1 {
                numeric_move_sum += alive_pieces.get(piece_index).unwrap()
            }
        }

        numeric_move_sum
    }

    fn preform_move(move_to_perform: u16, alive_pieces: &Vec<u8>) -> Vec<u8> {
        let mut new_alive = Vec::with_capacity(9);

        for piece_index in 0..alive_pieces.len() {
            let shifted = move_to_perform >> piece_index;

            if (shifted & 1) != 1 {
                new_alive.push(alive_pieces.get(piece_index).unwrap().clone());
            }
        }

        new_alive
    }

    pub fn get_rand_board(&self) -> Option<(&Board, u16)> {
        if self.boards.len() == 0 {
            return None;
        }

        let rand_index = rand::thread_rng().gen_range(0..self.boards.len());

        Some((
            self.boards.get(rand_index).expect("Will exist"),
            rand_index as u16
        ))
    }
}