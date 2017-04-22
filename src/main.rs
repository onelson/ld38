extern crate ggez;
extern crate specs;
extern crate omn_labs;


mod components;
mod systems;

use std::time::Duration;

use ggez::conf;
use ggez::event;
use ggez::timer;
use ggez::{GameResult, Context};
use ggez::graphics::Rect;


#[derive(Clone, Debug)]
pub enum InputState {
    JustPressed,
    Pressed,
    JustReleased,
    Released
}

pub type Delta = f32;

#[derive(Clone, Debug)]
pub struct TickData {
    dt: Delta,
    /// We only have one button to track (I think) so we can get away with a single member to
    /// track the state.
    input_state: InputState
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
    ecs: ECS,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        ctx.print_resource_stats();
        let s = MainState { ecs: ECS::new() };
        Ok(s)
    }
}


impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context, _dt: Duration) -> GameResult<()> {
        let delta_secs = _dt.subsec_nanos() as f32 / 1e9;

        // TODO: poll input

        let tick_data = TickData {
            dt: delta_secs,
            input_state: InputState::Released
        };
        self.ecs.tick(tick_data);
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
    if let Err(e) = event::run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
