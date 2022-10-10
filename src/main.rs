use ai_player::AIPlayer;
use game::{BattleshipGame, BOARD_SIZE};
use rayon::{prelude::*, ThreadPoolBuilder};
use std::{mem::size_of, time::Instant, env};

mod ai_player;
mod game;

fn run_for(parallel_games: usize, amount: usize, epoch_size: usize) -> BattleshipGame<AIPlayer, AIPlayer> {
    println!("RUNNING WITH {} GAMES, EPOCH OF {}, FOR {} EPOCHS", parallel_games, epoch_size, amount);
    let mut games = Vec::new();

    for _ in 0..parallel_games {
        games.push(BattleshipGame::new(AIPlayer::new(), AIPlayer::new()));
    }

    let mut epoch = 0;

    let start = Instant::now();

    loop {
        epoch += 1;
        games
            .par_iter_mut()
            .for_each(|game| game.run_for(epoch_size));

        games.iter_mut().reduce(|a, b| {
            a.merge_games(b);
            a
        });

        games.iter_mut().rev().reduce(|a, b| {
            b.clone_from(a);
            a
        });

        let duration = Instant::now().duration_since(start).as_secs_f64();

        println!(
            "done epoch {}, in {}ms, average games/s: {}, game/s/thread: {}",
            epoch,
            (duration * 1000.0) as u64,
            (epoch_size * epoch * parallel_games) as f64 / duration,
            (epoch_size * epoch) as f64 / duration
        );
        if epoch >= amount {
            break;
        }
    }

    games.into_iter().reduce(|mut a,b| {a.merge_games(&b); a}).unwrap()
}

fn main() {
    let args: Vec<usize> = env::args().skip(1).map(|s| s.parse::<usize>().unwrap()).collect();
    
    let parallel = *args.get(0).unwrap_or(&6);
    let amount = *args.get(1).unwrap_or(&10);
    let epoch = *args.get(2).unwrap_or(&200);
    
    ThreadPoolBuilder::new()
        .stack_size(size_of::<BattleshipGame<AIPlayer, AIPlayer>>())
        .num_threads(parallel)
        .build_global()
        .unwrap();

    println!("final game:\n{:?}", run_for(parallel, amount, epoch));
}
