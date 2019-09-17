use serde_derive::{Deserialize, Serialize};
use tetra::input::Key;

use std::fmt;
use std::fs::{read, File};
use std::io::Write;
use std::path::PathBuf;

use crate::common::replay_path;
use crate::consts::DEMO_LEVEL;

const REPLAY_VERSION: u32 = 1;
// the first replay action must be no later than MAX_DELAY ticks
const MAX_DELAY: u64 = 60 * 3;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Action {
    Up,
    Down,
    Throw,
}

#[derive(PartialEq)]
enum State {
    Idle,
    Recording,
    Replaying,
}

fn key_to_action(k: Key) -> Action {
    match k {
        Key::Up => Action::Up,
        Key::Down => Action::Down,
        Key::Space => Action::Throw,
        _ => unreachable!(),
    }
}

#[derive(Serialize, Deserialize)]
pub struct Move {
    tick: u64,
    act: Action,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} => ", self.tick)?;
        match self.act {
            Action::Up => write!(f, "Player UP"),
            Action::Down => write!(f, "Player DOWN"),
            Action::Throw => write!(f, "THROW"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Replay {
    version: u32,
    moves: Vec<Move>,
}

impl Default for Replay {
    fn default() -> Self {
        Replay { version: REPLAY_VERSION, moves: Vec::new() }
    }
}

pub struct ReplayEngine {
    replay: Replay,
    state: State,
    idx: usize,
    shift: u64,
}

impl ReplayEngine {
    pub fn new() -> Self {
        ReplayEngine { replay: Replay::default(), state: State::Idle, shift: 0, idx: 0 }
    }

    fn replay_filename(lvl: usize) -> PathBuf {
        PathBuf::from(&format!("level-{:04}.rpl", lvl))
    }

    pub fn rec_start(&mut self) {
        self.state = State::Recording;
        self.replay.moves.clear();
        // reassign because previous load call can load old version of replay
        self.replay.version = REPLAY_VERSION;
    }

    pub fn load(&mut self, lvl: usize) {
        let bytes: Vec<u8>;
        if lvl == DEMO_LEVEL {
            bytes = include_bytes!("../assets/level-0000.rpl").to_vec();
        } else {
            let mut rpath = replay_path();
            rpath.push(Self::replay_filename(lvl));
            if !rpath.is_file() {
                return;
            }
            if let Ok(v) = read(rpath) {
                bytes = v;
            } else {
                return;
            }
        }

        let replay: Replay = bincode::deserialize(&bytes).unwrap();
        if replay.version == REPLAY_VERSION {
            self.replay = replay;
            self.idx = 0;
            if !self.replay.moves.is_empty() {
                self.shift =
                    if self.replay.moves[0].tick > MAX_DELAY { self.replay.moves[0].tick - MAX_DELAY } else { 0 }
            }
        } else {
            eprintln!("Unsupported version: {}, can replay only version {}", replay.version, REPLAY_VERSION);
        }
    }

    pub fn save(&mut self, lvl: usize) {
        if self.replay.moves.is_empty() {
            return;
        }

        // make breaks between actions no longer than MAX_DELAY
        let mut shift = 0u64;
        let mut last_delay = 0u64;
        for v in self.replay.moves.iter_mut() {
            let delay = v.tick - shift - last_delay;
            if delay > MAX_DELAY {
                shift += delay - MAX_DELAY;
            }
            if shift != 0 {
                v.tick -= shift;
            }
            last_delay = v.tick;
        }

        let encoded: Vec<u8> = bincode::serialize(&self.replay).unwrap();
        let mut rpath = replay_path();
        rpath.push(Self::replay_filename(lvl));
        if let Ok(mut f) = File::create(rpath) {
            let _ = f.write_all(&encoded);
        }
    }

    pub fn is_loaded(&self) -> bool {
        !self.replay.moves.is_empty()
    }

    pub fn replay_start(&mut self) {
        assert!(self.state == State::Idle);
        if self.replay.moves.is_empty() {
            return;
        }
        self.state = State::Replaying;
        self.idx = 0;
    }

    pub fn is_playing(&self) -> bool {
        self.state == State::Replaying && self.idx < self.replay.moves.len()
    }

    pub fn replay_percent(&self) -> i32 {
        let l = self.replay.moves.len() as i32;
        if l == 0 || self.state != State::Replaying {
            0
        } else {
            self.idx as i32 * 100 / l
        }
    }

    pub fn next_replay_action(&mut self, tick: u64) -> Option<Action> {
        if !self.is_playing() || self.idx >= self.replay.moves.len() {
            return None;
        }

        if self.idx >= self.replay.moves.len() {
            return None;
        }

        let next_ticks = self.replay.moves[self.idx].tick - self.shift;
        if tick < next_ticks {
            return None;
        }

        self.idx += 1;
        Some(self.replay.moves[self.idx - 1].act)
    }

    pub fn add_action(&mut self, ticks: u64, key: Key) {
        assert!(self.state == State::Recording);
        let m = Move { tick: ticks, act: key_to_action(key) };
        self.replay.moves.push(m);
    }

    pub fn action_count(&mut self) -> usize {
        self.replay.moves.len()
    }
}
