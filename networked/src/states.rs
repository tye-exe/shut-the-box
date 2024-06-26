use mac_address2::MacAddress;
use serde::{Deserialize, Serialize};

// Possible Packets //

/// Contains every message that the client could send.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum ClientMessages {
    // Joining
    /// Requests to join the game.
    /// The [MacAddress] will be used to identify the player.
    OptInForPlaying(MacAddress),

    // Starting
    /// Informs the server that the client is ready to start the game.
    ReadyForStart(bool),

    // Playing
    ChosenRoll(RollRequest),
    /// Sends the move the client made back to the server.
    ChosenMove(ClientMove),

    /// If there was an error inform the server
    Error(ClientError)
}

/// Contains every message that the server could send.
#[derive(Serialize, Deserialize)]
pub enum ServerMessages {
    // Joining
    /// Informs the client that they were accepted into the game.
    OptInAccept,
    /// Informs the client that they were rejected from the game.
    OptInDeny,

    // Starting
    /// Informs the client of the number of connected players.
    PlayersConnected(u8),
    /// Informs the client of the number of ready players.
    PlayersReady(u8),

    // Playing
    /// Queries the client over how many dice they want rolled this move.
    QueryClientRoll,
    /// Queries the client for their move.
    QueryClientForMove(ClientToMove),

    // Ending
    /// Informs the client that they won.
    SendWin,
    /// Informs the client that they drew
    SendDraw(DrawingPlayerAmount),
    /// Informs the client that they lost.
    SendLoss(WinningScore),

    /// If there was an error inform the client
    Error(ServerError)
}

// Data types //

/// Whether the client wants one dice rolled or two dice rolled.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum RollRequest {
    BothDice,
    SingleDice,
}

/// Contains the possible dice the client used when making the move
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum ClientMove {
    BothDice(ClientMovedBoard),
    FirstDice(ClientMovedBoard),
    SecondDice(ClientMovedBoard),
    CannotMove,
}

/// Contains the board the client made the move to.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ClientMovedBoard(u16);


/// Contains the winners score.
#[derive(Serialize, Deserialize)]
pub struct WinningScore(u8);

/// Contains the amount of players that you drew with.
#[derive(Serialize, Deserialize)]
pub struct DrawingPlayerAmount(u8);


/// Contains the data for the client to make a move upon.
#[derive(Serialize, Deserialize)]
pub enum ClientToMove {
    /// Contains the board state of the current game & one rolled dice.
    OneDice(u16, u8),
    /// Contains the board state of the current game & two rolled dice.
    TwoDice{board: u16, dice_1: u8, dice_2: u8}
}

// Errors //

#[derive(Serialize, Deserialize)]
pub enum ServerError {
    /// Sent to the client if it requests a move before requesting a roll.
    MoveBeforeRoll
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum ClientError {
    
}