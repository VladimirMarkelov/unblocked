// Screen sizes
pub const SCR_W: f32 = 1024.0;
pub const SCR_H: f32 = 768.0;

pub const BRICK_SIZE: f32 = 48.0; // Width and height of a block
pub const WIDTH: usize = 21; // width of the window in blocks
pub const INFO_WIDTH: usize = 5; // width of the info window at the right in blocks
pub const HEIGHT: usize = 16; // height of the window in blocks
pub const MAX_SIZE: usize = 7; // puzzle max dimension: 7x7

pub const NUM_STATES: i32 = 4; // number of states

// number of the level used to show demo from main menu
// this level must be inaccessible in normal game
pub const DEMO_LEVEL: usize = 0;

// Plates ordinal number in a sprite
pub const PLATE_LEVEL_SOLVED: f32 = 0.0;
pub const PLATE_NO_MOVES: f32 = 1.0;
pub const PLATE_GAME_COMPLETED: f32 = 2.0;
pub const PLATE_REPLAY_COMPLETED: f32 = 3.0;
