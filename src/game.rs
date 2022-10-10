use nalgebra::SVector;

pub const BOARD_WIDTH: u32 = 10;
pub const BOARD_HEIGHT: u32 = 10;
pub const BOARD_SIZE: usize = (BOARD_WIDTH * BOARD_HEIGHT) as usize;

pub const SHIP_LENGTHS: [u32; 5] = [5, 4, 3, 3, 2];

pub const TOTAL_SHIP_HEALTH: u32 = 17;  // the total of the ship lengths

#[derive(Clone)]
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

pub struct TargetBoard {
    ships: Vec<Ship>,
    ships_required: Vec<u32>,
    cells_containing_ships: [bool; BOARD_SIZE]
}

impl TargetBoard {
    pub fn new() -> Self {
        Self {
            ships: Vec::new(),
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

pub trait Player: Send {
    fn new_game(&mut self);
    fn place_ships(&mut self) -> TargetBoard;
    fn take_shot(&mut self, aiming_board: &AimingBoard) -> usize;
    fn game_finish(&mut self, won: bool);
}

struct InternalPlayer  {
    player: Box<dyn Player>,
    hits_left: u32,
    aiming_board: AimingBoard,
    target_board: TargetBoard
}

impl InternalPlayer {
    fn new(mut p: Box<dyn Player>) -> Self {
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
}

pub struct BattleshipGame {
    players: [InternalPlayer; 2]
}

impl BattleshipGame {
    pub fn new(p1: Box<dyn Player>, p2: Box<dyn Player>) -> BattleshipGame {
        Self {
            players: [InternalPlayer::new(p1), InternalPlayer::new(p2)]
        }
    }
    pub fn run_game(&mut self) {
        for player in self.players.iter_mut() {
            player.reset();
        }
        let mut current_player_id: usize = 0;
        let mut _turns_taken: usize = 0;

        // continue running until a player has lost
        while self.players[0].hits_left > 0 && self.players[1].hits_left > 0 {
            // increase the counter of turns taken each time player 1 takes their turn
            if current_player_id == 0 {
                _turns_taken += 1;
            }
            // ask the current player to take a shot with the information in their aiming board
            let shot_taken: usize = self.players[current_player_id].player.take_shot(&self.players[current_player_id].aiming_board);
            if self.players[(current_player_id + 1).rem_euclid(2)].target_board.check_hit(shot_taken) {
                self.players[current_player_id].aiming_board.hits[shot_taken] = 1;
                // take one hit away from the player who was hit
                self.players[(current_player_id + 1).rem_euclid(2)].hits_left -= 1;
            } else {
                self.players[current_player_id].aiming_board.hits[shot_taken] = 0;
            }
            self.players[current_player_id].aiming_board.targetable[shot_taken] = 0;
            // switch to the other player
            current_player_id = (current_player_id + 1).rem_euclid(2);
        }
        // inform the players of their victory/loss when the game ends
        if self.players[0].hits_left == 0 {
            self.players[0].player.game_finish(false);
            self.players[1].player.game_finish(true);
        } else {
            self.players[0].player.game_finish(true);
            self.players[1].player.game_finish(false);
        }
    }
    pub fn run_multiple(&mut self, count: usize) {
        for _ in 0..count {
            Self::run_game(self)
        }
    }
}
