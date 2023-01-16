use crate::consts::MAX_SIZE;
use crate::field::BrickKind;

// file name with all levels
const STD_LEVELS: &str = include_str!("../assets/std_puzzles");
// if starting block is not set for a level in the file, use this one
const DEFAULT_KIND: BrickKind = BrickKind::Joker;

// convert character to type of a block
fn c2brick(c: char) -> BrickKind {
    match c {
        'S' | 's' | '$' | '1' => BrickKind::K1,
        'X' | 'x' | '%' | '2' => BrickKind::K2,
        'O' | 'o' | '@' | '3' => BrickKind::K3,
        'T' | 't' | '=' | '4' => BrickKind::K4,
        'Z' | 'z' | '+' | '5' => BrickKind::K5,
        'W' | 'w' | ':' | '6' => BrickKind::K6,
        '?' => BrickKind::Joker,
        _ => BrickKind::None,
    }
}

// a single level
#[derive(Clone)]
pub struct Level {
    pub corner: Vec<u8>,             // pattern of the top left corner
    pub puzzle: Vec<Vec<BrickKind>>, // initial block positions
    pub first: BrickKind,            // player's starting block
}

impl Default for Level {
    fn default() -> Self {
        Level { corner: Vec::new(), puzzle: Vec::new(), first: DEFAULT_KIND }
    }
}

pub struct Loader {
    levels: Vec<Level>, // all levels
}

impl Loader {
    pub fn new() -> Loader {
        let mut loader = Loader { levels: Vec::new() };
        loader.load_from_string(STD_LEVELS);
        loader
    }

    // returns a level info by its number.
    // Panics if the level number is invalid (that should never happen without
    // manual modification of hiscores file)
    pub fn level(&self, level_no: usize) -> Level {
        self.levels[level_no].clone()
    }

    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    // Validate level and fail early - in any case the game in not playable
    fn validate_level(&self, level: &Level, lvl_num: usize) {
        let max_size: u8 = MAX_SIZE as u8;
        // 1. Corner pattern must be:
        //   - Either missing
        //   - Or contain less than MAX_SIZE-1 lines
        // 2. No corner line length can exceed MAX_SIZE
        if level.corner.len() > MAX_SIZE - 1 || level.corner.len() == 1 {
            panic!(
                "Level {}: corner pattern must be omitted or has between 2 and {} lines, found {} lines",
                lvl_num,
                max_size - 1,
                level.corner.len()
            );
        }
        for l in level.corner.iter() {
            if *l > max_size {
                panic!("Level {}: corner line exceeds {} blocks = {} blocks", lvl_num, max_size, *l);
            }
        }

        // A puzzle must have:
        // 1. Width and height less than or equal to MAX_SIZE
        // 2. Both width and height at least 2 blocks
        // 3. No holes in any column
        if level.puzzle.len() > MAX_SIZE || level.puzzle.len() < 2 {
            panic!(
                "Level {}: puzzle must has between 2 and {} lines, found {} lines",
                lvl_num,
                max_size,
                level.puzzle.len()
            );
        }
        let max_w: usize = level.puzzle.iter().fold(0, |mx, x| if mx < x.len() { x.len() } else { mx });
        if !(2..=MAX_SIZE).contains(&max_w) {
            panic!("Level {}: puzzle must has between 2 and {} columns, found {} columns", lvl_num, max_size, max_w);
        }
        for i in 0..max_w {
            let mut found: bool = false;
            for l in level.puzzle.iter() {
                if l.len() < i {
                    continue;
                }
                if l[i] == BrickKind::None && found {
                    panic!("Level {} contains a hole in a puzzle", lvl_num);
                }
                if l[i] != BrickKind::None {
                    found = true;
                }
            }
        }
    }

    // Load all levels from a string
    // Level file restrictions:
    //  - No leading whitespaces
    //  - No whitespaces between 'start:' and the following block type
    //  - Mandatory empty line after a corner pattern if it is set for a level
    //  - You should not change the very first level in `std_puzzles` file - it is DEMO level:
    //      a) inaccessible for a player to play
    //      b) its replay is built-in in the binary
    //    So, if you ever change the first `DEMO` level you have to do the following as well:
    //      a) solve this and record your solution
    //      b) replace existing `assets/level-0000.rpl` with your new replay
    //    Otherwise `DEMO` in main menu would be broken
    // Format:
    // `;` - comment line. All lines starting with `;` are ignoerd
    // `#` at the first position means that from the next line the next level starts
    //      you can write any text after `#` (e.g, I write level numbers for easier debugging)
    // `start:BLOCK_TYPE`
    //    Optional line.
    //    It should be the first line of level description. The line defines the player's
    //    block at game start. If the line is missing, the player's first block is `?`
    // `*****`
    //    If line starts from `*` it means that it is corner pattern. After `*` any characters
    //    can follow because loader only reads the line of the length and does not parse it.
    //    It results in that you cannot create a corner with holes - it is always filled with
    //    icy blocks
    // The last part of the level is its puzzle: lines of blocks. How to encode a block with
    //    characters you can see in function `c2brick`
    // Full level example:
    // # level 01
    // start:?
    // ******
    // ****
    // ****
    // ***
    // **
    // *
    //
    // $%=
    // %%%
    // %=$
    fn load_from_string(&mut self, pset: &str) {
        let mut in_corner: bool = false;
        let mut in_puzzle: bool = false;
        let mut lvl: Level = Default::default();
        self.levels.clear();

        for s in pset.lines() {
            let s = s.trim_end();
            // empty line found. If previous section was corner pattern, switch to puzzle mode
            if s.is_empty() {
                if in_corner {
                    in_corner = false;
                    in_puzzle = true;
                }
                continue;
            }
            // skip comment lines
            if s.starts_with(';') {
                continue;
            }
            // sets the first block used by a player
            if s.starts_with("start:") {
                let s1 = s.trim_start_matches("start:");
                let s1 = s1.trim_start();
                if s1.is_empty() {
                    continue;
                }
                lvl.first = c2brick(s1.chars().next().unwrap());
                continue;
            }
            // new level starts. Save previous level and continue
            if s.starts_with('#') {
                if !lvl.puzzle.is_empty() {
                    self.validate_level(&lvl, self.levels.len());
                    self.levels.push(lvl);
                    lvl = Default::default();
                }
                continue;
            }
            // corner pattern starts
            if s.starts_with('*') {
                if !in_corner {
                    in_corner = true;
                }
                lvl.corner.push(s.len() as u8);
                continue;
            }
            if !in_puzzle {
                in_puzzle = true;
            }
            let puzzle_line: Vec<BrickKind> = s.chars().map(c2brick).collect();
            lvl.puzzle.push(puzzle_line);
        }
        // save the last level - there is no `#` after it
        if !lvl.puzzle.is_empty() {
            self.validate_level(&lvl, self.levels.len());
            self.levels.push(lvl);
        }
        println!("Loaded {} levels", self.levels.len());
    }
}
