use fastrand::Rng;

use crate::roll::Roll;

// 0000000 | 000000000
// 0000000 | 987654321

#[derive(Debug)]
pub struct Board {
    alive: u16,
    rolls: Vec<Roll>,
}

const POSSIBLE_ROLLS_INDEXES: [u8; 36] = [0, 1, 1, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 10];

impl Board {
    pub fn new(alive: u16) -> Board {
        let mut roles = Vec::with_capacity(11);
        for role in 2u8..13 {
            roles.push(Roll::new(role, alive));
        }

        Board {
            alive,
            rolls: roles,
        }
    }

    pub fn get_rand_roll(&self, rng: &mut Rng) -> &Roll {
        let index = rng.usize(..POSSIBLE_ROLLS_INDEXES.len());
        let roll_index = POSSIBLE_ROLLS_INDEXES.get(index).expect("Will never be empty");

        return self.rolls.get(*roll_index as usize).expect("A board always has 11 roles.");
    }

    pub fn calculate_value(&self) -> u8 {
        let mut total_value = 0;

        for index in 0..9 {
            let piece = self.alive >> index;

            if piece & 1 == 1 {
                total_value += index + 1;
            }
        }

        total_value
    }

    pub fn get_raw(&self) -> u16 {
        self.alive
    }
}