use serde_derive::{Deserialize, Serialize};
use std::fs::{read_to_string, write};
use std::path::PathBuf;

use chrono::prelude::*;
use chrono::NaiveDateTime;

use crate::common::score_path;

// a lever score info
#[derive(Copy, Clone, Serialize, Deserialize, Default)]
pub struct Score {
    pub attempts: u32,   // attempts to solve the puzzle
    pub wins: u32,       // puzzle solved N times
    pub hiscore: u32,    // best score
    pub first_win: i32,  // date of the first win
    pub help_used: bool, // help was used before any win
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ScoreVec {
    max_level: usize,
    levels: Vec<Score>,
}

pub struct Scores {
    scores: ScoreVec,
    curr_level: usize,  // current level a player plays (used by main menu and play scene)
    lvl_cnt: usize,     // total number of levels
    file_path: PathBuf, // file path to save/load hiscores
}

impl Scores {
    pub fn new(lvl_cnt: usize) -> Scores {
        let mut sc = Scores {
            scores: ScoreVec { levels: Vec::new(), max_level: 1 },
            curr_level: 1,
            lvl_cnt,
            file_path: score_path(),
        };
        sc.load();
        sc
    }

    pub fn load(&mut self) {
        if !self.file_path.exists() {
            // first start - no file, so initialize the scores with a default score info
            self.scores.levels.push(Score::default());
            return;
        }

        let data = match read_to_string(self.file_path.clone()) {
            Ok(s) => s,
            Err(_) => return,
        };

        let scores: ScoreVec = match toml::from_str(&data) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to parse config file: {:?}", e);
                return;
            }
        };

        self.scores = scores;
        // Set the current level to the maximum level a user has reached
        self.curr_level = self.scores.max_level;
        if self.scores.levels.is_empty() {
            self.scores.levels.push(Score::default());
        }
    }

    pub fn save(&self) {
        let tml = toml::to_string(&self.scores).unwrap();
        let name = score_path();
        let _ = write(name, tml);
    }

    pub fn level_info(&self, lvl_no: usize) -> Score {
        if self.scores.levels.len() > lvl_no {
            self.scores.levels[lvl_no]
        } else {
            Score::default()
        }
    }

    // save info about winning the level by a user. If it is the first time, save the date as well
    pub fn set_win(&mut self, lvl_no: usize, throws: u32) {
        if self.lvl_cnt <= lvl_no || self.scores.levels.len() + 1 < lvl_no {
            unreachable!()
        }

        // if the level was played for the first time, it may not have corresponding score info,
        // so fill the hiscore list with default one beforehand
        while self.scores.levels.len() <= lvl_no {
            self.scores.levels.push(Score::default());
        }

        let mut curr = self.scores.levels[lvl_no];
        curr.wins += 1;
        curr.attempts += 1;
        if curr.wins == 1 {
            // first win - remember the date
            let dt: NaiveDateTime = Local::now().naive_local();
            let days = dt.num_days_from_ce();
            curr.first_win = days;
        }
        if curr.hiscore == 0 || curr.hiscore > throws {
            curr.hiscore = if throws > 999 { 999 } else { throws };
        }
        self.scores.levels[lvl_no] = curr;

        if lvl_no < self.lvl_cnt - 1 {
            if lvl_no + 1 > self.scores.max_level {
                self.scores.max_level = lvl_no + 1;
            }
            self.curr_level = lvl_no + 1;
        }

        self.save();
    }

    // save info about the level failed
    pub fn set_fail(&mut self, lvl_no: usize) {
        if self.lvl_cnt <= lvl_no || self.scores.levels.len() + 1 < lvl_no {
            unreachable!()
        }

        while self.scores.levels.len() <= lvl_no {
            self.scores.levels.push(Score::default());
        }

        let mut curr = self.scores.levels[lvl_no];
        curr.attempts += 1;
        self.scores.levels[lvl_no] = curr;
        self.save();
    }

    // mark a level as being solved using help.
    // The detect is simple: if level has not solved and a user requests its replay, then
    // mark the level as help-used one
    pub fn set_help_used(&mut self, lvl_no: usize) {
        if self.scores.levels.len() <= lvl_no {
            unreachable!()
        }

        let mut curr = self.scores.levels[lvl_no];
        if curr.wins == 0 {
            curr.help_used = true;
            self.scores.levels[lvl_no] = curr;
            self.save();
        }
    }

    pub fn max_avail_level(&self) -> usize {
        self.scores.max_level
    }

    pub fn curr_level(&self) -> usize {
        self.curr_level
    }

    // used by main menu
    pub fn inc_curr_level(&mut self, delta: usize) -> usize {
        if self.curr_level + delta < self.scores.max_level {
            self.curr_level += delta
        } else {
            self.curr_level = self.scores.max_level;
        }
        self.curr_level
    }

    // used by main menu
    pub fn dec_curr_level(&mut self, delta: usize) -> usize {
        if self.curr_level - 1 > delta {
            self.curr_level -= delta
        } else {
            self.curr_level = 1;
        }
        self.curr_level
    }
}
