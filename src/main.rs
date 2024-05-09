use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;
use std::thread;

use eframe::egui;
use eframe::epaint::Color32;
use egui::{FontId, Id, Rect, RichText, TextBuffer, TextFormat, Ui, Vec2, Window};
use egui::ahash::HashMap;
use egui::text::LayoutJob;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::__private::de::IdentifierDeserializer;
use serde::de::{Error, Visitor};

use board_roll::BoardRoll;

use crate::simulation::playing;

mod simulation;
mod board_roll;

// The id's for the panels.
const WINDOW_NAME: &'static str = "Shut The Box";
const TOP_PANEL: &'static str = "Top Panel";
const RECALCULATE: &'static str = "Recalculate";
const ROLL_BOARD_TABLE: &'static str = "Roll Board Table";


struct Main {
    // Vars to do with the recalculation window
    /// Whether the window to recalculate the best moves is open.
    recalculate_window_open: bool,
    /// Whether the best moves are being recalculated.
    recalculation_in_progress: bool,
    /// The amount of games to simulate.
    games_to_simulate: u32,
    /// The unvalidated amount of games to simulate.
    unvalidated_games_to_simulate: String,
    /// Whether the parsing of the number to simulate is correct.
    could_parse_games: bool,

    // Vars to do with display the boards
    /// The current board having its moves displayed.
    root_board: u16,
    /// Stores the previous root board that was displayed
    previous_boards: Vec<u16>,
    /// Stores the pre-calculated best moves from a simulation.
    parsed_moves: Option<HashMap<BoardRoll, u16>>,
}

impl Default for Main {
    fn default() -> Self {
        Main {
            recalculate_window_open: false,
            recalculation_in_progress: false,
            games_to_simulate: 100000,
            unvalidated_games_to_simulate: String::from("100000"),
            could_parse_games: true,
            root_board: 511,
            previous_boards: vec![511],
            parsed_moves: parse_moves(),
        }
    }
}

fn parse_moves() -> Option<HashMap<BoardRoll, u16>> {
    let file = match File::open("best_moves.yml") {
        Ok(file) => { file }
        Err(_) => { return None; }
    };
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader).ok()
}

impl Main {
    fn recalculate_best(games_to_simulate: u32) {
        // Gets the amount of threads a system has.
        // Defaults to 4.
        let threads = match thread::available_parallelism() {
            Ok(number) => { number.get() as u8 }
            Err(_) => { 4 }
        };

        playing::compute_weights(threads, games_to_simulate);
        todo!("Make this async & include a progress bar")
    }
}

// The core function for drawing a gui.
impl eframe::App for Main {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        context.set_pixels_per_point(1.5);

        // Sets the content of the top panel
        egui::TopBottomPanel::top(Id::new(TOP_PANEL))
            .show(context, |ui| {
                self.top_panel(context, ui)
            });

        // Sets the content of the main window.
        egui::CentralPanel::default()
            .show(context, |ui| {
                // Draws the best possible moves
                let board_info = self.central_panel(context, ui);

                // If the moves haven't been calculated yet return.
                if board_info.is_none() { return; }
                let board_info = board_info.unwrap();

                // Checks if any of the move have been clicked on.
                let mut clicked_on = None;
                for board_index in 0..board_info.len() {
                    let board = board_info.get(board_index).expect("Will exist");

                    let clicked = ui.interact(board.1, board.0, egui::Sense::click()).clicked();
                    if !clicked { continue; }

                    clicked_on = Some(board_index as u8)
                }

                // If none of the boards were clicked on, return.
                if clicked_on == None { return; }
                let clicked_on = clicked_on.unwrap();

                // Checks if the root board was clicked on.
                // If it was move back.
                if clicked_on == 0 {
                    self.previous_boards.pop()
                        .and_then(|last_board| {
                            self.root_board = last_board;
                            None::<u16>
                        });

                    return;
                }

                let best_moves = self.parsed_moves.as_ref().expect("Will exist as board info must exist to get to this point. Board info requires this to be some.");

                let board_roll = BoardRoll::new(
                    self.root_board,
                    clicked_on + 1, // Clicked-on is one less than the value it should be.
                );

                // If the value doesn't exist, then it's a dying move.
                match best_moves.get(&board_roll) {
                    Some(best_move) => {
                        self.previous_boards.push(self.root_board);
                        self.root_board = *best_move
                    }
                    None => {
                        // Will execute on dying move
                    }
                };
            });
    }
}

// Sub-functions for drawing the gui.
impl Main {
    /// The code for drawing the top panel of the gui.
    fn top_panel(&mut self, context: &egui::Context, ui: &mut Ui) {
        // Creates a button that will be used to recalculate the best moves.
        let recalculate_window_button = ui.button("Recalculate");

        // Opens the window when the button is clicked.
        if recalculate_window_button.clicked() {
            self.recalculate_window_open = true;
        }

        // Creates a new window for the recalculating options.
        Window::new(RECALCULATE)
            .open(&mut self.recalculate_window_open)
            .show(context, |ui| {
                ui.set_width_range(100f32..=200f32);

                ui.heading(RichText::new("WARNING:").underline());
                ui.label("This is very intensive.");

                ui.add_space(10.);

                // Displays the amount of games to be simulated.
                ui.label("Games to simulate:");
                ui.horizontal(|ui| {
                    // The text box for the value to parse.
                    let text_box = ui.add(egui::TextEdit::singleline(&mut self.unvalidated_games_to_simulate));

                    // If the text can't be parsed as an unsigned int show an error.
                    match u32::from_str(self.unvalidated_games_to_simulate.as_ref()) {
                        Ok(to_simulate) => {
                            self.games_to_simulate = to_simulate;
                            self.could_parse_games = true;
                        }
                        Err(_) => {
                            ui.label("⚠");
                            self.could_parse_games = false;
                        }
                    }

                    // If the input is invalid then the text will lose focus.
                    text_box.request_focus();
                });

                ui.add_space(10.);

                // Recalculates the values.
                let recalculate_button = ui.button(RichText::new("Recalculate").color(Color32::LIGHT_RED));
                if recalculate_button.clicked() && self.could_parse_games {
                    Self::recalculate_best(self.games_to_simulate)
                }
            });
    }

