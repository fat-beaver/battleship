pub const BOARD_WIDTH: u32 = 10;
pub const BOARD_HEIGHT: u32 = 10;
pub const BOARD_SIZE: usize = (BOARD_WIDTH * BOARD_HEIGHT) as usize;

pub const SHIP_LENGTHS: [u32; 5] = [5, 4, 3, 3, 2];

pub const TOTAL_SHIP_HEALTH: u32 = 17;  // the total of the ship lengths

#[derive(Clone)]
pub struct AimingBoard {
    hits: [bool; BOARD_SIZE],
    misses: [bool; BOARD_SIZE]
}

impl AimingBoard {
    fn new() -> Self {
        Self {
            hits: [false; BOARD_SIZE],
            misses: [false; BOARD_SIZE]
        }
    }
    pub fn get_hits(&self) -> &[bool; BOARD_SIZE] {
        return &self.hits;
    }
    pub fn get_misses(&self) -> &[bool; BOARD_SIZE] {
        return &self.misses;
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

pub trait Player {
    fn new_game(&mut self);
    fn place_ships(&mut self) -> TargetBoard;
    fn take_shot(&mut self, aiming_board: &AimingBoard) -> usize;
    fn game_finish(&mut self, won: bool);
}

struct InternalPlayer {
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
}

pub fn run_game(player1: Box<dyn Player>, player2: Box<dyn Player>) {
    // create an array to hold the players so they can be alternated between easily
    let mut players: [InternalPlayer; 2] = [InternalPlayer::new(player1), InternalPlayer::new(player2)];

    let mut current_player_id: usize = 0;
    let mut _turns_taken: usize = 0;

    // continue running until a player has lost
    while players[0].hits_left > 0 && players[1].hits_left > 0 {
        // increase the counter of turns taken each time player 1 takes their turn
        if current_player_id == 0 {
            _turns_taken += 1;
        }
        // ask the current player to take a shot with the information in their aiming board
        let shot_taken: usize = players[current_player_id].player.take_shot(&players[current_player_id].aiming_board);
        if players[(current_player_id + 1).rem_euclid(2)].target_board.check_hit(shot_taken) {
            players[current_player_id].aiming_board.hits[shot_taken] = true;
            // take one hit away from the player who was hit
            players[(current_player_id + 1).rem_euclid(2)].hits_left -= 1;
        } else {
            players[current_player_id].aiming_board.hits[shot_taken] = false;
        }
        // switch to the other player
        current_player_id = (current_player_id + 1).rem_euclid(2);
    }
    // inform the players of their victory/loss when the game ends
    if players[0].hits_left == 0 {
        players[0].player.game_finish(false);
        players[1].player.game_finish(true);
        print!("player 2 won!")
    } else {
        players[0].player.game_finish(true);
        players[1].player.game_finish(false);
        print!("player 1 won!")
    }
}
