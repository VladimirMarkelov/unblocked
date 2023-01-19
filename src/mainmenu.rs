use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use tetra::graphics::{self, animation, Color, DrawParams, Rectangle, Texture};
use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::Context;

use crate::common::digits;
use crate::consts::{DEMO_LEVEL, SCR_H, SCR_W};
use crate::demo::DemoScene;
use crate::loader::Loader;
use crate::play::PlayScene;
use crate::scenes::{Scene, Transition};
use crate::scores::Scores;
use crate::textnum::{TextNumber, TextParams};

// height of a menu item sprite
const LBL_HEIGHT: f32 = 32.0;
// sizes of main menu arrow
const POINTER_W: f32 = 36.0;
const POINTER_H: f32 = 40.0;
// number of frames in main menu arrow animation
const POINTER_FRAMES: usize = 6;
// shift to draw the main menu arrow centered for the menu item
const POINTER_SHIFT: f32 = (LBL_HEIGHT - POINTER_H) * 0.5;
// menu items to manually select a level to start from
const LVL_MENU_ITEM: usize = 1;

pub struct TitleScene {
    item_pos: [Vec2<f32>; 4],        // positions of all 4 menu items
    animation: animation::Animation, // arrow
    menu_tx: Texture,
    menu_id: usize,
    txt_num: TextNumber,

    lbl_width: [f32; 4],     // width of menu items (at this moment it is hardcoded)
    lbl_gap: [f32; 4],       // extra space between menu item and arrow
    lbl_ext_width: [f32; 4], // full menu item width (include level number)

    loader: Rc<Loader>,
    scores: Rc<RefCell<Scores>>,
}

impl TitleScene {
    pub fn new(ctx: &mut Context) -> tetra::Result<TitleScene> {
        // hardcoded menu item widths (change it if you replace main menu sprites)
        let widths: [f32; 4] = [100.0, 98.0, 80.0, 80.0];
        let mut ext_widths: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
        let mut lbl_gap: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

        let line_gap = LBL_HEIGHT * 0.5; // vertical space between items

        // calculate menu item positions so they all are shown in the middle of the screen
        // Positions can be hardcoded but I was not sure that 4 menu items is permanent count.
        let item_cnt = 4; // number of menu items
        let half_cnt = (item_cnt / 2) as f32;
        let first = if item_cnt % 2 == 0 {
            half_cnt * LBL_HEIGHT + (half_cnt - 1.0) * line_gap + line_gap * 0.5
        } else {
            half_cnt * LBL_HEIGHT + half_cnt * line_gap + LBL_HEIGHT * 0.5
        };
        let half_scr_h = SCR_H * 0.5;
        let first = half_scr_h - first; // vertical position of the first menu item
        let mut v = [Vec2::new(0.0, 0.0); 4];

        let loader = Rc::new(Loader::new());
        let scores = Rc::new(RefCell::new(Scores::new(loader.level_count())));

        // calculates extra horizontal gaps - now it makes sense only for menu item
        // that allows a user manually select level to start from.
        // Extra space depends on width of one digit
        let number_image = include_bytes!("../assets/numbers.png");
        let txt = TextNumber::new(ctx, number_image)?;
        let lvl_cnt = loader.level_count();
        let sz = txt.digit_size();
        let digs = digits(lvl_cnt);
        let lvl_width = f32::from(digs) * sz.x;
        ext_widths[LVL_MENU_ITEM] += lvl_width;
        lbl_gap[LVL_MENU_ITEM] += sz.x;

        let half_scr_w = SCR_W * 0.5;
        for i in 0..4 {
            v[i] =
                Vec2::new(half_scr_w - (widths[i] + ext_widths[i]) * 0.5, first + i as f32 * (LBL_HEIGHT + line_gap));
        }

        let arrow_image = include_bytes!("../assets/menu_arrow.png");
        let menu_image = include_bytes!("../assets/menu_items.png");

        Ok(TitleScene {
            item_pos: v,
            animation: animation::Animation::new(
                Texture::from_encoded(ctx, arrow_image)?,
                Rectangle::row(0.0, 0.0, POINTER_W, POINTER_H).take(POINTER_FRAMES).collect(),
                Duration::from_millis(100),
            ),

            menu_tx: Texture::from_encoded(ctx, menu_image)?,
            menu_id: 0,
            txt_num: txt,

            lbl_width: widths,
            lbl_gap,
            lbl_ext_width: ext_widths,

            loader,
            scores,
        })
    }
}

