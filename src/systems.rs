//! Sketching out how the data might be modeled for this game.
//! I think maybe it's like this:
//!
//! * batter starts with ready = false
//! * player input sets batter ready = true
//! * pitcher (should initialize ready = true) begins wind-up, then pitches (setting ready = false)
//! * when pitcher releases the ball, we set velocity/angle on it and let it travel
//! * we check for bbox overlap while `Bat.swinging` is true
//! * if bat and ball collide, then we do a reflection calc for the angle and pump up the
//!   velocity on the ball
//! * largeish bbox for bounds checking - when ball exits pitcher is ready (again)
//! *
//!

use std::sync::mpsc::Sender;

use ggez::graphics;
use specs;
use specs::Join;
use rand::{self, Rng};

use omn_labs::sprites::{ClipStore, PlayMode};
use components::*;
use super::{InputState, TickData, GamePhase};

pub enum DrawCommand {
    DrawTransformed {
        path: String,
        x: f32,
        y: f32,
        rot: f32,
        sx: f32,
        sy: f32,
    },
    DrawSpriteSheetCell(String, usize, graphics::Point, graphics::Point),
}


#[derive(Clone, Debug)]
pub struct BatterThink {
    pub clips: ClipStore
}

fn key_pressed(input: &InputState) -> bool {
    match *input {
        InputState::Pressed | InputState::JustReleased => true,
        _ => false
    }
}

impl specs::System<TickData> for BatterThink {
    fn run(&mut self, arg: specs::RunArg, data: TickData) {

        let mut game_flow = arg.fetch(|w| { w.write::<GameFlow>() });

        for flow in (&mut game_flow).iter() {
            let maybe_phase = match (*flow).active {
                GamePhase::WaitingForPlayer => {
                    if key_pressed(&data.input_state) {
                        println!("Batter Up!");
                        Some(GamePhase::PlayerReady)
                    } else {
                        Some(GamePhase::WaitingForPlayer)
                    }
                },

                _ => None
            };

            if let Some(phase) = maybe_phase {
                flow.active = phase;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct PitcherThink {
    pub clips: ClipStore
}

impl specs::System<TickData> for PitcherThink {
    fn run(&mut self, arg: specs::RunArg, data: TickData) {

        let (batter, mut game_flow, mut pitcher) = arg.fetch(|w| {
            (w.read::<Batter>(), w.write::<GameFlow>(), w.write::<Pitcher>())
        });

        for (flow, pitch, bat) in (&mut game_flow, &mut pitcher, &batter).iter() {
            let drained = {
                if let Some(ref clip) = pitch.active_clip { clip.drained } else { true }
            };

            let mut rng = rand::thread_rng();

            let maybe_phase = match (*flow).active {
                GamePhase::PlayerReady => {
                    println!("Pitch system wants to pitch!");
                    pitch.active_clip = Some(self.clips.create("Winding", PlayMode::Loop).unwrap());
                    pitch.action_ttl = 3000. + (rng.gen::<f32>() * 2500.);
                    println!("Pitcher is winding up for {}!", pitch.action_ttl);
                    Some(GamePhase::Windup)
                },
                GamePhase::Windup => {
                    pitch.action_ttl -= data.delta_ms;
                    if pitch.action_ttl < 0. {
                        let clip = self.clips.create("Pitching", PlayMode::OneShot).unwrap();
                        let duration = clip.duration;
                        pitch.active_clip = Some(clip);
                        println!("Pitcher is pitching for {}!", duration);
                        Some(GamePhase::Pitching)
                    } else {
                        Some(GamePhase::Windup)
                    }
                },
                GamePhase::Pitching if drained => {
                    Some(GamePhase::BallInFlight)
                },
                GamePhase::BallInFlight => {
                    let mut already_idle = false;
                    if let Some(ref clip) = pitch.active_clip {
                        already_idle = clip.name == "Not Ready";
                    }

                    if !already_idle {
                        pitch.action_ttl = 5000.;
                        pitch.active_clip = Some(self.clips.create("Not Ready", PlayMode::Loop).unwrap())
                    } else {
                        pitch.action_ttl -= data.delta_ms;
                    }

                    if pitch.action_ttl < 0. {
                        // FIXME: just a temp game state reset until we have the player side implemented
                        pitch.active_clip = Some(self.clips.create("Ready", PlayMode::Loop).unwrap());
                        Some(GamePhase::WaitingForPlayer)
                    } else {
                        Some(GamePhase::BallInFlight)
                    }

                },
                _ => None

            };

            if let Some(phase) = maybe_phase {
                flow.active = phase;
            }

            if let Some(ref mut clip) = pitch.active_clip {
//                println!("{}", clip.name);
                clip.update(data.delta_ms);
            }
        }
    }
}


#[derive(Clone, Debug)]
pub struct Render {
    pub tx: Sender<DrawCommand>
}

impl specs::System<TickData> for Render {
    fn run(&mut self, arg: specs::RunArg, data: TickData) {

        let (batter, pitcher) = arg.fetch(|w| {
            (w.read::<Batter>(), w.read::<Pitcher>())
        });

        for (pitch, bat) in (&pitcher, &batter).iter() {
//            println!("Render: {:?}", pitch);
//            println!("Render: {:?}", bat);

            if let Some(ref clip) = pitch.active_clip {
                if let Some(idx) = clip.get_cell() {

//                    println!("Clip: nam={}, cell={}", clip.name, idx);
                    self.tx.send(DrawCommand::DrawSpriteSheetCell(
                        "pitching-machine.png".to_string(),
                        idx,
                        graphics::Point::new(512., 530.),
                        graphics::Point::new(2., 2.))
                    ).unwrap();
                }

            }
        }
    }
}

