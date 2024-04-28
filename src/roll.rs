use fastrand::Rng;

#[derive(Debug)]
pub struct Roll {
    pub roll_value: u8,
    pub boards: Vec<u16>,
}

impl Roll {
    pub fn new(rolled_value: u8, alive_pieces: u16) -> Roll {
        let mut boards = Vec::new();
        let unique_board_amount = 2u16.pow(alive_pieces.count_ones());
        let numeric_board = Self::pieces(alive_pieces);

        for unique_board in 1..unique_board_amount {
            let possible_move = Self::sum_move(unique_board, &numeric_board);
            if possible_move != rolled_value { continue; }

            boards.push(Self::preform_move(unique_board, &numeric_board));
        }

        Roll {
            roll_value: rolled_value,
            boards,
        }
    }


    /// Converts the binary bored into a vec containing the numeric value of each piece.
    fn pieces(alive_pieces: u16) -> Vec<u8> {
        let mut numeric_value = Vec::with_capacity(9);
        let max_iterations = (16 - alive_pieces.leading_zeros()) as u8;

        for index in 0..max_iterations {
            let shifted = alive_pieces >> index;

            if (shifted & 1) != 0 {
                numeric_value.push(index + 1)
            }
        }

        numeric_value
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

    /// Returns the board after the given move has been performed.
    fn preform_move(move_to_perform: u16, alive_pieces: &Vec<u8>) -> u16 {
        let mut board = 0u16;

        for piece_index in 0..alive_pieces.len() {
            let shifted = move_to_perform >> piece_index;

            if (shifted & 1) != 1 {
                let value = alive_pieces.get(piece_index).unwrap() - 1;
                board |= (1 << value) as u16;
            }
        }

        board
    }


    pub fn get_rand_board(&self, rng: &mut Rng) -> Option<u16> {
        if self.boards.len() == 0 {
            return None;
        }

        let index = Rng::fork(rng).usize(..self.boards.len());
        Some(*self.boards.get(index).expect("The rng is limited by the length"))
    }
}