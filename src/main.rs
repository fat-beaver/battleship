use std::sync::mpsc;
use std::thread;

use std::time::Instant;
use ai_player::AIPlayer;
use game::BattleshipGame;

mod game;
mod ai_player;

const SERIAL_GAMES: usize = 1000;
const TO_PLAY: usize = 1000000000;

fn main() {
    let parallel_games = 7;

    let start_time = Instant::now();

    let mut games_played: usize = 0;

    let mut games = Vec::new();

    let (tx, rx) = mpsc::channel();

    for _ in 0..parallel_games {
        games.push(BattleshipGame::new(AIPlayer::new(), AIPlayer::new()));
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
        games_played += SERIAL_GAMES;
        let duration = Instant::now().duration_since(start_time).as_secs_f64();
        println!("{} games played in {}s, {} per second", games_played, duration, games_played as f64 / duration);
        if games_played >= TO_PLAY {
            break;
        }
        let tx1 = tx.clone();
        thread::spawn(move || {
            game.run_multiple(SERIAL_GAMES);
            tx1.send(game).unwrap();
        });
    }
}
