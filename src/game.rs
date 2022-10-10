use std::any::Any;

use nalgebra::SVector;
use rayon::prelude::*;

pub const BOARD_WIDTH: u32 = 10;
pub const BOARD_HEIGHT: u32 = 10;
pub const BOARD_SIZE: usize = (BOARD_WIDTH * BOARD_HEIGHT) as usize;

pub const SHIP_LENGTHS: [u32; 5] = [5, 4, 3, 3, 2];

pub const TOTAL_SHIP_HEALTH: u32 = 17;  // the total of the ship lengths

#[derive(Debug, Clone)]
pub struct AimingBoard {
    hits: SVector<u32, BOARD_SIZE>,
    misses: SVector<u32, BOARD_SIZE>,
    targetable: SVector<u32, BOARD_SIZE>
}

impl AimingBoard {
    fn new() -> Self {
        Self {
            hits: SVector::repeat(0),
            misses: SVector::repeat(0),
            targetable: SVector::repeat(1)
        }
    }
    pub fn get_hits(&self) -> &SVector<u32, BOARD_SIZE> {
        return &self.hits;
    }
    pub fn get_misses(&self) -> &SVector<u32, BOARD_SIZE> {
        return &self.misses;
    }
    pub fn get_targetable(&self) -> &SVector<u32, BOARD_SIZE> {
        return &self.targetable;
    }
}

#[derive(Debug, Clone)]
pub struct Ship {
    x_coord: u32,
    y_coord: u32,
    length: u32,
    vertical: bool
}
impl Ship {
    pub fn new(x: u32, y: u32, len: u32, vert: bool) -> Self {
        Self {
            x_coord: x,
            y_coord: y,
            length: len,
            vertical: vert
        }
    }
}

#[derive(Debug, Clone)]
pub struct TargetBoard {
    ships: Vec<Ship>,
    ships_required: Vec<u32>,
    cells_containing_ships: [bool; BOARD_SIZE]
}

impl TargetBoard {
    pub fn new() -> Self {
        Self {
            ships: vec![],
            ships_required: Vec::from(SHIP_LENGTHS),
            cells_containing_ships: [false; BOARD_SIZE]
        }
    }
    pub fn place_ship(&mut self, x_coord: u32, y_coord: u32, ship_length: u32, vertical: bool) {
        if self.ships_required.contains(&ship_length) &&
            x_coord < BOARD_WIDTH && y_coord < BOARD_HEIGHT {
            // remove the ship from the list of ships to add and add it to the list of ships
            self.ships_required.remove(self.ships_required.iter().position(|x| *x == ship_length).unwrap());
            // create a ship object and add it to the target board
            let new_ship: Ship = Ship::new(x_coord, y_coord, ship_length, vertical);
            self.ships.push(new_ship);
            // determine which cells this ship occupies
            if vertical {
                for i in 0..ship_length {
                    self.cells_containing_ships[((i + y_coord) * BOARD_WIDTH + x_coord) as usize] = true;
                }
            } else {
                for i in 0..ship_length {
                    self.cells_containing_ships[(y_coord * BOARD_WIDTH + x_coord + i) as usize] = true;
                }
            }
        }
    }
    pub fn check_hit(&self, cell_number: usize) -> bool{
        return self.cells_containing_ships[cell_number];
    }
}

pub trait Player: Send + Sync {
    fn new_game(&mut self);
    fn place_ships(&mut self) -> TargetBoard;
    fn take_shot(&mut self, aiming_board: &AimingBoard) -> usize;
    fn game_finish(&mut self, won: bool);
    fn merge(&mut self, other: &Self) where Self: Sized;
    fn clone_from(&mut self, other: &Self) where Self: Sized;
}

struct InternalPlayer<T: Player> {
    player: T,
    hits_left: u32,
    aiming_board: AimingBoard,
    target_board: TargetBoard
}