    fn central_panel(&self, context: &egui::Context, ui: &mut Ui) -> Option<Vec<(Id, Rect)>> {
        // Checks if best moves have been calculated.
        if let Some(best_moves) = &self.parsed_moves {

            // Creates a vec which will store the position & id of each displayed board.
            let mut board_info = Vec::with_capacity(13);

            // Generates the layout for the root board.
            let root_layout = Self::generate_root_board(self.root_board);
            let gallery = context.fonts(|fonts| {
                fonts.layout_job(root_layout)
            });

            // Displays the root board.
            ui.painter().galley(ui.next_widget_position(), gallery, Color32::WHITE);
            // Saves the info about the root board to use later.
            board_info.push(ui.allocate_space(Vec2::new(100., 30.)));

            // Generates the layout for the best moves for each roll.
            let mut board_layouts = Vec::with_capacity(12);
            for roll in 2..13 {
                let board_roll = BoardRoll::new(self.root_board, roll);
                let best_move = *best_moves.get(&board_roll).unwrap_or(&0u16);

                board_layouts.push(Self::generate_board(self.root_board, roll, best_move));
            }

            // Iterates over the generate board & displays them.
            for layout in board_layouts {
                let gallery = context.fonts(|fonts| {
                    fonts.layout_job(layout)
                });

                // Draws the board
                ui.painter().galley(ui.next_widget_position(), gallery, Color32::WHITE);
                // Saves the info about the drawn board for later use.
                board_info.push(ui.allocate_space(Vec2::new(100., 20.)));
            }

            return Some(board_info);
        }

        None
    }

    fn generate_root_board(root_board: u16) -> LayoutJob {
        let root_pieces = Self::board_to_array(root_board);
        let mut board_text = LayoutJob::default();

        board_text.append(
            "<--",
            0.,
            TextFormat {
                background: Color32::BLUE,
                ..Default::default()
            },
        );

        board_text.append(
            " || ",
            0.,
            TextFormat::default(),
        );

        // Iterates from the highest to lowest pieces.
        for piece_index in (0..9u8).rev() {
            let root_piece = root_pieces[piece_index as usize];

            let background = match root_piece {
                // If the piece is alive then it should be green.
                true => { Color32::DARK_GREEN }
                // If the piece is down, then it should be grayed out.
                false => { Color32::DARK_GRAY }
            };

            // Gets the value of the piece as a string.
            let mut piece_value = (piece_index + 1).to_string();
            // Adds a space for padding.
            piece_value.push_str(" ");

            // Adds the piece string to the layout
            board_text.append(
                piece_value.as_str(),
                0.,
                TextFormat {
                    background,
                    ..Default::default()
                },
            );
        }

        board_text
    }

    fn generate_board(root_board: u16, roll_value: u8, move_board: u16) -> LayoutJob {
        let root_pieces = Self::board_to_array(root_board);
        let move_pieces = Self::board_to_array(move_board);

        let mut board_text = LayoutJob::default();

        // Adds the roll value first.
        let mut roll_string = roll_value.to_string();
        roll_string.push_str(" ");

        // If the roll is only a single digit add an extra two spaces, so
        // it lines up with the two digit rolls.
        if roll_value < 10 { roll_string.push_str("  "); }

        board_text.append(
            roll_string.as_str(),
            0.,
            TextFormat {
                background: Color32::BLUE,
                ..Default::default()
            },
        );

        // Adds padding text to separate the roll value from the board.
        board_text.append(
            " || ",
            0.,
            TextFormat::default(),
        );


        // Iterates from the highest to lowest pieces.
        for piece_index in (0..9u8).rev() {
            let root_piece = root_pieces[piece_index as usize];
            let move_piece = move_pieces[piece_index as usize];

            let mut background = match (root_piece, move_piece) {
                // If both pieces are alive, it wasn't affected in the move.
                (true, true) => { Color32::DARK_GREEN }
                // If both piece are down, then they should be grayed out.
                (false, false) => { Color32::DARK_GRAY }
                // If the root piece is alive & the move one isn't then it will get knocked down.
                (true, false) => { Color32::GOLD }
                // It shouldn't be possible that a root piece is dead, yet a move piece is alive.
                (false, true) => {
                    return LayoutJob::simple_singleline(
                        "INVALID BOARD STATE".to_string(),
                        FontId::default(),
                        Color32::RED,
                    );
                }
            };

            // If the move is a dying one then colour every piece gray.
            if move_board == 0 {
                background = Color32::DARK_GRAY;
            }

            let mut piece_value = (piece_index + 1).to_string();
            piece_value.push_str(" ");

            board_text.append(
                piece_value.as_str(),
                0.,
                TextFormat {
                    background,
                    ..Default::default()
                },
            );
        }

        board_text
    }

    /// Converts a binary representation of the board to an array.
    /// The 0th index represents piece 1.
    /// The 8th index represents piece 9.
    fn board_to_array(board: u16) -> [bool; 9] {
        let mut root_pieces = [false; 9];

        for index in 0..9 {
            let piece = board >> index;
            // if the piece is alive mark it as so.
            if piece & 1 == 1 {
                root_pieces[index] = true;
            }
        }

        root_pieces
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((350.0, 550.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        WINDOW_NAME,
        native_options,
        Box::new(|_| Box::<Main>::default()),
    )
}