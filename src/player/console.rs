use std::process;
use std::str::FromStr;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::board::{Board, Column};
use crate::player::{Player, Token};

pub struct ConsolePlayer {
    editor: Editor<()>,
}

impl ConsolePlayer {
    pub fn new() -> ConsolePlayer {
        ConsolePlayer {
            editor: Editor::new(),
        }
    }
}

impl Player for ConsolePlayer {
    fn decide_move(&mut self, board: &Board, token: Token) -> Column {
        let prompt = format!("{} >> ", token);

        loop {
            println!("\n{}", board);

            match self.editor.readline(&prompt) {
                Ok(line) => {
                    let line = line.trim();

                    match Column::from_str(line).map(|column| column.saturating_sub(1)) {
                        Ok(column) if board.is_legal(column) => return column,
                        _ => println!("\nIllegal move '{}', try again", line),
                    };
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    println!("\nQuitting program");
                    process::exit(1);
                }
                Err(err) => {
                    println!("\nError: {:?}", err);
                    process::exit(1);
                }
            }
        }
    }
}