impl<T: Player> InternalPlayer<T> {
    fn new(mut p: T) -> Self {
            Self {
            target_board: p.place_ships(),
            player: p,
            hits_left: TOTAL_SHIP_HEALTH,
            aiming_board: AimingBoard::new()
        }
    }
    fn reset(&mut self) {
        self.player.new_game();
        self.target_board = self.player.place_ships();
        self.hits_left = TOTAL_SHIP_HEALTH;
        self.aiming_board = AimingBoard::new();
    }
    fn take_shot(&mut self) -> usize {
        self.player.take_shot(&self.aiming_board)
    }
}

macro_rules! player_access {
    (ref $fn_name:ident, $field:ident, $ty:ty) => {
        pub fn $fn_name(&self, idx: usize) -> & $ty {
            match idx {
                0 => &self.player_a.$field,
                1 => &self.player_b.$field,
                _ => panic!(),
            }
        }
    };
    (mut $fn_name:ident, $field:ident, $ty:ty) => {
        pub fn $fn_name(&mut self, idx: usize) -> &mut $ty {
            match idx {
                0 => &mut self.player_a.$field,
                1 => &mut self.player_b.$field,
                _ => panic!(),
            }
        }
    };
}

pub struct BattleshipGame<A, B> where A: Player, B: Player {
    player_a: InternalPlayer<A>,
    player_b: InternalPlayer<B>,
    played: usize,
}

impl<A, B> BattleshipGame<A, B> where A: Player, B: Player {
    pub fn new(p1:A, p2: B) -> BattleshipGame<A, B> {
        Self {
            player_a: InternalPlayer::new(p1), 
            player_b: InternalPlayer::new(p2),
            played: 0,
        }
    }

    player_access!(ref player, player, dyn Player);
    player_access!(mut player_mut, player, dyn Player);
    player_access!(ref hits_left, hits_left, u32);
    player_access!(mut hits_left_mut, hits_left, u32);
    player_access!(ref aiming_board, aiming_board, AimingBoard);
    player_access!(mut aiming_board_mut, aiming_board, AimingBoard);
    player_access!(ref target_board, target_board, TargetBoard);
    player_access!(mut target_board_mut, target_board, TargetBoard);

    pub fn run_game(&mut self) {
        self.player_a.reset();
        self.player_b.reset();

        let mut current_player_id: usize = 0;
        let mut _turns_taken: usize = 0;

        // continue running until a player has lost
        while self.player_a.hits_left > 0 && self.player_b.hits_left > 0 {
            // increase the counter of turns taken each time player 1 takes their turn
            if current_player_id == 0 {
                _turns_taken += 1;
            }
            // ask the current player to take a shot with the information in their aiming board
            let shot_taken: usize = match current_player_id {
                0 => self.player_a.take_shot(),
                1 => self.player_b.take_shot(),
                _ => panic!(),
            };
            if self.target_board((current_player_id + 1).rem_euclid(2)).check_hit(shot_taken) {
                self.aiming_board_mut(current_player_id).hits[shot_taken] = 1;
                // take one hit away from the player who was hit
                *self.hits_left_mut((current_player_id + 1).rem_euclid(2)) -= 1;
            } else {
                self.aiming_board_mut(current_player_id).hits[shot_taken] = 0;
            }
            self.aiming_board_mut(current_player_id).targetable[shot_taken] = 0;
            // switch to the other player
            current_player_id = (current_player_id + 1).rem_euclid(2);
        }
        // inform the players of their victory/loss when the game ends
        if self.player_a.hits_left == 0 {
            self.player_a.player.game_finish(false);
            self.player_b.player.game_finish(true);
        } else {
            self.player_a.player.game_finish(true);
            self.player_b.player.game_finish(false);
        }
        self.played += 1;
    }

    pub fn run_for(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.run_game()
        }
    }

    pub fn into_players(self) -> (A,B) {
        (self.player_a.player, self.player_b.player)
    }

    pub fn merge_games(&mut self, other: &Self) {
        self.player_a.player.merge(&other.player_a.player);
        self.player_b.player.merge(&other.player_b.player);
    }

    pub fn clone_from(&mut self, other: &Self) {
        self.player_a.player.clone_from(&other.player_a.player);
        self.player_b.player.clone_from(&other.player_b.player);
    }
}
