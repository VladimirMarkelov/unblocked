use tetra::window;
use tetra::{Context, State};

pub trait Scene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result<Transition>;
    fn draw(&mut self, ctx: &mut Context, dt: f64) -> tetra::Result<Transition>;
}

pub enum Transition {
    None,
    Push(Box<dyn Scene>),
    Pop,
}

pub struct SceneManager {
    scenes: Vec<Box<dyn Scene>>,
}

impl SceneManager {
    pub fn new(initial_scene: Box<dyn Scene>) -> SceneManager {
        SceneManager { scenes: vec![initial_scene] }
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

    fn draw(&mut self, ctx: &mut Context, dt: f64) -> tetra::Result {
        match self.scenes.last_mut() {
            Some(active_scene) => match active_scene.draw(ctx, dt)? {
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
}
