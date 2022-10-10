use std::time::Instant;
use ai_player::AIPlayer;
use game::BattleshipGame;

mod game;
mod ai_player;


fn main() {
    let player1 = AIPlayer::new();
    let player2 = AIPlayer::new();

    let mut games_played: u32 = 0;
    let mut game = BattleshipGame::new(Box::new(player1), Box::new(player2));

    let start_time = Instant::now();

    loop {
        game.run_game();
        games_played += 1;
        if games_played % 10 == 0 {
            println!( "{} games played in {}ms", games_played, Instant::now().duration_since(start_time).as_millis())

        }
    }
}
