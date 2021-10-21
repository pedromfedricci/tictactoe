use tictactoe::{play_loop, GameBoard};

fn main() {
    const COLS: usize = 3;
    const LEN: usize = 9;

    let board = GameBoard::<LEN, COLS>::new().expect("Board construction failed");

    play_loop(board);
}
