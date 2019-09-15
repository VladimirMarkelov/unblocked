use chrono::{Datelike, NaiveDate};
use std::cell::RefCell;
use std::f32::consts::PI;
use std::fmt;
use std::rc::Rc;

use tetra::graphics::{self, Animation, Color, DrawParams, Rectangle, Texture, Vec2};
use tetra::input::{self, Key};
use tetra::Context;

use crate::common::{clamp, digits};
use crate::consts::{BRICK_SIZE, HEIGHT, INFO_WIDTH, MAX_SIZE, SCR_H, SCR_W, WIDTH};
use crate::loader::Loader;
use crate::scenes::Transition;
use crate::scores::{Score, Scores};
use crate::textnum::{TextNumber, TextParams};

const TICKS: u32 = 1;
const BRICK_DEF_SPEED: f32 = 48.0;
const BRICK_FALL_SPEED: f32 = 16.0;
const ARROW_FRAMES: usize = 4;

// developer best results - I know some of them can be improved
static RECORDS: &'static [u32] = &[
    4, // demo level
    4, 5, 4, 7, 4, 5, 6, 6, 9, 10, // 1-10
    11, 6, 6, 8, 8, 8, 8, 6, 7, 7, // 11-20
    8, 9, 8, 7, 9, 8, 7, 10, 12, 12, // 21-30
    10, 12, 11, 11, 10, 10, 11, 11, 10, 12, // 31-40
    12, 12, 14, 11, 10, 11, 15, 15, 17, 12, // 41-50
    15, 12, 12, 14, 16, 12, // 51-56
];
const RECORD_LEN: usize = 57;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GameState {
    Unfinished, // keep playing
    Winner,     // level cleared
    Looser,     // no moves available
    Completed,  // last level cleared
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameState::Unfinished => write!(f, "playing..."),
            GameState::Winner => write!(f, "level solved"),
            GameState::Looser => write!(f, "level failed"),
            GameState::Completed => write!(f, "game completed"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BrickKind {
    None,
    K1,
    K2,
    K3,
    K4,
    K5,
    K6,
    Joker,
}

impl fmt::Display for BrickKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BrickKind::K1 => write!(f, "'S'"),
            BrickKind::K2 => write!(f, "'X'"),
            BrickKind::K3 => write!(f, "'O'"),
            BrickKind::K4 => write!(f, "'T'"),
            BrickKind::K5 => write!(f, "'Z'"),
            BrickKind::K6 => write!(f, "'W'"),
            BrickKind::Joker => write!(f, "'?'"),
            _ => write!(f, "???"),
        }
    }
}

fn brick2shift(k: BrickKind) -> f32 {
    match k {
        BrickKind::K1 => BRICK_SIZE,
        BrickKind::K2 => BRICK_SIZE * 2.0,
        BrickKind::K3 => BRICK_SIZE * 3.0,
        BrickKind::K4 => BRICK_SIZE * 4.0,
        BrickKind::K5 => BRICK_SIZE * 5.0,
        BrickKind::K6 => BRICK_SIZE * 6.0,
        BrickKind::Joker => BRICK_SIZE * 7.0,
        BrickKind::None => 0.0,
    }
}

#[derive(Debug, Clone)]
struct Brick {
    // position in whole blocks
    x: usize,
    y: usize,
    scr_pos: Vec2,   // exact position in points
    kind: BrickKind, // kind of block
    vel: Vec2,       // velocity
    ticks: u32,      // ticks for moving (shift a block by velocity every N ticks)
    limit: Vec2,     // stop moving the block when it reaches the limit
}

