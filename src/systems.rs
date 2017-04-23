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

use omn_labs;
use omn_labs::sprites::{ClipStore, PlayMode};
use components::*;
use super::{InputState, TickData};

pub enum DrawCommand {
    DrawTransformed {
        path: String,
        x: f32,
        y: f32,
        rot: f32,
        sx: f32,
        sy: f32,
    },
    DrawSpriteSheetCell(String, usize, graphics::Point),
}


#[derive(Clone, Debug)]
pub struct BatterThink {
    pub clips: ClipStore
}

impl specs::System<TickData> for BatterThink {
    fn run(&mut self, arg: specs::RunArg, data: TickData) {

        let mut batter = arg.fetch(|w| { w.write::<Batter>() });

        for entity in (&mut batter).iter() {
            if !entity.ready
                && (data.input_state == InputState::Pressed
                    || data.input_state == InputState::JustReleased) {
                entity.ready = true;
                println!("Batter Up!");
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

        let (batter, mut pitcher) = arg.fetch(|w| {
            (w.read::<Batter>(), w.write::<Pitcher>())
        });

        for (p, b) in (&mut pitcher, &batter).iter() {

            let active_clip = p.clone().active_clip.unwrap();

            if p.ready && b.ready && !p.winding {
                println!("Pitch system wants to pitch!");
                p.winding = true;
                p.active_clip = Some(self.clips.create("Winding", PlayMode::OneShot).unwrap());
                println!("Pitcher is winding!");
            } else if p.winding && active_clip.drained() {
                p.ready = false; // ball is in flight, and pitcher won't be ready again until the ball is recovered.
                p.active_clip = Some(self.clips.create("Pitching", PlayMode::Loop).unwrap());
                println!("Pitcher is pitching!");
            }

            if let Some(ref mut clip) = p.active_clip {
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
//                    println!("Cell: {}", idx);
                    self.tx.send(DrawCommand::DrawSpriteSheetCell(
                        "pitcher.png".to_string(),
                        idx,
                        graphics::Point::new(800., 500.))  // FIXME: quit hardcoding position
                    ).unwrap();
                }

            }
        }
    }
}

