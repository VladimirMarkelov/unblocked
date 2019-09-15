use std::cell::RefCell;
use std::rc::Rc;

use tetra::graphics::{self, DrawParams, Rectangle, Texture};
use tetra::input::{self, Key};
use tetra::Context;

use crate::common::center_screen;
use crate::consts::{NUM_STATES, PLATE_GAME_COMPLETED, PLATE_LEVEL_SOLVED, PLATE_NO_MOVES};
use crate::demo::DemoScene;
use crate::field::{GameField, GameState};
use crate::loader::Loader;
use crate::replay::ReplayEngine;
use crate::scenes::{Scene, Transition};
use crate::scores::Scores;

// interrupting a game after making this many throws is considered a fail
const MIN_THROWS: u32 = 3;

pub struct PlayScene {
    field: GameField,
    state_tx: Texture,
    loader: Rc<Loader>,
    scores: Rc<RefCell<Scores>>,
    replay: ReplayEngine,
    tick: u64, // internal tick counter for replays
}

impl PlayScene {
    pub fn new(ctx: &mut Context, ld: Rc<Loader>, sc: Rc<RefCell<Scores>>) -> tetra::Result<Self> {
        let s = sc.clone();
        let l = ld.clone();
        let lvl = sc.borrow().curr_level();
        let state_image = include_bytes!("../assets/all_plates.png");
        let mut p = PlayScene {
            loader: l,
            scores: s,
            field: GameField::new(ctx, ld, sc, false)?,
            state_tx: Texture::from_file_data(ctx, state_image)?,
            replay: ReplayEngine::new(),
            tick: 0,
        };
        p.field.load(lvl);
        p.replay.rec_start();
        Ok(p)
    }

    fn draw_deco(&mut self, ctx: &mut Context) {
        let w = self.state_tx.width() as f32;
        let h = (self.state_tx.height() / NUM_STATES) as f32;

        // draw a plate that describes game state (if the game is over)
        let clip_rect = match self.field.state {
            GameState::Unfinished => return,
            GameState::Winner => Rectangle::new(0.0, h * PLATE_LEVEL_SOLVED, w, h),
            GameState::Looser => Rectangle::new(0.0, h * PLATE_NO_MOVES, w, h),
            GameState::Completed => Rectangle::new(0.0, h * PLATE_GAME_COMPLETED, w, h),
        };

        let pos = center_screen(w, h);
        graphics::draw(ctx, &self.state_tx, DrawParams::new().position(pos).clip(clip_rect));
    }
}

impl Scene for PlayScene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result<Transition> {
        if input::is_key_pressed(ctx, Key::Escape) {
            // Escape is pressed after the level is solved or failed - must save info anyway
            if self.field.state == GameState::Completed || self.field.state == GameState::Winner {
                let mut sc = self.field.scores.borrow_mut();
                sc.set_win(self.field.level, self.field.score);
            } else if self.field.state == GameState::Looser
                || (self.field.score >= MIN_THROWS && self.field.state == GameState::Unfinished)
            {
                let mut sc = self.field.scores.borrow_mut();
                sc.set_fail(self.field.level);
            }
            return Ok(Transition::Pop);
        }
        self.tick += 1;
        if self.field.is_interactive() {
            if input::is_key_pressed(ctx, Key::Space) {
                self.replay.add_action(self.tick, Key::Space);
                self.field.throw_brick();
                return Ok(Transition::None);
            } else if input::is_key_pressed(ctx, Key::Up) {
                self.replay.add_action(self.tick, Key::Up);
                self.field.player_up();
            } else if input::is_key_pressed(ctx, Key::Down) {
                self.replay.add_action(self.tick, Key::Down);
                self.field.player_down();
            } else if input::is_key_pressed(ctx, Key::F1) {
                // try to load a replay for the level. If there is no replay, do nothing
                let mut replay = ReplayEngine::new();
                replay.load(self.field.level);
                if replay.is_loaded() {
                    {
                        // save info that replay was called for the level
                        let mut sc = self.field.scores.borrow_mut();
                        sc.set_help_used(self.field.level);
                    }
                    return Ok(Transition::Push(Box::new(DemoScene::new(
                        ctx,
                        self.loader.clone(),
                        self.scores.clone(),
                        self.field.level,
                    )?)));
                }
            }
        }

        assert!(!self.field.demoing);
        // save replay. It rewrites any previously saved replay for this level
        if input::is_key_pressed(ctx, Key::F5) {
            self.replay.save(self.field.level);
        }

        // if the level is failed, reset replay recorder
        let field_res = self.field.update(ctx);
        if self.field.state == GameState::Looser {
            self.replay.rec_start();
            self.tick = 0;
        }
        field_res
    }

    fn draw(&mut self, ctx: &mut Context, dt: f64) -> tetra::Result<Transition> {
        let _ = self.field.draw(ctx, dt)?;
        self.draw_deco(ctx);
        Ok(Transition::None)
    }
}
