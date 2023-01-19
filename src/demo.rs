use std::cell::RefCell;
use std::rc::Rc;

use tetra::graphics::{DrawParams, Rectangle, Texture};
use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::Context;

use crate::common::{center_play_area, center_screen};
use crate::consts::{BRICK_SIZE, DEMO_LEVEL, INFO_WIDTH, NUM_STATES, PLATE_REPLAY_COMPLETED, WIDTH};
use crate::field::{GameField, GameState};
use crate::loader::Loader;
use crate::replay::{Action, ReplayEngine};
use crate::scenes::{Scene, Transition};
use crate::scores::Scores;

pub struct DemoScene {
    field: GameField,
    state_tx: Texture,
    progress_tx: Texture,
    info_tx: Texture,
    replay: ReplayEngine,
    tick: u64,         // internal ticker counter for displaying replays correctly
    rules_shown: bool, // true if replay must pause before start and show the game rules
}

impl DemoScene {
    pub fn new(ctx: &mut Context, ld: Rc<Loader>, sc: Rc<RefCell<Scores>>, lvl: usize) -> tetra::Result<Self> {
        let lvl = if lvl == 0 { DEMO_LEVEL } else { lvl };
        let state_image = include_bytes!("../assets/all_plates.png");
        let progress_image = include_bytes!("../assets/progress.png");
        let info_image = include_bytes!("../assets/rules.png");
        let mut p = DemoScene {
            field: GameField::new(ctx, ld, sc, true)?,
            state_tx: Texture::from_encoded(ctx, state_image)?,
            progress_tx: Texture::from_encoded(ctx, progress_image)?,
            info_tx: Texture::from_encoded(ctx, info_image)?,
            replay: ReplayEngine::new(),
            tick: 0,
            rules_shown: lvl == DEMO_LEVEL,
        };
        p.field.load(lvl);
        p.replay.load(lvl);
        p.replay.replay_start();
        println!("Replay for level {} loaded. {} moves.", lvl, p.replay.action_count());
        Ok(p)
    }

    // the only decoration is a plate that shows that the replay has finished
    fn draw_deco(&mut self, ctx: &mut Context) {
        let w = self.state_tx.width() as f32;
        let h = (self.state_tx.height() / NUM_STATES) as f32;

        let clip_rect = match self.field.state {
            GameState::Unfinished => return,
            _ => Rectangle::new(0.0, h * PLATE_REPLAY_COMPLETED, w, h),
        };

        let dp = DrawParams::new().position(center_screen(w, h));
        self.state_tx.draw_region(ctx, clip_rect, dp);
    }

    // show progress bar for replay
    fn draw_progress(&mut self, ctx: &mut Context) {
        let progress = if self.replay.replay_percent() > 100 { 100 } else { self.replay.replay_percent() };
        if progress == 0 {
            return;
        }

        let x = ((WIDTH - INFO_WIDTH) as f32 + 0.5) * BRICK_SIZE;
        let y = BRICK_SIZE * 1.0;
        let w = self.progress_tx.width() * progress / 100;
        let h = self.progress_tx.height() as f32;
        let clip_rect = Rectangle::new(0.0, 0.0, w as f32, h);
        let dp = DrawParams::new().position(Vec2::new(x, y));
        self.progress_tx.draw_region(ctx, clip_rect, dp);
    }
}

impl Scene for DemoScene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result<Transition> {
        // take a break while game rules are displayed
        if self.rules_shown {
            if input::is_key_pressed(ctx, Key::Space) || input::is_key_pressed(ctx, Key::Escape) {
                self.rules_shown = false;
            }
            return Ok(Transition::None);
        }

        self.tick += 1;
        while let Some(act) = self.replay.next_replay_action(self.tick) {
            match act {
                Action::Up => {
                    println!("{} - UP", self.tick);
                    self.field.player_up();
                }
                Action::Down => {
                    println!("{} - DOWN", self.tick);
                    self.field.player_down();
                }
                Action::Throw => {
                    println!("{} - THROW", self.tick);
                    self.field.throw_brick();
                }
            }
        }

        // if replay ends, consider this as the level is solved
        if !self.replay.is_playing() {
            self.field.state = GameState::Winner;
        }

        if input::is_key_pressed(ctx, Key::Escape) {
            return Ok(Transition::Pop);
        }

        if input::is_key_pressed(ctx, Key::Space)
            || input::is_key_pressed(ctx, Key::Enter)
            || input::is_key_pressed(ctx, Key::NumPadEnter) && self.field.state == GameState::Winner
        {
            return Ok(Transition::Pop);
        }

        // field.update is always considered to return None because DEMO mode
        // contols itself
        let _ = self.field.update(ctx);
        Ok(Transition::None)
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result<Transition> {
        let _ = self.field.draw(ctx)?;
        self.draw_deco(ctx);
        self.draw_progress(ctx);

        if self.rules_shown {
            let w = self.info_tx.width() as f32;
            let h = self.info_tx.width() as f32;
            let pos = center_play_area(w, h);
            self.info_tx.draw(ctx, DrawParams::new().position(pos));
        }

        Ok(Transition::None)
    }
}
