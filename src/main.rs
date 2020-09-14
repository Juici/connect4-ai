use crate::game::Game;
use crate::player::ai::{AIPlayer, Difficulty};
use crate::player::console::ConsolePlayer;

pub mod board;
pub mod game;
pub mod player;

fn main() {
    let player1 = ConsolePlayer::new();
    // let player2 = ConsolePlayer::new();
    let player2 = AIPlayer::new(Difficulty::Master);

    let game = Game::new(player1, player2);
    let (board, winner) = game.play();

    println!("\nFinal board:\n{}", board);

    println!();
    match winner {
        Some(token) => println!("Player {} wins", token.player()),
        None => println!("The game ended in a draw"),
    }
}
