//! Sketching out how the data might be modeled for this game.
//! I think maybe it's like this:
//!
//! * batter starts with ready = false
//! * player input sets batter ready = true
//! * pitcher (should initialize ready = true) begins wind-up, then pitches (setting ready = false)
//! * when pitcher releases the ball, we set velocity/angle on it and let it travel
//! * we check for bbox overlap while Bat.swinging is true
//! * if bat and ball collide, then we do a reflection calc for the angle and pump up the
//!   velocity on the ball
//! * largeish bbox for bounds checking - when ball exits pitcher is ready (again)
//! *
//!

use omn_labs;
use specs;
use specs::Join;

use components::*;

#[derive(Clone, Debug)]
pub struct Pitch {
    pub factor: f32
}

impl specs::System<omn_labs::Delta> for Pitch {
    fn run(&mut self, arg: specs::RunArg, dt: omn_labs::Delta) {

        let (batter, mut pitcher) = arg.fetch(|w| {
            (w.read::<Batter>(), w.write::<Pitcher>())
        });

        // FIXME compiler thinks it can't join these
//        for (p, b) in (&mut pitcher, &batter).join() {
//
//        }
    }
}