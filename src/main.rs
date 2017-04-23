extern crate ggez;
extern crate specs;
extern crate omn_labs;


mod components;
mod systems;

use std::time::Duration;
use std::sync::mpsc::{channel, Receiver, Sender};

use ggez::conf;
use ggez::event::*;
use ggez::timer;
use ggez::GameResult;
use ggez::Context;
use ggez::graphics::{self, Point};

use omn_labs::assets::AssetBundle;
use omn_labs::systems::DrawCommand;


#[derive(Clone, Debug, PartialEq)]
pub enum InputState {
    JustPressed,
    Pressed,
    JustReleased,
    Released
}

pub type Delta = f32;

#[derive(Clone, Debug, PartialEq)]
pub struct TickData {
    delta_ms: Delta,
    /// We only have one button to track (I think) so we can get away with a single member to
    /// track the state.
    input_state: InputState,
}

impl TickData {
    pub fn new() -> Self {
        Self {
            input_state: InputState::Released,
            delta_ms: 0.
        }
    }
}


pub struct ECS {
    pub planner: specs::Planner<TickData>,
    render_tx: Sender<DrawCommand>,
}

impl ECS {
    pub fn new(render_tx: Sender<DrawCommand>) -> ECS {
        // The world is in charge of component storage, and as such contains all the game state.
        let mut world = specs::World::new();
        world.register::<components::Pitcher>();
        world.register::<components::Batter>();
        world.register::<components::Bat>();
        world.register::<components::Ball>();
        world.register::<components::OuterSpace>();
        world.register::<components::Ground>();

        // entities are created by combining various components via the world
        world.create_now()
            .with(components::Pitcher { ready: true, winding: false })
            .with(components::Batter { ready: false })
            // FIXME: I forget if the coord system is top to bottom or not.
            // I feel like origin is top left.
            .with(components::OuterSpace { y:  20. })
            .with(components::Ground { y:  280. })
            .build();

        // systems are registered with a planner, which manages their execution
        let mut plan = specs::Planner::new(world, 1);


        let batter_sys = systems::BatterThink { } ;
        let pitch_sys = systems::PitcherThink { factor: 1. } ;
        plan.add_system(batter_sys, "batter", 10);
        plan.add_system(pitch_sys, "pitcher", 15);

//        plan.add_system(render_sys, "render_layer", 20);

        ECS {
            planner: plan,
            render_tx: render_tx
        }
    }

    pub fn tick(&mut self, tick_data: TickData) -> bool {

        // dispatch() tells the planner to run the registered systems in a
        // thread pool.
        self.planner.dispatch(tick_data);

        // the wait() is like a thread.join(), and will block until the systems
        // have completed their work.
        self.planner.wait();

        true
    }
}

struct MainState {
    assets: AssetBundle,
    last_tick: TickData,
    current_tick: TickData,
    ecs: ECS,
    render_rx: Receiver<DrawCommand>
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        ctx.print_resource_stats();

        // render pipe - Sender/Receiver
        let (tx, rx) = channel::<DrawCommand>();

        let s = MainState {
            assets: AssetBundle::new(ctx, &vec![
                "background.png",
                "pitcher.png"
            ]),
            ecs: ECS::new(tx),
            last_tick: TickData::new(),
            current_tick: TickData::new(),
            render_rx: rx
        };

        Ok(s)
    }

    fn update_current_tick_data(&mut self, delta_ms: Delta) {

        self.current_tick.delta_ms = delta_ms;

        // If the state was "just" anything last tick, and the state hasn't been updated by an event
        // handler during this tick, we'll "decay" the state and make it the non-"just" version.
        match self.last_tick.input_state {
            InputState::JustPressed if self.current_tick.input_state == InputState::JustPressed => {
                self.current_tick.input_state = InputState::Pressed
            }

            InputState::JustReleased if self.current_tick.input_state == InputState::JustReleased => {
                self.current_tick.input_state = InputState::Released
            },
            _ => ()
        };
    }
}


impl EventHandler for MainState {

    fn key_down_event(&mut self, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match keycode {
            // guard to prevent key repeats on long holds
            Keycode::Space if self.current_tick.input_state != InputState::Pressed => {
                self.current_tick.input_state = InputState::JustPressed;
            }
            _ => (),
        }
    }

    fn key_up_event(&mut self, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match keycode {

            Keycode::Space => {
                self.current_tick.input_state = InputState::JustReleased;
            }
            _ => (),
        }
    }

    fn update(&mut self, _ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        let delta_ms = _dt.subsec_nanos() as f32 / 1e6;
        self.update_current_tick_data(delta_ms);
        println!("{:?}", self.current_tick);
        self.ecs.tick(self.current_tick.clone());
        self.last_tick = self.current_tick.clone();

        timer::sleep_until_next_frame(_ctx, 60);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        // draw the background before all the dynamic stuff
        let bg = self.assets.get_image(ctx, "background.png");
//        let bg = self.assets.get_image(ctx, "pitcher.png");
        graphics::draw(ctx, bg, Point::zero(), 0.)?;

        for cmd in self.render_rx.try_iter() {
            match cmd {
                DrawCommand::DrawTransformed { path, x, y, rot , .. } => {
                    let image = self.assets.get_image(ctx, path.as_ref());
                    graphics::draw(ctx, image, Point::new(x, y), rot)?;
                }
                DrawCommand::Flush => {}
            }
        }

        Ok(())
    }
}


pub fn main() {

    let mut conf = conf::Conf::new();
    conf.window_height = 768;
    conf.window_width = 1024;
    conf.window_title = "Home World Derby".to_string();

    println!("Starting with default config: {:#?}", conf);

    let ctx = &mut Context::load_from_conf("HWD", "HWD", conf).unwrap();

    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
