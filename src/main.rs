use ai_player::AIPlayer;
use game::{BattleshipGame, Player};
use rayon::{prelude::*, iter::Zip};
use std::{any::Any, sync::atomic::AtomicU32, time::Instant};

mod ai_player;
mod game;

const PARALLEL_GAMES: usize = 4;
const EPOCH_SIZE: usize = 1000;

fn main() {
    let mut games = Vec::new();

    for _ in 0..PARALLEL_GAMES {
        games.push(BattleshipGame::new(AIPlayer::new(), AIPlayer::new()));
    }

    let mut epoch = 0;

    let start = Instant::now();

    loop {
        games.par_iter_mut().for_each(|game| game.run_for(EPOCH_SIZE));

        games.iter_mut().reduce(|a,b| { a.merge_games(b); a });
        
        games.iter_mut().rev().reduce(|a,b| {b.clone_from(a); a});

        let duration = Instant::now().duration_since(start).as_secs_f64();

        println!("done epoch {}, in {}ms, average games/s: {}", epoch, (duration * 1000.0) as u64, (EPOCH_SIZE * (epoch + 1) * PARALLEL_GAMES) as f64 / duration);
        epoch += 1;
    }
}
