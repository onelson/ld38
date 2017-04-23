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

use omn_labs;
use specs;

use specs::Join;

use components::*;
use super::{InputState, TickData};

#[derive(Clone, Debug)]
pub struct BatterThink;

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
    pub factor: f32
}

impl specs::System<TickData> for PitcherThink {
    fn run(&mut self, arg: specs::RunArg, data: TickData) {

        let (batter, mut pitcher) = arg.fetch(|w| {
            (w.read::<Batter>(), w.write::<Pitcher>())
        });

        for (p, b) in (&mut pitcher, &batter).iter() {
            if p.ready && b.ready && !p.winding {
                println!("Pitch system wants to pitch!");
                p.winding = true;
            }

            if p.winding {
                println!("Pitcher is winding!");
                // TODO: add some sort of duration to the wind (based on animation clips?)
            }
        }
    }
}