impl Brick {
    fn new(x: usize, y: usize, kind: BrickKind) -> Self {
        Brick {
            x,
            y,
            kind,
            scr_pos: Vec2::new(BRICK_SIZE * x as f32, BRICK_SIZE * y as f32),
            vel: Vec2::new(0.0, 0.0),
            ticks: 0,
            limit: Vec2::new(0.0, 0.0),
        }
    }
    fn start_moving(&mut self, vel: Vec2, limit: Vec2) {
        self.vel = vel;
        self.limit = limit;
        self.ticks = TICKS;
    }
    // A block must start falling when:
    //   - thrown block hits the right wall
    //   - thrown block annihilates a block and a block at the top of it must fall now
    // A velocity for both cases differs
    fn fall(&mut self, speed: f32) {
        if !self.is_moving() {
            self.vel = Vec2::new(0.0, speed);
            self.limit = Vec2::new(self.scr_pos.x, self.scr_pos.y + BRICK_SIZE);
            self.ticks = TICKS;
        } else {
            self.limit.y += BRICK_SIZE;
        }
    }
    fn is_moving(&self) -> bool {
        self.vel.x.abs() > 0.1 || self.vel.y.abs() > 0.1
    }
    fn is_moving_down(&self) -> bool {
        self.vel.y.abs() > 0.1
    }
    // stop moving
    fn stop(&mut self) {
        self.vel = Vec2::new(0.0, 0.0);
        self.x = (self.scr_pos.x / BRICK_SIZE) as usize;
        self.y = (self.scr_pos.y / BRICK_SIZE) as usize;
    }
    fn update(&mut self) {
        if !self.is_moving() {
            return;
        }

        self.ticks -= 1;
        if self.ticks != 0 {
            return;
        }

        // time to move the block
        self.ticks = TICKS;
        self.scr_pos.x += self.vel.x;
        self.scr_pos.y += self.vel.y;

        if (self.scr_pos.x > self.limit.x && self.vel.x > 0.0) || (self.scr_pos.x < self.limit.x && self.vel.x < 0.0) {
            self.scr_pos.x = self.limit.x;
        }
        if (self.scr_pos.y > self.limit.y && self.vel.y > 0.0) || (self.scr_pos.y < self.limit.y && self.vel.y < 0.0) {
            self.scr_pos.y = self.limit.y;
        }

        // convert current exact screen position into whole blocks when the block reaches the limit
        if (self.scr_pos.x - self.limit.x).abs() < 0.1 && (self.scr_pos.y - self.limit.y).abs() < 0.1 {
            let xx = (self.scr_pos.x / BRICK_SIZE).round() as usize;
            let yy = (self.scr_pos.y / BRICK_SIZE).round() as usize;
            self.x = xx;
            self.y = yy;
            self.vel = Vec2::new(0.0, 0.0);
        }
    }
}

// convert coordinate in whole blocks into screen coordinates
fn b2s<T: Into<usize>>(x: T, y: T) -> Vec2 {
    Vec2::new(x.into() as f32 * BRICK_SIZE, y.into() as f32 * BRICK_SIZE)
}
// puzzle is a one-dimensional array, the function converts X,Y coordinate into
// position inside the puzzle array
fn pos2puz<T: Into<usize>>(x: T, y: T) -> usize {
    x.into() + y.into() * WIDTH
}

pub struct GameField {
    puzzle: [u32; HEIGHT * WIDTH],
    bricks: Vec<Brick>,
    pub level: usize, // current level No inside puzzle_set
    pub state: GameState,

    player: Brick,
    player_row: usize,
    going_back: bool,  // the player's block is flying back after throw
    pub score: u32,    // the number of throws so far
    lvl_score: Score,  // info about level hiscores
    pub demoing: bool, // is in demo mode(for demo mode some things are not displayed)

    // calculated and orientation of an arrow that shows the first block that
    // player's block would hit after throwing
    arrow_pos: Vec2,
    arrow_down: bool,
    arrow_animation: Animation,

    // kind of a block that player's block would hit after throwing
    first_brick: BrickKind,

    brick_tx: Texture,
    back_tx: Texture,

    level_no_tx: Texture,
    throws_tx: Texture,
    attempts_tx: Texture,
    solved_tx: Texture,

    txt_num: TextNumber,
    loader: Rc<Loader>,
    pub scores: Rc<RefCell<Scores>>,
}

