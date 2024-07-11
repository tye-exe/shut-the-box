use core::panic;
use derive_more::Display;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use simulation::playing::compute_weights;
use std::fmt::Formatter;
use std::str::FromStr;
use std::sync::mpsc::Sender;
use std::sync::OnceLock;

mod simulation;

/// Randomly simulates the given amount of games to play on the number of given threads.
/// This method writes the best move for each board-roll combination to "best_moves.yml"
pub fn compute(threads: u8, games_to_play: u32, sender: Sender<bool>) {
    compute_weights(threads, games_to_play, sender);
}

// const  c

/// Contains a board & a roll.
/// This is used as a key in a hashmap to the best move.
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct BoardRoll {
    pub(crate) board: u16,
    pub(crate) roll: DiceRoll,
}

impl BoardRoll {
    pub fn new(board: u16, roll: DiceRoll) -> BoardRoll {
        BoardRoll { board, roll }
    }
}

impl Serialize for BoardRoll {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&format!("{}-{}", self.board, self.roll))
    }
}

impl<'de> Deserialize<'de> for BoardRoll {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(BoardRollVisitor)
    }
}

/// The custom visitor to enable deserializing of the [`BoardRoll`] struct.
struct BoardRollVisitor;

impl<'de> Visitor<'de> for BoardRollVisitor {
    type Value = BoardRoll;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a u16 between 0 & 511, a dash '-', a valid encoded dice roll"
        )
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let values = value
            .rsplit_once('-')
            .ok_or(E::custom("invalid string for a board roll"))?;
        // Tries to parse the board.
        let board = match u16::from_str(values.0) {
            Ok(board) => board,
            Err(_) => {
                return Err(E::custom("invalid u16 for board"));
            }
        };
        // Tries to parse the roll.
        let roll: DiceRoll = match u8::from_str(values.1) {
            Ok(roll) => DiceRoll(roll),
            Err(_) => {
                return Err(E::custom("invalid u8 for roll"));
            }
        };

        // Validation on the parsed ints.
        if board > 511 {
            return Err(E::custom("board cannot have a value above 511"));
        }
        if !roll.is_valid() {
            eprintln!("{}", roll.get_value());
            return Err(E::custom("roll cannot have a value above 12"));
        }

        Ok(BoardRoll { board, roll })
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_str(value.as_str())
    }
}

/// Contains a dice combination.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Display, Debug)]
pub struct DiceRoll(u8);

impl DiceRoll {
    pub fn new_single(one: u8) -> Self {
        DiceRoll(one << 5)
    }
    pub fn new_dual(one: u8, two: u8) -> Self {
        let one = one << 5;
        let two = (two & 0b00000111) << 1;
        DiceRoll(one | two)
    }
    /// Returns the summed value of the contained dice.
    pub fn get_value(self) -> u8 {
        let one = (self.0 & 0b11100000) >> 5;
        let two = (self.0 & 0b00001110) >> 1;
        one + two
    }
    /// Returns true if this DiceRoll is a valid roll. False otherwise.
    pub fn is_valid(self) -> bool {
        let value = self.get_value();
        value > 1 && 13 > value
    }
}

impl From<u8> for DiceRoll {
    /// The given value must be between 1 & 12 (inclusive), otherwise this function will panic.
    fn from(value: u8) -> Self {
        match value {
            1 => DiceRoll::new_single(1),
            2 => DiceRoll::new_dual(1, 1),
            3 => DiceRoll::new_dual(2, 1),
            4 => DiceRoll::new_dual(3, 1),
            5 => DiceRoll::new_dual(4, 1),
            6 => DiceRoll::new_dual(5, 1),
            7 => DiceRoll::new_dual(6, 1),
            8 => DiceRoll::new_dual(6, 2),
            9 => DiceRoll::new_dual(6, 3),
            10 => DiceRoll::new_dual(6, 4),
            11 => DiceRoll::new_dual(6, 5),
            12 => DiceRoll::new_dual(6, 6),
            val => panic!(
                "A DiceRoll cannot be a smaller than 1 or larger than 12! Value found: {val}"
            ),
        }
    }
}

static DUAL_ROLLS: OnceLock<[DiceRoll; 36]> = OnceLock::new();

pub fn get_rolls() -> &'static [DiceRoll; 36] {
    DUAL_ROLLS.get_or_init(|| {
        let mut dual_rolls = [DiceRoll(0); 36];

        for one in 1..7u8 {
            for two in 1..7u8 {
                let index = ((one - 1) * 6) + two - 1;
                dual_rolls[index as usize] = DiceRoll::new_dual(one, two);
            }
        }

        dual_rolls
    })
}
