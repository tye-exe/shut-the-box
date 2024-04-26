use weighted_rand::builder::{NewBuilder, WalkerTableBuilder};
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

        let mut roles = Vec::new();
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
        let possible_roll_indexes: [u8; 11] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let roll_weights = [1, 2, 3, 4, 5, 6, 5, 4, 3, 2, 1];

        let builder = WalkerTableBuilder::new(&roll_weights);
        let wa_table = builder.build();

        for i in (0..11).map(|_| wa_table.next()) {
            return (
                self.rolls.get(possible_roll_indexes[i] as usize).expect("Will exist"),
                i as u16
            );
        }

        panic!("A value should always be chosen by the above code.")
    }

    pub fn calculate_value(&self) -> u8 {
        let mut total_value = 0;
        for value in &self.alive {
            total_value+=value;
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