use std::fmt::Formatter;
use std::str::FromStr;

use derive_more::{Display, From, Into};
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Contains a board & a roll.
/// This is used as a key to the best move.
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
            "a u16 between 0 & 511, a dash '-', a u8 between 2 & 12"
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
        let roll = match u8::from_str(values.1) {
            Ok(roll) => DiceRoll(roll),
            Err(_) => {
                return Err(E::custom("invalid u8 for roll"));
            }
        };

        // Validation on the parsed ints.
        if board > 511 {
            return Err(E::custom("board cannot have a value above 511"));
        }
        if roll.get_value() > 12 {
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Display, Debug)]
pub struct DiceRoll(u8);

impl DiceRoll {
    pub fn new(one: u8, two: u8) -> Self {
        let one = one << 5;
        let two = (two & 0b00000111) << 1;
        DiceRoll(one | two)
    }
    pub fn get_value(self) -> u8 {
        let one = self.0 & 0b11100000;
        let two = self.0 & 0b00001110;
        one + two
    }
}
