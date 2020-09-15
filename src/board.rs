use std::fmt;
use std::ops::Deref;

use crate::player::Token;

pub type BitBoard = u64;
pub type Column = u8;
pub type ColumnDiff = i8;

// Bitmask of the board in the 7x6 case.
//  .  .  .  .  .  .  .  TOP
//  5 12 19 26 33 40 47
//  4 11 18 25 32 39 46
//  3 10 17 24 31 38 45
//  2  9 16 23 30 37 44
//  1  8 15 22 29 36 43
//  0  7 14 21 28 35 42  BOTTOM

// WIDTH * (HEIGHT + 1) must not be larger than the number of bits in a BitBoard.
// ie. 7 * (6 + 1) = 49 < 64
pub const WIDTH: Column = 7;
pub const HEIGHT: Column = 6;
pub const BOARD_SIZE: Column = WIDTH * HEIGHT;

const BOTTOM: BitBoard = ((1 << ((HEIGHT + 1) * WIDTH)) - 1) / ((1 << (HEIGHT + 1)) - 1);
const TOP: BitBoard = BOTTOM << HEIGHT;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Board {
    moves: [Column; BOARD_SIZE as usize],
    heights: [Column; WIDTH as usize],
    pub players: [BitBoard; 2],
    ply: usize,
}

impl Board {
    /// Creates a new empty board.
    pub const fn new() -> Board {
        let moves = [0; BOARD_SIZE as usize];
        let players = [0; 2];

        let mut heights = [0; WIDTH as usize];
        let mut i = 0;
        while i < WIDTH {
            heights[i as usize] = (HEIGHT + 1) * i;
            i += 1;
        }

        Board {
            moves,
            heights,
            players,
            ply: 0,
        }
    }

    /// Gets the token of the current player.
    pub fn current_player(&self) -> Token {
        match self.ply & 1 {
            0 => Token::Player1,
            1 => Token::Player2,
            // This is optimised out in release builds.
            _ => unreachable!(),
        }
    }

    /// Gets the token in the given row and column if not empty.
    pub fn token_at(&self, row: Column, column: Column) -> Option<Token> {
        let mask = 1 << (row + (column * (HEIGHT + 1)));

        if (self.players[0] & mask) != 0 {
            Some(Token::Player1)
        } else if (self.players[1] & mask) != 0 {
            Some(Token::Player2)
        } else {
            None
        }
    }

    /// Gets whether the given column has space.
    pub fn has_space(&self, column: Column) -> bool {
        assert!(
            column < WIDTH,
            "column out of range [0, {}): {}",
            WIDTH,
            column
        );

        Self::is_legal_board(self.players[self.ply & 1] | (1 << self.heights[column as usize]))
    }

    /// Gets whether a move in the given column is legal.
    pub fn is_legal(&self, column: Column) -> bool {
        column < WIDTH && self.has_space(column)
    }

    /// Peeks ahead at the board state following a move in the given column.
    pub fn peekable(&mut self, column: Column) -> PeekableBoard {
        self.make_move(column);
        PeekableBoard { board: self }
    }

    /// Makes a move in the given column for the current player.
    pub fn make_move(&mut self, column: Column) {
        assert!(self.has_space(column), "column is full: {}", column);

        self.players[self.ply & 1] ^= 1 << self.heights[column as usize];
        self.heights[column as usize] += 1;

        self.moves[self.ply] = column;
        self.ply += 1;
    }

    /// Undoes the previous move.
    pub fn undo_move(&mut self) {
        self.ply -= 1;
        let column = self.moves[self.ply];

        self.heights[column as usize] -= 1;
        self.players[self.ply & 1] ^= 1 << self.heights[column as usize];
    }

    /// Gets the token of the winner, if there is one.
    ///
    /// This will return `None` if the game is not finished or if it is a draw.
    pub fn winner(&self) -> Option<Token> {
        let [player1, player2] = self.players;
        if Self::is_win(player1) {
            Some(Token::Player1)
        } else if Self::is_win(player2) {
            Some(Token::Player2)
        } else {
            None
        }
    }

    /// Gets an iterator of legal moves.
    pub fn legal_moves(&self) -> LegalMoves {
        LegalMoves {
            board: self.players[self.ply & 1],
            heights: self.heights,
            column: 0,
        }
    }

    /// Gets an encoding of the board position state.
    pub fn position_code(&self) -> BitBoard {
        self.players[self.ply & 1] + self.players[0] + self.players[1] + BOTTOM
    }

    /// Gets whether the given board is a winning board.
    const fn is_win(board: BitBoard) -> bool {
        let mut h = board & (board >> (HEIGHT + 1)); // Horizontal
        let mut v = board & (board >> 1); // Vertical
        let mut d1 = board & (board >> HEIGHT); // Diagonal \
        let mut d2 = board & (board >> (HEIGHT + 2)); // Diagonal /

        h &= h >> 2 * (HEIGHT + 1);
        v &= v >> 2;
        d1 &= d1 >> 2 * HEIGHT;
        d2 &= d2 >> 2 * (HEIGHT + 2);

        (h | v | d1 | d2) != 0
    }

    /// Gets whether the given board is legal.
    const fn is_legal_board(board: BitBoard) -> bool {
        (board & TOP) == 0
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in (0..HEIGHT).rev() {
            for column in 0..WIDTH {
                if column > 0 {
                    f.write_str(" ")?;
                }

                f.write_str(match self.token_at(row, column) {
                    Some(token) => token.char(),
                    None => ".",
                })?;
            }

            f.write_str("\n")?;
        }

        // Divider below board.
        const DIVIDER_LEN: usize = ((2 * WIDTH) - 1) as usize;
        writeln!(f, "{:-<len$}", "", len = DIVIDER_LEN)?;

        for i in 1..=WIDTH {
            if i > 1 {
                f.write_str(" ")?;
            }
            write!(f, "{}", i)?;
        }

        Ok(())
    }
}

pub struct PeekableBoard<'a> {
    board: &'a mut Board,
}

impl<'a> PeekableBoard<'a> {
    pub fn peek(&mut self, column: Column) -> PeekableBoard {
        self.board.make_move(column);

        PeekableBoard { board: self.board }
    }
}

impl<'a> Drop for PeekableBoard<'a> {
    fn drop(&mut self) {
        self.board.undo_move();
    }
}

impl<'a> Deref for PeekableBoard<'a> {
    type Target = Board;

    fn deref(&self) -> &Self::Target {
        self.board
    }
}

pub struct LegalMoves {
    board: BitBoard,
    heights: [Column; WIDTH as usize],
    column: Column,
}

impl Iterator for LegalMoves {
    type Item = Column;

    fn next(&mut self) -> Option<Self::Item> {
        while self.column < WIDTH {
            let column = self.column;
            self.column += 1;

            if Board::is_legal_board(self.board | (1 << self.heights[column as usize])) {
                return Some(column);
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(WIDTH as usize))
    }
}
