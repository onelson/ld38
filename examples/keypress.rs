extern crate ggez;

use ggez::conf;
use ggez::event::*;
use ggez::{GameResult, Context};
use ggez::graphics;
use std::time::Duration;

struct MainState {}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        println!("{:?}", ctx.filesystem);
        let s = MainState {};
        Ok(s)
    }
}

impl EventHandler for MainState {
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        graphics::clear(ctx);
        Ok(())
    }

    fn key_down_event(&mut self, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match keycode {
            Keycode::Space => {
                println!("Down!");
            }
            _ => (),
        }
    }

    fn key_up_event(&mut self, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match keycode {
            Keycode::Space => {
                println!("Up!");
            }
            _ => (),
        }
    }
}

fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("foo", "kd38", c).unwrap();

    match MainState::new(ctx) {
        Err(e) => {
            println!("Could not load game!");
            println!("Error: {}", e);
        }
        Ok(ref mut game) => {
            let result = run(ctx, game);
            if let Err(e) = result {
                println!("Error: {}", e);
            } else {
                println!("Clean exit");
            }
        }
    }
}