impl GameField {
    pub fn new(ctx: &mut Context, loader: Rc<Loader>, scores: Rc<RefCell<Scores>>, demo: bool) -> tetra::Result<Self> {
        let lvl_curr = scores.borrow().curr_level();
        let lvl_info = scores.borrow().level_info(lvl_curr);
        let arrow_image = include_bytes!("../assets/arrows.png");
        let brick_image = include_bytes!("../assets/bricks.png");
        let background_image = include_bytes!("../assets/background.png");
        let number_image = include_bytes!("../assets/numbers.png");
        let level_no_image = include_bytes!("../assets/level_no.png");
        let throws_image = include_bytes!("../assets/throws.png");
        let attempts_image = include_bytes!("../assets/attempts.png");
        let solved_image = include_bytes!("../assets/solved.png");
        Ok(GameField {
            bricks: Vec::new(),
            puzzle: [0; HEIGHT * WIDTH],
            level: lvl_curr,
            state: GameState::Unfinished,
            player: Brick::new(WIDTH - INFO_WIDTH - 1, HEIGHT - 2, BrickKind::Joker),
            player_row: HEIGHT - 2,
            going_back: false,
            lvl_score: lvl_info,
            score: 0,
            demoing: demo,

            arrow_down: false,
            arrow_pos: Vec2::new(0.0, 0.0),
            first_brick: BrickKind::None,

            txt_num: TextNumber::new(ctx, number_image)?,
            loader,
            scores,

            brick_tx: Texture::from_file_data(ctx, brick_image)?,
            back_tx: Texture::from_file_data(ctx, background_image)?,
            level_no_tx: Texture::from_file_data(ctx, level_no_image)?,
            throws_tx: Texture::from_file_data(ctx, throws_image)?,
            attempts_tx: Texture::from_file_data(ctx, attempts_image)?,
            solved_tx: Texture::from_file_data(ctx, solved_image)?,

            arrow_animation: Animation::new(
                Texture::from_file_data(ctx, arrow_image)?,
                Rectangle::row(0.0, 0.0, BRICK_SIZE, BRICK_SIZE).take(ARROW_FRAMES).collect(),
                60 / 6, // 60HZ to a frame per 250ms
            ),
        })
    }

    // are user key strokes processed?
    // All key presses are ignored if the player's block in moving or game is over
    pub fn is_interactive(&self) -> bool {
        !self.going_back && self.state == GameState::Unfinished
    }

    // start moving player's block back after hitting the floor or an non-matching block
    fn go_back(&mut self) {
        self.going_back = true;
        let xlimit = (WIDTH - INFO_WIDTH - 1) as f32 * BRICK_SIZE;
        let ylimit = self.player_row as f32 * BRICK_SIZE;
        let xn = (xlimit - self.player.scr_pos.x) / BRICK_DEF_SPEED;
        let dy = (self.player_row as f32 * BRICK_SIZE - self.player.scr_pos.y) / xn;
        self.player.start_moving(Vec2::new(BRICK_DEF_SPEED, dy), Vec2::new(xlimit, ylimit));
    }

