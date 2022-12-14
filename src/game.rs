use nalgebra::{SVector};

pub const BOARD_WIDTH: u32 = 10;
pub const BOARD_HEIGHT: u32 = 10;
pub const BOARD_SIZE: usize = (BOARD_WIDTH * BOARD_HEIGHT) as usize;

pub const SHIP_LENGTHS: [u32; 5] = [5, 4, 3, 3, 2];

pub const TOTAL_SHIP_HEALTH: u32 = 17;  // the total of the ship lengths

#[derive(Clone)]
pub struct AimingBoard {
    hits: SVector<f64, BOARD_SIZE>,
    misses: SVector<f64, BOARD_SIZE>,
    targetable: SVector<f64, BOARD_SIZE>
}

impl AimingBoard {
    fn new() -> Self {
        Self {
            hits: SVector::repeat(0.0),
            misses: SVector::repeat(0.0),
            targetable: SVector::repeat(1.0)
        }
    }
    pub fn get_hits(&self) -> &SVector<f64, BOARD_SIZE> {
        return &self.hits;
    }
    pub fn get_misses(&self) -> &SVector<f64, BOARD_SIZE> {
        return &self.misses;
    }
    pub fn get_targetable(&self) -> &SVector<f64, BOARD_SIZE> {
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

pub struct BattleshipGame<A, B> where A: Player, B: Player {
    player_a: InternalPlayer<A>,
    player_b: InternalPlayer<B>,
    played: usize,
    turns_taken: Vec<usize>
}

impl<A, B> BattleshipGame<A, B> where A: Player, B: Player {
    pub fn new(p1:A, p2: B) -> BattleshipGame<A, B> {
        Self {
            player_a: InternalPlayer::new(p1),
            player_b: InternalPlayer::new(p2),
            played: 0,
            turns_taken: Vec::new()
        }
    }

    pub fn run_game(&mut self) {
        self.player_a.reset();
        self.player_b.reset();

        let mut current_player_id: usize = 0;
        let mut turns_taken: usize = 0;

        // continue running until a player has lost
        while self.player_a.hits_left > 0 && self.player_b.hits_left > 0 {
            // increase the counter of turns taken each time player 1 takes their turn
            if current_player_id == 0 {
                turns_taken += 1;
            }

            // ask the current player to take a shot with the information in their aiming board
            let shot_taken: usize = match current_player_id {
                0 => self.player_a.take_shot(),
                1 => self.player_b.take_shot(),
                _ => panic!(),
            };

            if match current_player_id {
                0 => self.player_b.target_board.check_hit(shot_taken),
                1 => self.player_a.target_board.check_hit(shot_taken),
                _ => panic!()
            } {
                match current_player_id {
                0 => self.player_a.aiming_board.hits[shot_taken] = 1.0,
                1 => self.player_b.aiming_board.hits[shot_taken] = 1.0,
                _ => panic!(),
                };
                match current_player_id {
                0 => self.player_b.hits_left -= 1,
                1 => self.player_a.hits_left -= 1,
                _ => panic!(),
                };
            }
            current_player_id = (current_player_id + 1) % 2;
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
        self.turns_taken.push(turns_taken);
    }

    pub fn run_multiple(&mut self, count: usize) {
        for _ in 0..count {
            Self::run_game(self)
        }
        let mut total = 0;
        let mut count = 0;
        for i in self.turns_taken.iter() {
            total += *i;
            count += 1;
        }
        println!("{} turns taken on average", total / count)
    }
}
