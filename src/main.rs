use std::sync::mpsc;
use std::thread;
use std::thread::{JoinHandle};
use std::time::Instant;
use ai_player::AIPlayer;
use game::BattleshipGame;

mod game;
mod ai_player;

const PARALLEL_GAMES: usize = 4;
const SERIAL_GAMES: usize = 100;

fn main() {
    let parallel_games = num_cpus::get() - 1;

    let player1 = AIPlayer::new();
    let player2 = AIPlayer::new();

    let start_time = Instant::now();

    let mut games_played: usize = 0;

    let mut games: Vec<BattleshipGame> = vec![];

    let (tx, rx) = mpsc::channel();


    for _ in 0..parallel_games {
        games.push(BattleshipGame::new(Box::new(player1.clone()), Box::new(player2.clone())));
    }

    while !games.is_empty() {
        let mut game = games.pop().unwrap();
        let tx1 = tx.clone();
        thread::spawn(move || {
            game.run_multiple(SERIAL_GAMES);
            tx1.send(game).unwrap();
        });
    }
    for mut game in rx {
        games_played += parallel_games * SERIAL_GAMES;
        let duration = Instant::now().duration_since(start_time).as_secs_f64();
        println!("{} games played in {}s, {} per second", games_played, duration, games_played as f64 / duration as f64);
        let tx1 = tx.clone();
        thread::spawn(move || {
            game.run_multiple(SERIAL_GAMES);
            tx1.send(game).unwrap();
        });
    }
}
