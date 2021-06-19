#![feature(bindings_after_at)]

use std::fmt::{self, Display};
use std::io::{self, Write};

pub type Pos = (usize, usize);

#[derive(Debug)]
pub struct GameBoard<const S: usize> {
    table: [Spot; S],
    player: Player,
    cols: usize,
}

impl<const S: usize> GameBoard<S> {
    //
    //
    pub fn new(cols: usize) -> Result<GameBoard<S>, Malformed> {
        if S % cols != 0 {
            return Err(Malformed);
        }

        Ok(GameBoard {
            table: [Spot::default(); S],
            player: Player::X,
            cols,
        })
    }

    //
    //
    pub fn turn(&mut self, pos: Pos) -> Result<Turn, BoardError> {
        let idx = self.set_piece(pos)?;

        if self.check_win(idx) {
            Ok(Turn::Win(self.player.into()))
        } else if self.check_draw() {
            Ok(Turn::Draw)
        } else {
            self.toggle_player();
            Ok(Turn::Next)
        }
    }

    //
    //
    fn convert_to_linear(&self, pos: Pos) -> Result<usize, Bound> {
        let rows = self.table.len() / self.cols;
        let row_oob = pos.0 >= rows;
        let col_oob = pos.1 >= self.cols;

        match (row_oob, col_oob) {
            (true, true) => Err(Bound::Both),
            (true, false) => Err(Bound::Row),
            (false, true) => Err(Bound::Col),
            (false, false) => Ok(pos.1 + self.cols * pos.0),
        }
    }

    //
    //
    fn set_piece(&mut self, pos: Pos) -> Result<usize, BoardError> {
        let idx = self.convert_to_linear(pos)?;
        let spot = self.table.get_mut(idx).expect("Index is out of bounds!");

        if let Spot::Empty = *spot {
            *spot = Spot::Occupied(self.player.into());
            Ok(idx)
        } else {
            Err(BoardError::Occupied)
        }
    }

    //
    //
    fn check_horizontal(&self, idx: usize) -> bool {
        let start = (idx / self.cols) * self.cols;
        let end = start + self.cols;
        let row = &self.table[start..end];

        row.iter()
            .all(|spot| Spot::Occupied(self.player.into()) == *spot)
    }

    //
    //
    fn check_vertical(&self, idx: usize) -> bool {
        let mut col = self.table.iter().enumerate().filter_map(|(col, spot)| {
            let remainder = |n| n % self.cols;

            if remainder(col) == remainder(idx) {
                Some(spot)
            } else {
                None
            }
        });

        col.all(|spot| Spot::Occupied(self.player.into()) == *spot)
    }

    //
    //
    fn check_diagonal(&self, _idx: usize) -> bool {
        // TODO
        false
    }

    //
    //
    fn check_win(&self, idx: usize) -> bool {
        self.check_horizontal(idx) || self.check_vertical(idx) || self.check_diagonal(idx)
    }

    //
    //
    fn check_draw(&self) -> bool {
        self.table.iter().all(|spot| Spot::Empty != *spot)
    }

    //
    //
    pub fn print(&self) {
        let rows = self.table.chunks(self.cols);

        for (n, row) in rows.enumerate() {
            if let Some((last, rest)) = row.split_last() {
                rest.iter().for_each(|spot| print!("{}|", spot));
                println!("{}", last);
            }

            if n != self.cols - 1 {
                println!("{:-^5}", "-");
            }
        }
    }

    //
    //
    #[inline]
    pub fn current_player(&self) -> &Player {
        &self.player
    }

    //
    //
    #[inline]
    fn toggle_player(&mut self) {
        match self.player {
            Player::O => self.player = Player::X,
            Player::X => self.player = Player::O,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Spot {
    Empty,
    Occupied(Piece),
}

impl Display for Spot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Spot::Empty => write!(f, " "),
            Spot::Occupied(piece) => write!(f, "{}", piece),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Piece {
    X,
    O,
}

impl Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Piece::X => write!(f, "X"),
            Piece::O => write!(f, "O"),
        }
    }
}

impl Default for Spot {
    fn default() -> Self {
        Self::Empty
    }
}

impl From<Player> for Piece {
    fn from(player: Player) -> Self {
        match player {
            Player::X => Piece::X,
            Player::O => Piece::O,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    X,
    O,
}

impl Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::X => write!(f, "X"),
            Self::O => write!(f, "O"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    Win(Player),
    Draw,
    Next,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    Row,
    Col,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardError {
    OutOfBounds(Bound),
    Occupied,
}

impl From<Bound> for BoardError {
    fn from(bound: Bound) -> Self {
        Self::OutOfBounds(bound)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Malformed;

fn play_loop<const S: usize>(mut board: GameBoard<S>) {
    let mut turn = Turn::Next;

    while let Turn::Next = turn {
        println!("Current player turn is: {}", board.current_player());
        let pos = player_input().expect("Something went wrong!");

        match board.turn(pos) {
            Err(err) => match err {
                BoardError::Occupied => println!("This position is already occupied!"),
                BoardError::OutOfBounds(_) => {
                    println!("Provided coordinates are out of the board!")
                }
            },

            Ok(win @ Turn::Win(player)) => {
                println!("Game over, player: {} won!", player);
                turn = win;
            }
            Ok(draw @ Turn::Draw) => {
                println!("Game over, it's a draw!");
                turn = draw;
            }
            _ => (),
        }

        board.print();
    }
}

fn player_input() -> Result<(usize, usize), io::Error> {
    loop {
        print!("Inform position (row column): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let matches = input
            .trim()
            .split_whitespace()
            .filter_map(|s| usize::from_str_radix(s, 10).ok())
            .take(2)
            .collect::<Vec<_>>();

        if matches.len() >= 2 {
            return Ok((matches[0], matches[1]));
        } else if matches.len() == 1 {
            println!("Need to inform both coordinates! (row and column)");
        } else {
            println!("Could not convert provided input to coordinates!");
        }
    }
}

fn main() {
    let cols = 3;
    const BOARD_LEN: usize = 9;

    let board = GameBoard::<BOARD_LEN>::new(cols)
        .expect("Cannot construct a board with the specified paramaters");

    play_loop(board);
}
