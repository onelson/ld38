extern crate ggez;
extern crate specs;
extern crate omn_labs;


mod components;
mod systems;

use std::time::Duration;

use ggez::conf;
use ggez::event::*;
use ggez::timer;
use ggez::{GameResult, Context};
use ggez::graphics::Rect;


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
    input_state: InputState
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
}

impl ECS {
    pub fn new() -> ECS {
        // The world is in charge of component storage, and as such contains all the game state.
        let mut world = specs::World::new();
        world.register::<components::Pitcher>();
        world.register::<components::Batter>();
        world.register::<components::Bat>();
        world.register::<components::Ball>();
        world.register::<components::OuterSpace>();
        world.register::<components::Ground>();

        let pitch_sys = systems::Pitch { factor: 1. } ;
        // entities are created by combining various components via the world
        world.create_now()
            .with(components::Pitcher { ready: false })
            .with(components::Batter { ready: false })
            // FIXME: I forget if the coord system is top to bottom or not.
            // I feel like origin is top left.
            .with(components::OuterSpace { y:  20. })
            .with(components::Ground { y:  280. })
            .build();

        // systems are registered with a planner, which manages their execution
        let mut plan = specs::Planner::new(world, 1);


        plan.add_system(pitch_sys, "pitch", 10);

//        plan.add_system(render_sys, "render_layer", 20);

        ECS { planner: plan }
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
    last_tick: TickData,
    current_tick: TickData,
    ecs: ECS,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        ctx.print_resource_stats();
        let s = MainState { ecs: ECS::new(), last_tick: TickData::new(), current_tick: TickData::new() };
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


//        println!("{:?} -> {:?}", self.last_tick, self.current_tick);
        println!("{:?}", self.current_tick);

        self.ecs.tick(self.current_tick.clone());
        self.last_tick = self.current_tick.clone();

        ggez::timer::sleep_until_next_frame(_ctx, 60);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }
}


pub fn main() {

    let mut conf = conf::Conf::new();
    conf.window_height = 300;
    conf.window_width = 300;
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
