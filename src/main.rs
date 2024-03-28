use std::collections::{HashMap, VecDeque};

use crate::game_node::GameNode;
use crate::game_state::GameState;

mod game_node;
mod game_state;

fn main() {

    let mut explored_states: HashMap<GameState, GameNode> = HashMap::new();
    let mut unexplored_nodes: VecDeque<GameNode> = VecDeque::new();

    // Adds the beginning state of the game with all the numbers alvie to the unexplored node queue.
    unexplored_nodes.push_back(GameNode::new_root_node());

    // Keeps looping until all the possible states of the game have been explored.
    while !unexplored_nodes.is_empty() {

        // Gets the node to explore & calculates its children.
        let mut node = unexplored_nodes.pop_front().expect("Value should exist due to the loop condition");
        node.calculate_children();

        // Gets the state of the board & all the children that it could have.
        let node_state = (&node).get_state().clone();
        let children: Vec<GameNode> = node.get_children().clone();

        // If the node is an explored one then app the parents of this node to the already existing node.
        if explored_states.contains_key(&node_state) {

            let mut explored_node = explored_states.get_mut(&node_state)
                .expect("Value should exist as alternate path is taken if it doesn't.");

            explored_node.add_parents(node.into_parents());

            continue;
        }

        // If the node is unexplored then add the node to the explored nodes
        // & add its children to the unexplored nodes.
        explored_states.insert(node_state, node);
        for child in children {
            unexplored_nodes.push_back(child);
        }

    }
}