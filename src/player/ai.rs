use std::collections::HashMap;
use std::ops::Range;

use rand::prelude::ThreadRng;
use rand::Rng;

use crate::board::{BitBoard, Board, Column, ColumnDiff, PeekableBoard, HEIGHT, WIDTH};
use crate::player::{Player, Token};

pub enum Difficulty {
    Easy = 3,
    Medium = 5,
    Hard = 7,
    Master = 9,
    Unfair = 11,
}

pub struct AIPlayer {
    depth: usize,
    ttable: TTable,
    rng: ThreadRng,
}

impl AIPlayer {
    pub fn new(difficulty: Difficulty) -> AIPlayer {
        let depth = difficulty as usize;

        AIPlayer {
            depth,
            ttable: TTable::with_capacity(depth * (WIDTH as usize)),
            rng: rand::thread_rng(),
        }
    }
}

impl Player for AIPlayer {
    fn decide_move(&mut self, board: &Board, token: Token) -> Column {
        let mut board = *board;

        let mut best_moves = [0; WIDTH as usize];
        let mut len_best_moves = 0;
        let mut value_best_move = Score::MIN;

        for column in board.legal_moves() {
            let value = negamax(
                &mut self.ttable,
                board.peekable(column),
                self.depth,
                Score::MIN,
                Score::MAX,
                token.opponent(),
            )
            .saturating_neg();

            match value_best_move.saturating_sub(value) {
                0 => {
                    best_moves[len_best_moves] = column;
                    len_best_moves += 1;
                }
                diff if diff < 0 => {
                    value_best_move = value;
                    best_moves[0] = column;
                    len_best_moves = 1;
                }
                _ => {}
            }
        }

        match len_best_moves {
            0 => panic!("no legal moves"),
            1 => best_moves[0],
            // Pick a move from the best moves at random.
            len => best_moves[self.rng.gen_range(0, len)],
        }
    }
}

type TTable = HashMap<BitBoard, TTEntry>;
type Score = i32;

struct TTEntry {
    depth: usize,
    value: Score,
    flag: TTFlag,
}

enum TTFlag {
    Exact,
    Lowerbound,
    Upperbound,
}

fn negamax(
    ttable: &mut TTable,
    mut board: PeekableBoard,
    depth: usize,
    mut a: Score,
    mut b: Score,
    side: Token,
) -> Score {
    let a_orig = a;

    let position_code = board.position_code();

    // Look up board in transposition table.
    match ttable.get(&position_code) {
        Some(entry) if entry.depth >= depth => {
            match entry.flag {
                TTFlag::Exact => return entry.value,
                TTFlag::Lowerbound => a = a.max(entry.value),
                TTFlag::Upperbound => b = b.min(entry.value),
            }

            if a >= b {
                return entry.value;
            }
        }
        _ => {}
    }

    let mut legal_moves = board.legal_moves().peekable();

    // If reached max depth or at a terminal board state, return heuristic value.
    {
        let winner = board.winner();
        let is_full = legal_moves.peek().is_none();

        if depth == 0 || (winner.is_some() || is_full) {
            return heuristic_value(&board, side, winner, is_full);
        }
    }

    let mut value = Score::MIN;
    for column in legal_moves {
        value = value.max(
            negamax(
                ttable,
                board.peek(column),
                depth.saturating_sub(1),
                b.saturating_neg(),
                a.saturating_neg(),
                side.opponent(),
            )
            .saturating_neg(),
        );
        a = a.max(value);

        if a >= b {
            break;
        }
    }

    // Store board in transposition table.
    let flag = if value <= a_orig {
        TTFlag::Upperbound
    } else if value >= b {
        TTFlag::Lowerbound
    } else {
        TTFlag::Exact
    };
    let entry = TTEntry { depth, value, flag };
    ttable.insert(position_code, entry);

    value
}

fn heuristic_value(board: &Board, side: Token, winner: Option<Token>, is_full: bool) -> Score {
    const WIN: Score = 10_000;

    if let Some(winner) = winner {
        return if winner == side { WIN } else { -WIN };
    }

    // If the board is full at this point, the game is a draw.
    if is_full {
        return 0;
    }

    let mut total_score = 0;

    for column in 0..WIDTH {
        for row in 0..HEIGHT {
            if let Some(token) = board.token_at(row, column) {
                const DIRECTION: [(ColumnDiff, ColumnDiff); 4] = [(1, 0), (1, 1), (0, 1), (-1, 1)];

                for &(i, j) in &DIRECTION {
                    let forward = get_length(board, (row, column), (i, j), token);
                    let backward = get_length(board, (row, column), (-i, -j), token);

                    let current_len = forward.0 + backward.0 + 1;
                    let possible_len = forward.1 + backward.1 + 1;

                    if possible_len >= 4 {
                        let score = 10 * Score::from(current_len);
                        if side == token {
                            total_score += score;
                        } else {
                            total_score -= score;
                        }
                    }
                }
            }
        }
    }

    total_score
}

fn get_length(
    board: &Board,
    pos: (Column, Column),
    direction: (ColumnDiff, ColumnDiff),
    side: Token,
) -> (Column, Column) {
    let mut current = 0;
    let mut possible = 0;

    let mut row = pos.0 as i8;
    let mut column = pos.1 as i8;

    const ROWS: Range<i8> = 0..(HEIGHT as i8);
    const COLUMNS: Range<i8> = 0..(WIDTH as i8);

    loop {
        row += direction.0;
        column += direction.1;

        // Check the cell is inbounds, this is optimised in release builds.
        if !(ROWS.contains(&row) && COLUMNS.contains(&column)) {
            break;
        }

        match board.token_at(row as Column, column as Column) {
            Some(token) if token == side => current += 1,
            Some(_) => break,
            _ => {}
        }

        possible += 1;
    }

    (current, possible)
}