    fn update_player(&mut self) {
        // detect that anything should be updated: player's block must be moving
        // or just has stopped
        let moved = self.player.is_moving();
        let moved_down = self.player.is_moving_down();
        self.player.update();
        let stopped = moved && !self.player.is_moving();
        if !moved || !stopped {
            return;
        }

        // player's block returned back after throw
        if self.going_back && stopped {
            self.going_back = false;
            self.recalc_arrow();
            self.state = self.calc_state();
            return;
        }

        // below this line it is the case when player's block is still moving

        if moved_down {
            // player's block is falling
            //
            // hit the floor
            if self.player.y == HEIGHT - 2 {
                self.player.stop();
                self.go_back();
                return;
            }

            // calculate the new kind of player's block
            let bricks = self.bricks.iter().filter(|b| b.x == self.player.x && b.y == self.player.y + 1);
            let mut removed: bool = false;
            let mut exists: bool = false;
            let mut new_kind = self.player.kind;
            for brick in bricks {
                exists = true;
                if brick.kind == new_kind || new_kind == BrickKind::Joker {
                    removed = true;
                }
                new_kind = brick.kind;
            }

            // annihilate matched blocks and drop block that were on top of them
            if !removed && exists {
                self.player.kind = new_kind;
                let x = self.player.x;
                let y = self.player.y;
                self.bricks.retain(|b| b.x != x || b.y != y + 1);
                self.bricks.iter_mut().for_each(|b| {
                    if b.x == x && b.y < y {
                        b.fall(BRICK_FALL_SPEED);
                    }
                });
                self.player.stop();
                self.go_back();
                return;
            }
            self.player.kind = new_kind;
            self.player.fall(BRICK_DEF_SPEED);

            let x = self.player.x;
            let y = self.player.y;
            self.bricks.retain(|b| b.x != x || b.y != y + 1);
            self.bricks.iter_mut().for_each(|b| {
                if b.x == x && b.y < y {
                    b.fall(BRICK_FALL_SPEED);
                }
            });
        } else {
            // player's block is moving horizontally
            let x = self.player.x;
            let y = self.player.y;

            let (dx, dy) = if self.puzzle[pos2puz(x - 1, y)] == 1 {
                //hit wall -> block falls down
                (0i32, 1i32)
            } else {
                (-1i32, 0i32)
            };

            let bricks =
                self.bricks.iter().filter(|b| b.x == (x as i32 + dx) as usize && b.y == (y as i32 + dy) as usize);
            let mut removed: bool = false;
            let mut exists: bool = false;
            let mut new_kind = self.player.kind;
            for brick in bricks {
                exists = true;
                if brick.kind == new_kind || new_kind == BrickKind::Joker {
                    removed = true;
                }
                new_kind = brick.kind;
            }

            if !removed && !exists && dx != 0 && self.puzzle[pos2puz(self.player.x - 1, self.player.y)] == 0 {
                self.player.vel = Vec2::new(-BRICK_DEF_SPEED, 0.0);
                self.player.limit = Vec2::new(BRICK_SIZE * (x as i32 + dx) as f32, BRICK_SIZE * y as f32);
                self.player.ticks = TICKS;
                return;
            }

            if removed {
                self.player.kind = new_kind;
                self.bricks.retain(|b| b.x != (x as i32 + dx) as usize || b.y != (y as i32 + dy) as usize);
                self.bricks.iter_mut().for_each(|b| {
                    if b.x == (x as i32 + dx) as usize && b.y < y {
                        b.fall(BRICK_FALL_SPEED);
                    }
                });
                if dx == 0 {
                    if self.player.y == HEIGHT - 2 {
                        self.player.stop();
                        self.go_back();

                        return;
                    }
                    self.player.fall(BRICK_DEF_SPEED);
                } else {
                    self.player.vel = Vec2::new(-BRICK_DEF_SPEED, 0.0);
                    self.player.limit = Vec2::new(BRICK_SIZE * (x as i32 + dx) as f32, BRICK_SIZE * y as f32);
                    self.player.ticks = TICKS;
                }
                return;
            }
            if exists {
                self.player.stop();
                self.player.kind = new_kind;
                self.bricks.retain(|b| b.x != (x as i32 + dx) as usize || b.y != (y as i32 + dy) as usize);
                self.bricks.iter_mut().for_each(|b| {
                    if b.x == (x as i32 + dx) as usize && b.y < y {
                        b.fall(BRICK_FALL_SPEED);
                    }
                });
                self.go_back();

                return;
            }
            // hit the floor
            if self.player.y == HEIGHT - 2 {
                self.player.stop();
                self.go_back();

                return;
            }
            self.player.fall(BRICK_DEF_SPEED);
        }
    }

    pub fn update(&mut self, ctx: &mut Context) -> tetra::Result<Transition> {
        self.arrow_animation.tick();
        for b in self.bricks.iter_mut() {
            b.update();
        }

        self.update_player();

        if self.going_back {
            return Ok(Transition::None);
        }

        if self.state == GameState::Unfinished {
            return Ok(Transition::None);
        }

        // reach here only if the level solved or failed or demo replay finished.
        // Update hiscores if it is not in DEMO mode
        if input::is_key_pressed(ctx, Key::Space) || input::is_key_pressed(ctx, Key::Return) {
            match self.state {
                GameState::Completed => {
                    if !self.demoing {
                        let mut sc = self.scores.borrow_mut();
                        sc.set_win(self.level, self.score);
                    }
                    return Ok(Transition::Pop);
                }
                GameState::Looser => {
                    {
                        let mut sc = self.scores.borrow_mut();
                        sc.set_fail(self.level);
                    }
                    self.load(self.level);
                    self.score = 0;
                }
                GameState::Winner => {
                    if !self.demoing {
                        {
                            let mut sc = self.scores.borrow_mut();
                            sc.set_win(self.level, self.score);
                        }
                        self.level += 1;
                        self.score = 0;
                        self.load(self.level);
                    }
                }
                _ => {
                    dbg!(self.state);
                }
            }
        }
        Ok(Transition::None)
    }

