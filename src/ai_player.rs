use nalgebra::{SVector, DMatrix, SMatrix};
use crate::game::{AimingBoard, BOARD_SIZE, Player, SHIP_LENGTHS, TargetBoard};

use rand::distributions::WeightedIndex;
use rand::prelude::*;

const INITIAL_WEIGHT: u32 = 1000;
const BASE_WEIGHT: u32 = 10;

#[derive(Debug, Clone)]
/// struct to record which actions have been taken with particular inputs, for adjusting weights
struct Action {
    hits_input: SVector<u32, BOARD_SIZE>,
    misses_input: SVector<u32, BOARD_SIZE>,
    shot_taken: usize
}

impl Action {
    fn new(hits: SVector<u32, BOARD_SIZE>, misses: SVector<u32, BOARD_SIZE>, shot: usize) -> Self {
        Self {
            hits_input: hits,
            misses_input: misses,
            shot_taken: shot
        }
    }
}

#[derive(Debug, Clone)]
pub struct AIPlayer {
    base_weights: SVector<u32, BOARD_SIZE>,
    hits_weights: SMatrix<u32, BOARD_SIZE, BOARD_SIZE>,
    misses_weights: SMatrix<u32, BOARD_SIZE, BOARD_SIZE>,
    possible_shots: [usize; BOARD_SIZE],
    actions: Vec<Action>
}

impl AIPlayer {
    pub fn new() -> Self {
        Self {
            base_weights: SVector::repeat(BASE_WEIGHT),
            hits_weights: SMatrix::repeat(INITIAL_WEIGHT),
            misses_weights: SMatrix::repeat(INITIAL_WEIGHT),
            possible_shots: core::array::from_fn(|i| i),
            actions: vec![]
        }
    }
}

impl Player for AIPlayer {
    fn new_game(mut self: &mut AIPlayer) {
        self.actions = vec![]
    }

    fn place_ships(&mut self) -> TargetBoard {
        // reset actions because place_ships is called at the start of each game
        let mut target_board: TargetBoard = TargetBoard::new();
        for i in 0..SHIP_LENGTHS.len() {
            target_board.place_ship(i as u32, 0, SHIP_LENGTHS[i], true);
        }
        return target_board;
    }

    fn take_shot(&mut self, aiming_board: &AimingBoard) -> usize{
        // add an initial weight to each cell
        let mut shot_weights: SVector<u32, BOARD_SIZE> = self.base_weights.clone();
        // determine weights for each cell based on hits and misses
        
        shot_weights += self.hits_weights.clone() * aiming_board.get_hits();
        shot_weights += self.misses_weights.clone() * aiming_board.get_misses();

        // remove cells which have been shot at already
        shot_weights.component_mul_assign(aiming_board.get_targetable());
        // use weightedIndex to choose a shot to take based on the random weights which have been generated
        let chosen_shot: usize = self.possible_shots[WeightedIndex::new(shot_weights.iter()).unwrap().sample(&mut thread_rng())];
        // record the action taken to use it for adjusting weights later
        self.actions.push(Action::new(aiming_board.get_hits().clone(), aiming_board.get_misses().clone(), chosen_shot));
        return chosen_shot;
    }

    fn game_finish(&mut self, won: bool) {

    }

    fn merge(&mut self, other: &Self) where Self: Sized {
        self.base_weights += other.base_weights;
        self.misses_weights += other.misses_weights.clone();
        self.hits_weights += other.hits_weights.clone();
        self.base_weights /= 2;
        self.misses_weights /= 2;
        self.hits_weights /= 2;
    }

    fn clone_from(&mut self, other: &Self) where Self: Sized {
        self.base_weights = other.base_weights;
        self.misses_weights = other.misses_weights.clone();
        self.hits_weights = other.hits_weights.clone();
    }
}
