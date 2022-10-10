use crate::ai_player::AIPlayer;
use crate::game::run_game;

mod game;
mod ai_player;


fn main() {
    let player1 = AIPlayer::new();
    let player2 = AIPlayer::new();

    run_game(Box::new(player1), Box::new(player2));
}
