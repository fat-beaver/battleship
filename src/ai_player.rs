use nalgebra::{RowVector, SMatrix, SVector};
use crate::game::{AimingBoard, BOARD_SIZE, Player, SHIP_LENGTHS, TargetBoard};

use rand::distributions::WeightedIndex;
use rand::prelude::*;

const INITIAL_WEIGHT: f64 = 10.0;
const STANDARD_WEIGHT_MODIFICATION_FACTOR: f64 = 0.0001;
const BASE_WEIGHT: f64 = 1.0;
const BASE_WEIGHT_MODIFICATION_FACTOR: f64 = 0.00001;

/// struct to record which actions have been taken with particular inputs, for adjusting weights
#[derive(Clone)]
struct Action {
    hits_input: Box<SVector<f64, BOARD_SIZE>>,
    misses_input: Box<SVector<f64, BOARD_SIZE>>,
    shot_taken: usize
}

impl Action {
    fn new(hits: SVector<f64, BOARD_SIZE>, misses: SVector<f64, BOARD_SIZE>, shot: usize) -> Self {
        Self {
            hits_input: Box::from(hits),
            misses_input: Box::from(misses),
            shot_taken: shot
        }
    }
}

#[derive(Clone)]
pub struct AIPlayer {
    base_weights: SVector<f64, BOARD_SIZE>,
    hits_weights: Box<SMatrix<f64, BOARD_SIZE, BOARD_SIZE>>,
    misses_weights: Box<SMatrix<f64, BOARD_SIZE, BOARD_SIZE>>,
    possible_shots: [usize; BOARD_SIZE],
    actions: Vec<Action>
}

impl AIPlayer {
    pub fn new() -> Self {
        Self {
            base_weights: SMatrix::repeat(BASE_WEIGHT),
            hits_weights: Box::from(SMatrix::repeat(INITIAL_WEIGHT)),
            misses_weights: Box::from(SMatrix::repeat(INITIAL_WEIGHT)),
            possible_shots: core::array::from_fn(|i| i),
            actions: Vec::new()
        }
    }
    fn merge(&mut self, other: &Self) where Self: Sized {
        self.base_weights += other.base_weights.clone();
        *self.misses_weights += *other.misses_weights.clone();
        *self.hits_weights += *other.hits_weights.clone();
        self.base_weights /= 2.0;
        *self.misses_weights /= 2.0;
        *self.hits_weights /= 2.0;
    }

    fn clone_from(&mut self, other: &Self) where Self: Sized {
        self.base_weights = other.base_weights.clone();
        self.misses_weights = other.misses_weights.clone();
        self.hits_weights = other.hits_weights.clone();
    }
}

impl Player for AIPlayer {
    fn new_game(&mut self) {
        self.actions = Vec::new()
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
        let mut shot_weights: SVector<f64, BOARD_SIZE> = self.base_weights.clone();
        // determine weights for each cell based on hits and misses
        shot_weights += *self.hits_weights.clone() * aiming_board.get_hits();
        shot_weights += *self.misses_weights.clone() * aiming_board.get_misses();
        // remove cells which have been shot at already
        shot_weights = shot_weights.component_mul(aiming_board.get_targetable());
        // use weightedIndex to choose a shot to take based on the random weights which have been generated
        let chosen_shot: usize = self.possible_shots[WeightedIndex::new(shot_weights.iter()).unwrap().sample(&mut thread_rng())];
        // record the action taken to use it for adjusting weights later
        self.actions.push(Action::new(aiming_board.get_hits().clone(), aiming_board.get_misses().clone(), chosen_shot.clone()));
        return chosen_shot;
    }

    fn game_finish(&mut self, won: bool) {
        for action in self.actions.clone() {
            self.base_weights[action.shot_taken] += match won {
                true => BASE_WEIGHT_MODIFICATION_FACTOR,
                false => -BASE_WEIGHT_MODIFICATION_FACTOR
            };
            for i in self.hits_weights.row_mut(action.shot_taken).iter_mut() {
                *i += match won {
                    true => STANDARD_WEIGHT_MODIFICATION_FACTOR,
                    false => -STANDARD_WEIGHT_MODIFICATION_FACTOR
                };
            }
        }
    }
}
