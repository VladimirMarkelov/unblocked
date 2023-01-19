use tetra::graphics::scaling::{ScalingMode, ScreenScaler};
use tetra::graphics::{self, Color};
use tetra::window;
use tetra::{Context, Event, State};

use crate::consts::{SCR_H, SCR_W};
use crate::mainmenu::TitleScene;

pub trait Scene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result<Transition>;
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result<Transition>;
}

pub enum Transition {
    None,
    Push(Box<dyn Scene>),
    Pop,
}

pub struct SceneManager {
    scaler: ScreenScaler,
    scenes: Vec<Box<dyn Scene>>,
}

impl SceneManager {
    pub fn new(ctx: &mut Context) -> tetra::Result<SceneManager> {
        let ts = TitleScene::new(ctx)?;
        Ok(SceneManager {
            // with this scaling the drawn area is scaled with the window.
            // So a user can make game window fullscreen and all sprites are scaled as well
            scaler: ScreenScaler::with_window_size(ctx, SCR_W as i32, SCR_H as i32, ScalingMode::ShowAll)?,
            scenes: vec![Box::new(ts)],
        })
    }
}

impl State for SceneManager {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        match self.scenes.last_mut() {
            Some(active_scene) => match active_scene.update(ctx)? {
                Transition::None => {}
                Transition::Push(s) => {
                    self.scenes.push(s);
                }
                Transition::Pop => {
                    self.scenes.pop();
                }
            },
            None => window::quit(ctx),
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        match self.scenes.last_mut() {
            Some(active_scene) => {
                graphics::set_canvas(ctx, self.scaler.canvas());
                match active_scene.draw(ctx)? {
                    Transition::None => {}
                    Transition::Push(s) => {
                        self.scenes.push(s);
                    }
                    Transition::Pop => {
                        self.scenes.pop();
                    }
                }
                graphics::reset_canvas(ctx);
                graphics::clear(ctx, Color::BLACK);
                self.scaler.draw(ctx);
            }
            None => window::quit(ctx),
        }

        Ok(())
    }

    fn event(&mut self, _: &mut Context, event: Event) -> tetra::Result {
        if let Event::Resized { width, height } = event {
            self.scaler.set_outer_size(width, height);
        }
        Ok(())
    }
}
