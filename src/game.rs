use crate::board::Board;
use crate::player::{Player, Token};

pub struct Game<P1: Player, P2: Player> {
    board: Board,
    player1: P1,
    player2: P2,
}

impl<P1: Player, P2: Player> Game<P1, P2> {
    /// Creates a new game.
    pub fn new(player1: P1, player2: P2) -> Game<P1, P2> {
        Game {
            board: Board::new(),
            player1,
            player2,
        }
    }

    /// Plays the game and returns the board and the winner if there is one.
    pub fn play(self) -> (Board, Option<Token>) {
        let Game {
            mut board,
            mut player1,
            mut player2,
        } = self;

        loop {
            let token = board.current_player();
            let column = match token {
                Token::Player1 => player1.decide_move(&board, token),
                Token::Player2 => player2.decide_move(&board, token),
            };
            board.make_move(column);

            match board.winner() {
                Some(winner) => return (board, Some(winner)),
                None if board.legal_moves().next().is_none() => return (board, None),
                None => {}
            }
        }
    }
}