    fn draw_background(&mut self, ctx: &mut Context) {
        let info_w = INFO_WIDTH as i32 * BRICK_SIZE as i32;
        let bw = self.back_tx.width() as i32;
        let bh = self.back_tx.height() as i32;
        let wn = (SCR_W as i32 - info_w + bw - 1) / bw;
        let hn = (SCR_H as i32 + bh - 1) / bh;
        for y in 0..hn {
            for x in 0..wn {
                let pos = Vec2::new((x * bw) as f32, (y * bh) as f32);
                graphics::draw(ctx, &self.back_tx, DrawParams::new().position(pos));
            }
        }
    }

    fn draw_static(&mut self, ctx: &mut Context) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let t = self.puzzle[pos2puz(x, y)];
                if t == 0 {
                    continue;
                }
                let clip_rect = Rectangle::new(0.0, (t - 1) as f32 * BRICK_SIZE, BRICK_SIZE, BRICK_SIZE);
                let pos = b2s(x, y);
                graphics::draw(ctx, &self.brick_tx, DrawParams::new().position(pos).clip(clip_rect));
            }
        }

        let first_num_pos = |x: f32, y: f32| -> Vec2 { Vec2::new(x + BRICK_SIZE * 0.25, y + 10.0) };
        let second_num_pos = |x: f32, y: f32| -> Vec2 { Vec2::new(x + BRICK_SIZE * 2.0, y + 10.0) };

        // score
        let x = ((WIDTH - INFO_WIDTH) as f32 + 0.5) * BRICK_SIZE;
        let y = BRICK_SIZE * 3.0;
        graphics::draw(ctx, &self.throws_tx, DrawParams::new().position(Vec2::new(x, y)));
        let tp = TextParams::new().with_width(3).with_right_align();
        let n = clamp(self.score, 999);
        self.txt_num.draw(ctx, first_num_pos(x, y), n, tp.clone());
        if self.lvl_score.hiscore != 0 {
            let dev_hiscore =
                if self.level >= RECORD_LEN || self.level == 0 { self.lvl_score.hiscore } else { RECORDS[self.level] };
            let mut tp_hscore = TextParams::new().with_width(3).with_right_align();
            if self.lvl_score.hiscore < dev_hiscore {
                tp_hscore = tp_hscore.with_color(Color::rgb(0.0, 0.8, 0.3));
            } else if self.lvl_score.hiscore > dev_hiscore {
                tp_hscore = tp_hscore.with_color(Color::rgb(0.0, 0.3, 0.8));
            }
            self.txt_num.draw(ctx, second_num_pos(x, y), self.lvl_score.hiscore as u32, tp_hscore);
        }

        // level # in game, replay progress in demo
        let y = BRICK_SIZE * 1.0;
        graphics::draw(ctx, &self.level_no_tx, DrawParams::new().position(Vec2::new(x, y)));

        if self.demoing {
            return;
        }

        let digit_size = self.txt_num.digit_size();
        // level #
        let level_digits = digits(self.loader.level_count());
        let w = (self.level_no_tx.width() / 2) as f32;
        let lw = f32::from(level_digits) * digit_size.x;
        let pos = Vec2::new(x + w - lw / 2.0, y + 10.0);
        self.txt_num.draw(
            ctx,
            pos,
            self.level as u32,
            TextParams::new().with_width(level_digits).with_leading_zeroes(),
        );

        // attempts
        let y = BRICK_SIZE * 5.0;
        graphics::draw(ctx, &self.attempts_tx, DrawParams::new().position(Vec2::new(x, y)));
        let tp = TextParams::new().with_width(3).with_right_align();
        let att = clamp(self.lvl_score.attempts, 999);
        let win = clamp(self.lvl_score.wins, 999);
        self.txt_num.draw(ctx, first_num_pos(x, y), att, tp.clone());
        self.txt_num.draw(ctx, second_num_pos(x, y), win, tp);

        // solved on
        let y = BRICK_SIZE * 7.0;
        graphics::draw(ctx, &self.solved_tx, DrawParams::new().position(Vec2::new(x, y)));
        if self.lvl_score.first_win > 0 {
            let dw = digit_size.x;
            let dt: NaiveDate = NaiveDate::from_num_days_from_ce(self.lvl_score.first_win);
            let mut tp = TextParams::new().with_width(2).with_leading_zeroes();
            // change color if help had been used before the level was solved
            if self.lvl_score.help_used {
                tp = tp.with_color(Color::rgb(0.0, 0.7, 0.7));
            }
            let year = (dt.year() as u32) % 100;
            self.txt_num.draw(ctx, first_num_pos(x, y), year, tp.clone());
            let month = dt.month() as u32;
            self.txt_num.draw(ctx, first_num_pos(x + dw * 2.5, y), month, tp.clone());
            let day = dt.day() as u32;
            self.txt_num.draw(ctx, first_num_pos(x + dw * 5.0, y), day, tp.clone());
        };
    }

    fn draw_bricks(&mut self, ctx: &mut Context) {
        for b in self.bricks.iter() {
            let clip_rect = Rectangle::new(0.0, brick2shift(b.kind), BRICK_SIZE, BRICK_SIZE);
            graphics::draw(ctx, &self.brick_tx, DrawParams::new().position(b.scr_pos).clip(clip_rect));
        }
    }

    fn draw_player(&mut self, ctx: &mut Context) {
        let clip_rect = Rectangle::new(0.0, brick2shift(self.player.kind), BRICK_SIZE, BRICK_SIZE);
        graphics::draw(ctx, &self.brick_tx, DrawParams::new().position(self.player.scr_pos).clip(clip_rect));

        if !self.player.is_moving() {
            let color = if self.first_brick != BrickKind::None
                && (self.first_brick == self.player.kind || self.player.kind == BrickKind::Joker)
            {
                Color::rgb(0.0, 0.8, 0.2)
            } else {
                Color::rgb(0.8, 0.0, 0.0)
            };
            let rotate: f32 = if self.arrow_down { 0.0 } else { PI / 2.0 };

            graphics::draw(
                ctx,
                &self.arrow_animation,
                DrawParams::new().position(self.arrow_pos).color(color).rotation(rotate),
            );
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result<Transition> {
        graphics::clear(ctx, Color::rgb(0.094, 0.11, 0.16));
        self.draw_background(ctx);
        self.draw_static(ctx);
        self.draw_bricks(ctx);
        self.draw_player(ctx);

        Ok(Transition::None)
    }

    pub fn player_down(&mut self) {
        if self.player.is_moving() {
            return;
        }
        if self.player.y < (HEIGHT as usize) - 2 {
            self.player.y += 1;
            self.player.scr_pos = b2s(self.player.x, self.player.y);
        }
        self.recalc_arrow();
    }

    pub fn player_up(&mut self) {
        if self.player.is_moving() {
            return;
        }
        if self.player.y > 1 {
            self.player.y -= 1;
            self.player.scr_pos = b2s(self.player.x, self.player.y);
        }
        self.recalc_arrow();
    }

    // should return error?
    pub fn load(&mut self, lvl_no: usize) {
        self.state = GameState::Unfinished;
        self.puzzle = [0u32; WIDTH * HEIGHT];

        // top and bottom lines
        for i in 0..WIDTH {
            self.puzzle[pos2puz(i, 0)] = 1;
            self.puzzle[pos2puz(i, HEIGHT - 1)] = 1;
        }
        // info panel
        for i in 1..HEIGHT - 1 {
            self.puzzle[pos2puz(0, i)] = 1;
            for p in 0..INFO_WIDTH {
                self.puzzle[pos2puz(WIDTH - p - 1, i)] = 1;
            }
        }

        let lvl = self.loader.level(lvl_no);
        self.player = Brick::new(WIDTH - INFO_WIDTH - 1, HEIGHT - 2, lvl.first);

        // corner
        if lvl.corner.is_empty() {
            for i in 1..=MAX_SIZE {
                for j in 1..=(MAX_SIZE - i) {
                    self.puzzle[pos2puz(j, i)] = 1;
                }
            }
        } else {
            let mut y = 1usize;
            for line_len in lvl.corner.iter() {
                for x in 1..=*line_len {
                    self.puzzle[pos2puz(x, y as u8)] = 1;
                }
                y += 1;
            }
        }

        self.bricks.clear();
        let cnt = lvl.puzzle.len();
        for (yidx, bricks) in lvl.puzzle.iter().enumerate() {
            for (xidx, brick) in bricks.iter().enumerate() {
                if *brick == BrickKind::None {
                    continue;
                }
                let y = HEIGHT - cnt + yidx - 1;
                let x = xidx + 1;
                self.bricks.push(Brick::new(x, y, *brick));
            }
        }

        self.lvl_score = self.scores.borrow().level_info(self.level);
        self.recalc_arrow();
    }

    fn can_throw(&self) -> bool {
        !self.player.is_moving()
            && self.state == GameState::Unfinished
            && self.first_brick != BrickKind::None
            && (self.player.kind == BrickKind::Joker || self.player.kind == self.first_brick)
    }

    pub fn throw_brick(&mut self) {
        if !self.can_throw() {
            return;
        }
        self.score += 1;
        self.player_row = self.player.y;
        let bricks = self.bricks.iter().filter(|b| b.y == self.player.y);
        let mut x = 0;
        for brick in bricks {
            if brick.x > x {
                x = brick.x
            }
        }
        if x == 0 {
            for i in 0..MAX_SIZE + 4 {
                if self.puzzle[self.player.y * WIDTH + i] != 0 {
                    x = i;
                }
            }
        }
        x += 1;
        self.player.start_moving(Vec2::new(-BRICK_DEF_SPEED, 0.0), b2s(x, self.player.y));
    }

    fn target(&self, row: usize) -> (bool, usize, usize, BrickKind) {
        let mut first = BrickKind::None;
        let mut bx: usize = 0;
        let mut by: usize = row;

        let down = if row < HEIGHT - 1 - MAX_SIZE { true } else { !self.bricks.iter().any(|b| b.y == row) };

        if down {
            bx = if row >= HEIGHT - 1 - MAX_SIZE {
                1
            } else {
                let mut n: usize = 0;
                for i in 0..MAX_SIZE + 4 {
                    if self.puzzle[row * WIDTH + i] == 0 {
                        n = i;
                        break;
                    }
                }
                if n == 0 {
                    panic!("Nothing found");
                };
                n
            };
            let bricks = self.bricks.iter().filter(|b| b.x == bx && b.y >= row);
            by = HEIGHT - 1;
            for brick in bricks {
                if brick.y < by {
                    by = brick.y;
                    first = brick.kind;
                }
            }
            by -= 1
        } else {
            let bricks = self.bricks.iter().filter(|b| b.y == row);
            for brick in bricks {
                if brick.x > bx {
                    bx = brick.x;
                }
            }
            bx += 1;
            for brick in self.bricks.iter().filter(|b| b.y == row && b.x == bx - 1) {
                first = brick.kind;
            }
        }

        (down, bx, by, first)
    }

    fn recalc_arrow(&mut self) {
        let (is_down, x, y, brick) = self.target(self.player.y);
        if is_down {
            self.arrow_pos = b2s(x, y);
        } else {
            self.arrow_pos = b2s(x + 1, y);
        }
        self.first_brick = brick;
        self.arrow_down = is_down;
    }

    pub fn calc_state(&self) -> GameState {
        if self.bricks.is_empty() {
            if self.level + 1 >= self.loader.level_count() {
                return GameState::Completed;
            }
            return GameState::Winner;
        }
        if self.player.kind == BrickKind::Joker {
            return GameState::Unfinished;
        }

        for y in 1..HEIGHT - 1 {
            let (_d, _x, _y, kind) = self.target(y);
            if kind == self.player.kind {
                return GameState::Unfinished;
            }
        }

        GameState::Looser
    }
}