impl Scene for TitleScene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result<Transition> {
        self.animation.advance(ctx);
        // Key processing:
        // - Up and Down to select a menu item
        // - Space and Return to execute the selected menu item
        // - Left and Right to increase and decrease the starting level number by `1`
        //   if the menu item `LVL_MENU_ITEM` is selected
        // - Shift+Left and Shift+Right to increase and decrease the starting level number by `10`
        //   if the menu item `LVL_MENU_ITEM` is selected
        if input::is_key_pressed(ctx, Key::Up) {
            if self.menu_id == 0 {
                self.menu_id = 3;
            } else {
                self.menu_id -= 1;
            }
            Ok(Transition::None)
        } else if input::is_key_pressed(ctx, Key::Down) {
            if self.menu_id == 3 {
                self.menu_id = 0;
            } else {
                self.menu_id += 1;
            }
            Ok(Transition::None)
        } else if input::is_key_pressed(ctx, Key::Left) && self.menu_id == 1 {
            let diff = if input::is_key_down(ctx, Key::RightShift) || input::is_key_down(ctx, Key::LeftShift) {
                10usize
            } else {
                1usize
            };
            {
                let mut sc = self.scores.borrow_mut();
                sc.dec_curr_level(diff);
            }
            Ok(Transition::None)
        } else if input::is_key_pressed(ctx, Key::Right) && self.menu_id == 1 {
            let diff = if input::is_key_down(ctx, Key::RightShift) || input::is_key_down(ctx, Key::LeftShift) {
                10usize
            } else {
                1usize
            };
            {
                let mut sc = self.scores.borrow_mut();
                sc.inc_curr_level(diff);
            }
            Ok(Transition::None)
        } else if input::is_key_pressed(ctx, Key::Space)
            || input::is_key_pressed(ctx, Key::Enter)
            || input::is_key_pressed(ctx, Key::NumPadEnter)
        {
            if self.menu_id == 3 {
                Ok(Transition::Pop)
            } else if self.menu_id == 0 || self.menu_id == 1 {
                Ok(Transition::Push(Box::new(PlayScene::new(ctx, self.loader.clone(), self.scores.clone())?)))
            } else if self.menu_id == 2 {
                Ok(Transition::Push(Box::new(DemoScene::new(
                    ctx,
                    self.loader.clone(),
                    self.scores.clone(),
                    DEMO_LEVEL,
                )?)))
            } else {
                Ok(Transition::None)
            }
        } else {
            Ok(Transition::None)
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result<Transition> {
        graphics::clear(ctx, Color::rgb(0.094, 0.11, 0.16));

        let mut start: f32 = 0.0;

        // show main menu items
        for i in 0..4 {
            let clip = Rectangle::new(start, 0.0, self.lbl_width[i], LBL_HEIGHT);
            let dp = DrawParams::new().position(self.item_pos[i]);
            self.menu_tx.draw_region(ctx, clip, dp);
            start += self.lbl_width[i];
        }

        // show "arrows" to the left and to the right from the selected menu item
        let pos =
            Vec2::new(self.item_pos[self.menu_id].x - POINTER_W - 5.0, self.item_pos[self.menu_id].y + POINTER_SHIFT);
        self.animation.draw(ctx, DrawParams::new().position(pos).color(Color::rgb(0.0, 1.0, 1.0)));
        let wdth = self.lbl_width[self.menu_id] + self.lbl_ext_width[self.menu_id] + self.lbl_gap[self.menu_id];
        let pos = Vec2::new(self.item_pos[self.menu_id].x + wdth + 5.0, self.item_pos[self.menu_id].y + POINTER_SHIFT);
        self.animation.draw(ctx, DrawParams::new().position(pos).color(Color::rgb(0.0, 1.0, 1.0)));

        // show the level number to start playing from
        let digits = digits(self.scores.borrow().max_avail_level());
        let mx = self.item_pos[LVL_MENU_ITEM].x + self.lbl_gap[LVL_MENU_ITEM] + self.lbl_width[LVL_MENU_ITEM];
        self.txt_num.draw(
            ctx,
            Vec2::new(mx, self.item_pos[LVL_MENU_ITEM].y),
            self.scores.borrow().curr_level() as u32,
            TextParams::new().with_width(digits).with_leading_zeroes(),
        );

        Ok(Transition::None)
    }
}
