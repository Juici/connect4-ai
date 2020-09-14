use std::fmt;

use crate::board::{Board, Column};

pub mod ai;
pub mod console;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Token {
    Player1 = 1,
    Player2 = 2,
}

impl Token {
    pub fn char(self) -> &'static str {
        match self {
            Token::Player1 => "x",
            Token::Player2 => "o",
        }
    }

    pub fn player(self) -> u8 {
        self as u8
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.char())
    }
}

pub trait Player {
    /// Gets the move the player wishes to make.
    ///
    /// `board` is a copy of the game board and `token` is the token the player uses.
    fn decide_move(&mut self, board: &Board, token: Token) -> Column;
}
