// Binary structure
//
// 0000       | 000    | 000000000
// dice value | unused | board state
//
// The dice value stores the numerical value of the dice role
//
// The board state represents the position of the numbers, from 1 to 9, left to right.
// If a number has a 0 in its place it's been used.
// If a number has a 1 in its place it's not been used.

use std::fmt;
use std::fmt::{Formatter, Write};
use std::hash::{Hash, Hasher};

const BOARD_BITS_HIGH: u16 = 0b0000000111111111;
const DICE_BITS_HIGH: u16 = 0b1111000000000000;

/// Stores a state of the board & the dice role needed from the parent to get to this state.
#[derive(Copy, Clone)]
pub struct GameState {
    state: u16
}


impl GameState {

    /// Creates a new root state.
    /// The root state has all the numbers alive & a die roll of 0.
    pub fn new_root_state() -> GameState {
        GameState { state: 0b0000000111111111 }
    }

    /// Creates a new [GameState] with the given bored & die values.
    pub fn from_board_and_dice(board: &u16, dice: &u8) -> GameState {
        let validated_board = board & BOARD_BITS_HIGH;

        let dice_value: u16 = (0u8 | dice) as u16;
        let dice_value = dice_value << 12;

        GameState {
            state: dice_value | validated_board
        }
    }
}

impl GameState {

    /// Gets the binary representation of the board.
    pub fn get_board(&self) -> u16 {
        self.state & BOARD_BITS_HIGH
    }

    /// Gets the numeric value of the dice role.
    pub fn get_dice(&self) -> u8 {
        (self.state >> 12) as u8
    }


    /// Sets the binary representation of the board.
    pub fn set_board(&mut self, new_board: u16) {
        // Input validation on incoming board.
        let validated_board = new_board & 0b0000000111111111;

        self.state = self.state & !BOARD_BITS_HIGH;
        self.state = self.state | validated_board;
    }

    /// Sets the numeric value of the dice.
    pub fn set_dice(&mut self, new_dice: u8) {
        let formatted_dice_bits: u16 = (new_dice as u16) << 12;

        self.state = self.state & !DICE_BITS_HIGH;
        self.state = self.state | formatted_dice_bits;
    }


    /// Gets the raw state value.
    /// Using this is discouraged.
    pub fn get_raw_state(&self) -> u16 {
        self.state
    }

    /// Sets the raw state value.
    /// Using this is discouraged.
    pub fn set_raw_state(&mut self, raw_state: u16) {
        self.state = raw_state;
    }
}


impl fmt::Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // "{:#0011b}" Prints a minimum of 11 chars of binary (including the 0b prefix).
        // The split removes the starting "0b" chars.
        f.write_str(format!("{:#0011b}", self.get_board()).split_at(2).1)?;
        f.write_str(" : ")?;
        f.write_str(format!("{}", self.get_dice()).as_str())
    }
}

impl fmt::Debug for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("GameState { \n")?;

        // "{:#0011b}" Prints a minimum of 11 chars of binary (including the 0b prefix).
        // The split removes the starting "0b" chars.
        let binary_board = format!("{:#0011b}", self.get_board()).split_at(2).1.to_string();
        f.write_str(format!("    board: {}\n", binary_board).as_str())?;

        f.write_str(format!("    dice: {}\n", self.get_dice()).as_str())?;
        f.write_str("} ")
    }
}

impl PartialEq for GameState {
    fn eq(&self, other: &Self) -> bool {
        return self.get_board() == other.get_board()
    }

    fn ne(&self, other: &Self) -> bool {
        return self.get_board() != other.get_board()
    }
}

impl Eq for GameState {}


impl Hash for GameState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_board().hash(state);
    }
}