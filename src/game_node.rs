use std::fmt;
use std::fmt::{Debug, Formatter};

use crate::game_state::GameState;

/// Stores an array of possible values that two pairs of dice can land on.
/// It is assumed that this is in lowest to highest order.
const POSSIBLE_DICE_VALUES: [u8; 11] = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];


/// This struct represents the current state of the board as well as any state that lead to it or follow it.
#[derive(Clone)]
pub struct GameNode {
    state: GameState,

    parents: Vec<GameState>,
    children: Vec<GameNode>,
}


impl GameNode {

    /// Instantiates a new root node.
    /// A root node is a node with all the numbers high, no parents, & all the children populated.
    pub fn new_root_node() -> GameNode {
        let mut root_node = GameNode {
            state: GameState::new_root_state(),
            parents: vec![],
            children: vec![],
        };
        root_node.calculate_children();
        root_node
    }

    /// Returns a reference to the current game state of the board.
    pub fn get_state(&self) -> &GameState {
        &self.state
    }

    /// Returns a reference to the current children for this state of the board.
    pub fn get_children(&self) -> &Vec<GameNode> {
        &self.children
    }

    /// Returns a copy of the current children for this state of the board.
    pub fn get_children_clone(&self) -> Vec<GameNode> {
        self.children.clone()
    }

    /// Returns a reference to the parents of this node.
    pub fn get_parents(&self) -> &Vec<GameState> {
        &self.parents
    }

    /// Converts the node into a vector that contains the nodes parents
    pub fn into_parents(self) -> Vec<GameState> {
        self.parents
    }


    /// Adds a parent that represents the game state.
    /// A parent has a game state that could lead to this state.
    pub fn add_parent(&mut self, parent: GameState) {
        self.parents.push(parent);
    }

    /// Adds multiple parents that represent this game state.
    /// A parent has a game state that could lead to this state.
    pub fn add_parents(&mut self, parents: Vec<GameState>) {
        self.parents.extend(parents);
    }
}


impl GameNode {

    /// Calculates the children for this node.
    pub fn calculate_children(&mut self) {
        if self.state.get_board() == 0 {
            return;
        }

        let alive_pieces = self.create_vector_representation();

        // Calculates the number of possible combinations that exist for the given game state.
        // Since each number can only be alive or dead, the number of combinations follows a 2^x pattern.
        let unique_combinations: u16 = (1 << alive_pieces.len()) as u16;

        // Iterates every unique combination of pieces possible for the remaining alive pieces.
        // This works by taking the binary representation of the current iteration & converting it
        // to a possible combination.
        // For each bit that is high in the combination it gets the number from alive_pieces at
        // the same index as the bit.
        // All the numbers that were marked are added together to get the sum of that possible combination.
        for combination in 1..unique_combinations {

            // Converts the binary encoded combination to its numeric value.
            let summed_pieces = Self::combination_to_piece_value(combination, &alive_pieces);


            for dice_role in POSSIBLE_DICE_VALUES {

                // Dice roles are ordered lowest to highest.
                if summed_pieces < dice_role { break; }

                // If the pieces don't add up to the dice role then the move is invalid.
                if summed_pieces != dice_role { continue; }

                // -- Creation of child state --
                let child_board = self.state.get_board() & !combination;
                let child_state = GameState::from_board_and_dice(&child_board, &dice_role);

                let child_node = Self::new_child_node(&child_state, self.state);

                self.children.push(child_node);
            }
        }

    }


    /// Returns a vector representation of the alive pieces.
    /// The returned vector will be sorted from smallest to largest.
    fn create_vector_representation(&self) -> Vec<u8> {
        let mut alive_pieces: Vec<u8> = Vec::new();

        // Loops over every piece on the board
        for piece in 0..10 {

            // Shifts the current piece being checked into the least significant position
            let shifted = self.board >> piece;

            // If the piece is dead then continue the loop
            if shifted & 1 != 1 { continue; }

            // Adds the alive pieces to the vector
            alive_pieces.push(piece + 1)
        }

        alive_pieces
    }


    /// Converts the binary encoded combination to its numeric value.
    /// For example, 0101 would become the value of the numbers at index 2 + index 0 of the given vector.
    fn combination_to_piece_value(encoded_combination: u16, alive_pieces: &Vec<u8>) -> u8 {
        let mut summed_pieces: u8 = 0;

        for piece_index in (0..alive_pieces.len()).rev() {
            // Moves the current bit being evaluated into the least signification position.
            let shifted = encoded_combination >> piece_index;

            // Adds the value of the piece at the current index if it's in the combination.
            if shifted & 1 == 1 {
                summed_pieces += alive_pieces.get(piece_index)
                    .expect("Value should exist as its numbers \
                    are bound by the length of this vector.");
            }
        }

        summed_pieces
    }

    /// Creates a new node that is the child of the parent [GameState].
    fn new_child_node(state: &GameState, parent: GameState) -> GameNode {
        GameNode {
            state: state.clone(),
            parents: vec![parent],
            children: vec![],
        }
    }
}

impl GameNode {

    /// Returns a string representation of this [GameNode]
    fn display(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        let mut output = "GameState {\n    state: ".to_string();

        output.push_str(&self.state.to_string());

        output.push_str(",\n    parents: [");

        if self.parents.is_empty() {
            output.push_str("],")
        }
        else {
            // Adds all the parents to the output
            for parent in &self.parents {
                output.push_str("\n        ");
                output.push_str(&parent.to_string());
                output.push(',');
            }
            output.push_str("\n    ],");
        }

        output.push_str("\n    children: [");

        if self.children.is_empty() {
            output.push_str("],")
        }
        else {
            // Adds all the children to the output
            for child in &self.children {
                output.push_str("\n        ");
                output.push_str(&child.to_string());
                output.push(',');
            }
            output.push_str("\n    ],")
        }

        output.push_str("\n}");

        write!(fmt, "{}", output)
    }
}

impl Debug for GameNode {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        self.display(fmt)
    }
}

impl fmt::Display for GameNode {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        self.display(fmt)
    }
}