use tetra::ContextBuilder;

mod common;
mod consts;
mod demo;
mod field;
mod loader;
mod mainmenu;
mod play;
mod replay;
mod scenes;
mod scores;
mod textnum;

use crate::mainmenu::TitleScene;
use crate::scenes::SceneManager;

fn main() -> tetra::Result {
    ContextBuilder::new("Unblocked", consts::SCR_W as i32, consts::SCR_H as i32)
        .resizable(true)
        .quit_on_escape(false)
        .show_mouse(true)
        .tick_rate(60.0)
        .build()?
        .run_with(|ctx| Ok(SceneManager::new(Box::new(TitleScene::new(ctx)?))))
}
