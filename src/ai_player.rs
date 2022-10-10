use crate::game::{AimingBoard, BOARD_SIZE, Player, SHIP_LENGTHS, TargetBoard};

use rand::distributions::WeightedIndex;
use rand::prelude::*;

const INITIAL_WEIGHT: u32 = 1000;
const BASE_WEIGHT: u32 = 10;

// used to determine whether a particular set of weights should be added to the output or not
fn multiply_arrays(bool_arr: &[bool; BOARD_SIZE], num_arr: [u32; BOARD_SIZE]) -> [u32; BOARD_SIZE]{
    let result: Vec<u32> = num_arr.iter().zip(bool_arr.iter()).map(|(num, b)| num * match b {true => 1, false => 0,}).collect();
    return result.try_into().unwrap()
}
// used to remove cells which have been fired upon as legitimate targets
fn multiply_arrays_inverse(bool_arr: &[bool; BOARD_SIZE], num_arr: [u32; BOARD_SIZE]) -> [u32; BOARD_SIZE]{
    let result: Vec<u32> = num_arr.iter().zip(bool_arr.iter()).map(|(num, b)| num * match b {true => 0, false => 1,}).collect();
    return result.try_into().unwrap()
}

// used to add weights to the weights total
fn add_arrays(array1: [u32; BOARD_SIZE], array2: [u32; BOARD_SIZE]) -> [u32; BOARD_SIZE]{
    let array3: Vec<u32> = array1.iter().zip(array2.iter()).map(|(a, b)|a + b).collect();
    return array3.try_into().unwrap();
}

// struct to record which actions have been taken with particular inputs, for adjusting weights
struct Action {
    hits_input: [bool; BOARD_SIZE],
    misses_input: [bool; BOARD_SIZE],
    shot_taken: usize
}

impl Action {
    fn new(hits: [bool; BOARD_SIZE], misses: [bool; BOARD_SIZE], shot: usize) -> Self {
        Self {
            hits_input: hits,
            misses_input: misses,
            shot_taken: shot
        }
    }
}

pub struct AIPlayer {
    base_weights: [u32; BOARD_SIZE],
    hits_weights: [[u32; BOARD_SIZE]; BOARD_SIZE],
    misses_weights: [[u32; BOARD_SIZE]; BOARD_SIZE],
    possible_shots: [usize; BOARD_SIZE],
    actions: Vec<Action>
}

impl AIPlayer {
    pub fn new() -> Self {
        Self {
            base_weights: [BASE_WEIGHT; BOARD_SIZE],
            hits_weights: [[INITIAL_WEIGHT; BOARD_SIZE]; BOARD_SIZE],
            misses_weights: [[INITIAL_WEIGHT; BOARD_SIZE]; BOARD_SIZE],
            possible_shots: core::array::from_fn(|i| i),
            actions: vec![]
        }
    }
}

impl Player for AIPlayer {
    fn new_game(mut self: &mut AIPlayer) {
        self.actions = vec![]
    }

    fn place_ships(self: &mut AIPlayer) -> TargetBoard {
        // reset actions because place_ships is called at the start of each game
        let mut target_board: TargetBoard = TargetBoard::new();
        for i in 0..SHIP_LENGTHS.len() {
            target_board.place_ship(i as u32, 0, SHIP_LENGTHS[i], true);
        }
        return target_board;
    }

    fn take_shot(mut self: &mut AIPlayer, aiming_board: &AimingBoard) -> usize{
        // add an initial weight to each cell
        let mut shot_weights = self.base_weights.clone();
        // determine weights for each cell based on hits and misses
        for cell_number in 0..BOARD_SIZE {
            shot_weights = add_arrays(shot_weights, multiply_arrays(aiming_board.get_hits(), self.hits_weights[cell_number]));
            shot_weights = add_arrays(shot_weights, multiply_arrays(aiming_board.get_misses(), self.misses_weights[cell_number]));
        }
        // remove cells which have been shot at already
        shot_weights = multiply_arrays_inverse(aiming_board.get_hits(), shot_weights);
        shot_weights = multiply_arrays_inverse(aiming_board.get_misses(), shot_weights);
        // use weightedIndex to choose a shot to take based on the random weights which have been generated
        let chosen_shot: usize = self.possible_shots[WeightedIndex::new(shot_weights).unwrap().sample(&mut thread_rng())];
        // record the action taken to use it for adjusting weights later
        self.actions.push(Action::new(aiming_board.get_hits().clone(), aiming_board.get_misses().clone(), chosen_shot));
        return chosen_shot;
    }

    fn game_finish(mut self: &mut AIPlayer, won: bool) {

    }
}
