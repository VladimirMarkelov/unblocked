use std::env::current_exe;
use std::fs;
use std::path::{Path, PathBuf};

use tetra::graphics::Vec2;

use crate::consts::{BRICK_SIZE, INFO_WIDTH, SCR_H, SCR_W};

const CONF_FILE: &str = "config.toml";
const SCORE_FILE: &str = "hiscores.toml";
const DEV_NAME: &str = "rionnag";
const GAME_NAME: &str = "unblocked";
const REPLAY_DIR: &str = "replays";

// Returns the number of digits in a number.
// Used for small numbers like level number or the number of throws
pub fn digits(n: usize) -> u8 {
    if n > 999_999 {
        panic!("Number too big")
    } else if n > 99_999 {
        6
    } else if n > 9_999 {
        5
    } else if n > 999 {
        4
    } else if n > 99 {
        3
    } else if n > 9 {
        2
    } else {
        1
    }
}

// Returns the directory where the game binary is
fn exe_path() -> PathBuf {
    match current_exe() {
        Ok(mut p) => {
            p.pop();
            p
        }
        Err(_) => unreachable!(),
    }
}

// Returns current user's directory for configuration files
//    For Windows it is %USER%/Appdata/Roaming
//    For Linux it is ~/.config
fn user_config_path() -> PathBuf {
    match dirs::config_dir() {
        Some(p) => p,
        None => unreachable!(),
    }
}

// Returns the directory where the application save to/loads from all its configs/hiscores etc
// In normal mode:
//    For Windows it is %USER%/Appdata/Roaming/DEV_NAME/GAME_NAME/
//    For Linux it is ~/.config/DEV_NAME/GAME_NAME/
// In portable mode:
//    For all OSes it is directory where the application binary is
fn base_path() -> PathBuf {
    if is_portable() {
        exe_path()
    } else {
        // exe_path() // TODO:
        let mut path = user_config_path();
        path.push(DEV_NAME);
        path.push(GAME_NAME);
        ensure_path_exists(&path);
        path
    }
}

// Returns if the application works in portable mode.
// If there is CONF_FILE file in the directory where the application binary, it means the
// portable mode is on
fn is_portable() -> bool {
    let mut p = exe_path();
    p.push(CONF_FILE);
    p.exists()
}

// there is no config yet, so comment the function out for now
// pub fn config_path() -> PathBuf {
//     let mut p = base_path();
//     p.push(CONF_FILE);
//     p
// }

// Returns path to the file with hiscores
pub fn score_path() -> PathBuf {
    let mut p = base_path();
    p.push(SCORE_FILE);
    p
}

// Returns path to the directory where replays are
pub fn replay_path() -> PathBuf {
    let mut path = base_path();
    path.push(REPLAY_DIR);
    ensure_path_exists(&path);
    path
}

// Creates all path's intermediate directories to make sure that the `p` exists.
// Returns false if it failed to create required directories (may happen, e.g, on read-only media
pub fn ensure_path_exists(p: &Path) -> bool {
    if p.exists() {
        return true;
    }
    fs::create_dir_all(p).is_ok()
}

// Returns position for an object to put it in the center of the screen
pub fn center_screen(width: f32, height: f32) -> Vec2 {
    let x = (SCR_W - width) / 2.0;
    let y = (SCR_H - height) / 2.0;
    Vec2::new(x, y)
}

// Returns position for an object to put it in the center of the play area
pub fn center_play_area(width: f32, height: f32) -> Vec2 {
    let info_width = INFO_WIDTH as f32 * BRICK_SIZE;
    let x = (SCR_W - info_width - width) / 2.0;
    let y = (SCR_H - height) / 2.0;
    Vec2::new(x, y)
}

// return max value if the value exceeds max
pub fn clamp(value: u32, max: u32) -> u32 {
    if value > max {
        max
    } else {
        value
    }
}
