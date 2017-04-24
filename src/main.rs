extern crate ggez;
extern crate specs;
extern crate rand;
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
use ggez::graphics;

use omn_labs::assets::AssetBundle;
use omn_labs::sprites::{SpriteSheetData, PlayMode};
use systems::DrawCommand;

#[derive(Clone, Debug, PartialEq)]
pub enum GamePhase {
    WaitingForPlayer,
    PlayerReady,
    Windup, // variable duration
    Pitching, // fixed length
    BallInFlight,

    // different results of a swing
    Foul,
    HomeRun,
    Hit,
    Miss
}


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
            delta_ms: 0.,
        }
    }
}

pub struct ECS {
    pub planner: specs::Planner<TickData>,
    pub render_tx: Sender<DrawCommand>,
}

impl ECS {
    pub fn new(render_tx: Sender<DrawCommand>,
               bat_sheet: &SpriteSheetData,
               pitcher_sheet: &SpriteSheetData,
               power_meter_sheet: &SpriteSheetData,
               pointer_sheet: &SpriteSheetData) -> ECS {

        let mut world = specs::World::new();
        world.register::<components::Pitcher>();
        world.register::<components::PowerMeter>();
        world.register::<components::Batter>();
        world.register::<components::Bat>();
        world.register::<components::Ball>();
        world.register::<components::GameFlow>();

        // entities are created by combining various components via the world
        world.create_now()
            .with(components::Pitcher {
                action_ttl: 0., // will get set by system when we enter the winding phase
                active_clip: Some(pitcher_sheet.clips.create("Ready", PlayMode::Loop).unwrap()),
            })
            .with(components::Batter { })
            .with(components::PowerMeter {
                active_clip: Some(power_meter_sheet.clips.create("No Bar", PlayMode::Hold).unwrap()),
                pointer_clip: pointer_sheet.clips.create("Default", PlayMode::Loop).unwrap(),
                power_level: 0.,
                time: 0.
            })
            .with(components::GameFlow { active: GamePhase::WaitingForPlayer })
            .build();

        let mut plan = specs::Planner::new(world, 1);

        let power_sys = systems::PowerMeterSys {
            clips: power_meter_sheet.clips.clone()
        };
        plan.add_system(power_sys, "power", 10);

        let batter_sys = systems::BatterThink { clips: bat_sheet.clips.clone() };
        plan.add_system(batter_sys, "batter", 10);

        let pitch_sys = systems::PitcherThink {
            clips: pitcher_sheet.clips.clone(),
        };
        plan.add_system(pitch_sys, "pitcher", 15);

        let render_sys = systems::Render { tx: render_tx.clone() };
        plan.add_system(render_sys, "render", 100);

        ECS {
            planner: plan,
            render_tx: render_tx
        }
    }

    pub fn tick(&mut self, tick_data: TickData) -> bool {
        self.planner.dispatch(tick_data);
        self.planner.wait();
        true
    }
}

struct MainState {
    assets: AssetBundle,
    last_tick: TickData,
    current_tick: TickData,
    ecs: ECS,
    render_rx: Receiver<DrawCommand>,
    pitcher_sheet: SpriteSheetData,
    power_meter_sheet: SpriteSheetData,
    pointer_sheet: SpriteSheetData,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        ctx.print_resource_stats();

        // render pipe - Sender/Receiver
        let (tx, rx) = channel::<DrawCommand>();

        let bat_sheet = SpriteSheetData::from_file("resources/bat.json");
        let pitcher_sheet = SpriteSheetData::from_file("resources/pitching-machine.json");
        let power_meter_sheet = SpriteSheetData::from_file("resources/bar.json");
        let pointer_sheet = SpriteSheetData::from_file("resources/pointer.json");

        let s = MainState {
            assets: AssetBundle::new(ctx, &vec![
                "background.png",
                "pitching-machine.png",
                "bar.png",
                "pointer.png"
            ]),
            ecs: ECS::new(tx, &bat_sheet, &pitcher_sheet, &power_meter_sheet, &pointer_sheet),
            last_tick: TickData::new(),
            current_tick: TickData::new(),
            render_rx: rx,
            pitcher_sheet: pitcher_sheet,
            power_meter_sheet: power_meter_sheet,
            pointer_sheet: pointer_sheet,
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
//        println!("{:?}", self.current_tick);
        self.ecs.tick(self.current_tick.clone());
        self.last_tick = self.current_tick.clone();

        timer::sleep_until_next_frame(_ctx, 60);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        // draw the background before all the dynamic stuff
        let bg = self.assets.get_image(ctx, "background.png");

        graphics::draw(ctx, bg, graphics::Point::new(1024. / 2., 768. / 2.), 0.)?;

        for cmd in self.render_rx.try_iter() {
            match cmd {
                DrawCommand::DrawTransformed { path, x, y, rot , .. } => {
                    let image = self.assets.get_image(ctx, path.as_ref());
                    graphics::draw(ctx, image, graphics::Point::new(x, y), rot)?;
                }
                DrawCommand::DrawSpriteSheetCell(name, idx, pos, scale) => {
                    let atlas = self.assets.get_image(ctx, name.as_ref());
                    let w = atlas.width() as f32;
                    let h = atlas.height() as f32;

                    let maybe_cell = match name.as_ref() {
                        "pitching-machine.png" => Some(&self.pitcher_sheet.cells[idx]),
                        "bar.png" => Some(&self.power_meter_sheet.cells[idx]),
                        "pointer.png" => Some(&self.pointer_sheet.cells[idx]),
                        _ => None
                    };

                    if let Some(cell) = maybe_cell {
                        let param = graphics::DrawParam {
                            src: graphics::Rect::new(
                                cell.bbox.x as f32 / w,
                                cell.bbox.y as f32 / h,
                                cell.bbox.width as f32 / w,
                                cell.bbox.height as f32 / h),
                            dest: pos,
                            scale: scale,
                            ..Default::default()
                        };

                        graphics::draw_ex(ctx, atlas,  param)?;
                    }
                },
                _ => ()
            }
        }
        graphics::present(ctx);
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
