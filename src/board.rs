use rand::seq::SliceRandom;
use rand::thread_rng;
use weighted_rand::builder::NewBuilder;

use crate::roll::Roll;

#[derive(Debug)]
pub struct Board {
    alive: Vec<u8>,
    rolls: Vec<Roll>,

    depth: u16,
}

impl Board {
    pub fn new(mut alive: Vec<u8>, depth: u16) -> Board {
        alive.sort();

        let mut roles = Vec::with_capacity(11);
        for role in 2u8..13 {
            roles.push(Roll::new(role, &alive, depth + 1));
        }

        Board {
            alive,
            rolls: roles,
            depth,
        }
    }

    pub fn get_rand_roll(&self) -> (&Roll, u16) {
        let possible_rolls_indexes: [u8; 36] = [0, 1, 1, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 10];
        let roll_index = possible_rolls_indexes.choose(&mut thread_rng()).expect("Will never be empty");

        return (
            self.rolls.get(*roll_index as usize).expect("A board always has 11 roles."),
            *roll_index as u16
        );
    }

    pub fn calculate_value(&self) -> u8 {
        let mut total_value = 0;
        for value in &self.alive {
            total_value += value;
        }
        total_value
    }
}


impl Board {
    pub fn get_max_depth(&self, depth: u16) -> u16 {
        let mut deepest = depth.clone();

        for roll in &self.rolls {
            for board in &roll.boards {
                let found = board.get_max_depth(depth.clone() + 1);

                if found > deepest {
                    deepest = found;
                }
            }
        }

        deepest
    }
}