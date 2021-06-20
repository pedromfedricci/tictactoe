#![feature(bindings_after_at)]

use std::fmt::{self, Display};
use std::io::{self, Write};

pub type Pos = (usize, usize);

#[derive(Debug, Clone, Copy)]
pub struct GameBoard<const LEN: usize, const COLS: usize> {
    board: [Spot; LEN],
    player: Player,
}

impl<const LEN: usize, const COLS: usize> GameBoard<LEN, COLS> {
    //
    //
    pub fn new() -> Result<Self, MalformedError> {
        Self::constraints()?;

        Ok(GameBoard {
            board: [Spot::default(); LEN],
            player: Player::X,
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
        let rows = self.board.len() / COLS;
        let row_oob = pos.0 >= rows;
        let col_oob = pos.1 >= COLS;

        match (row_oob, col_oob) {
            (true, true) => Err(Bound::Both),
            (true, false) => Err(Bound::Row),
            (false, true) => Err(Bound::Col),
            (false, false) => Ok(pos.1 + COLS * pos.0),
        }
    }

    //
    //
    fn set_piece(&mut self, pos: Pos) -> Result<usize, BoardError> {
        let idx = self.convert_to_linear(pos)?;
        let spot = self.board.get_mut(idx).expect("Index is out of bounds!");

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
        let start = (idx / COLS) * COLS;
        let end = start + COLS;

        self.board[start..end]
            .iter()
            .all(|spot| Spot::Occupied(self.player.into()) == *spot)
    }

    //
    //
    fn streak_line<F>(&self, f: F) -> bool
    where
        F: Fn(usize) -> bool,
    {
        self.board
            .iter()
            .enumerate()
            .filter_map(|(n, spot)| if f(n) { Some(spot) } else { None })
            .all(|spot| Spot::Occupied(self.player.into()) == *spot)
    }

    //
    //
    fn check_vertical(&self, idx: usize) -> bool {
        let remainder = |n| n % COLS;
        let same_col = |n| remainder(n) == remainder(idx);

        self.streak_line(same_col)
    }

    //
    //
    fn check_diagonals(&self, idx: usize) -> bool {
        if !Self::is_squared() {
            return false;
        }

        if Self::in_center(idx) {
            self.check_diagonal(Diagonal::Both)
        } else if Self::in_primary(idx) {
            self.check_diagonal(Diagonal::Primary)
        } else if Self::in_secondary(idx) {
            self.check_diagonal(Diagonal::Secondary)
        } else {
            false
        }
    }

    //
    //
    #[inline]
    fn check_diagonal(&self, diag: Diagonal) -> bool {
        match diag {
            Diagonal::Primary => self.streak_line(Self::in_primary),
            Diagonal::Secondary => self.streak_line(Self::in_secondary),
            Diagonal::Both => {
                self.streak_line(Self::in_primary) || self.streak_line(Self::in_secondary)
            }
        }
    }

    //
    //
    #[inline]
    fn check_win(&self, idx: usize) -> bool {
        self.check_horizontal(idx) || self.check_vertical(idx) || self.check_diagonals(idx)
    }

    //
    //
    #[inline]
    fn check_draw(&self) -> bool {
        self.board.iter().all(|spot| Spot::Empty != *spot)
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
    pub fn is_squared() -> bool {
        COLS.pow(2) == LEN
    }

    //
    //
    #[inline]
    pub fn in_primary(idx: usize) -> bool {
        idx % (COLS + 1) == 0
    }

    //
    //
    #[inline]
    pub fn in_secondary(idx: usize) -> bool {
        idx % (COLS - 1) == 0
    }

    //
    //
    #[inline]
    pub fn in_center(idx: usize) -> bool {
        Self::in_primary(idx) && Self::in_secondary(idx)
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

    #[inline]
    fn constraints() -> Result<(), MalformedError> {
        if COLS == 0 {
            Err(MalformedError::ColsZero)
        } else if LEN == 0 {
            Err(MalformedError::LenZero)
        } else {
            Ok(())
        }
    }
}

impl<const LEN: usize, const COLS: usize> Display for GameBoard<LEN, COLS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rows = self.board.chunks(COLS);
        let len = rows.len();

        for (n, row) in rows.enumerate() {
            if let Some((last, rest)) = row.split_last() {
                for spot in rest {
                    write!(f, "{}|", spot)?;
                }
                writeln!(f, "{}", last)?;
            }

            if n < len - 1 {
                writeln!(f, "{:-^width$}", "", width = COLS * 2 - 1)?;
            }
        }

        Ok(())
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
        match self {
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
enum Diagonal {
    Primary,
    Secondary,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardError {
    OutOfBounds(Bound),
    Occupied,
}

impl Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Occupied => write!(f, "This position is already occupied!"),
            Self::OutOfBounds(_) => write!(f, "Provided coordinates are out of the board!"),
        }
    }
}

impl From<Bound> for BoardError {
    fn from(bound: Bound) -> Self {
        Self::OutOfBounds(bound)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MalformedError {
    LenZero,
    ColsZero,
}

impl Display for MalformedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenZero => write!(f, "Cannot construct board with zero length"),
            Self::ColsZero => write!(f, "Cannot construct board with zero columns"),
        }
    }
}

fn play_loop<const LEN: usize, const COLS: usize>(mut board: GameBoard<LEN, COLS>) {
    println!("{}", board);

    let mut turn = Turn::Next;
    while let Turn::Next = turn {
        println!("Current player turn is: {}", board.current_player());
        let pos = player_input().expect("Something went wrong with I/O!");

        match board.turn(pos) {
            Ok(win @ Turn::Win(player)) => {
                println!("Game over, player: {} won!", player);
                turn = win;
            }
            Ok(draw @ Turn::Draw) => {
                println!("Game over, it's a draw!");
                turn = draw;
            }
            Ok(Turn::Next) => (),

            Err(err) => println!("{}", err),
        }

        println!("{}", board);
    }
}

fn player_input() -> Result<Pos, io::Error> {
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

        match matches.len() {
            len if len >= 2 => return Ok((matches[0], matches[1])),
            1 => println!("Need to inform both coordinates!"),
            _ => println!("Could not convert provided input to coordinates!"),
        }
    }
}

fn main() {
    const COLS: usize = 3;
    const LEN: usize = 9;

    let board = GameBoard::<LEN, COLS>::new().expect("Board construction failed");

    play_loop(board);
}
