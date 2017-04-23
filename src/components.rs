use ggez::graphics::{Point, Rect};
use specs;

#[derive(Clone, Debug)]
pub struct Pitcher {
    pub ready: bool,
    pub winding: bool
}

impl specs::Component for Pitcher {
    type Storage = specs::HashMapStorage<Pitcher>;
}

#[derive(Clone, Debug)]
pub struct Batter {
    pub ready: bool
}

impl specs::Component for Batter {
    type Storage = specs::HashMapStorage<Batter>;
}

#[derive(Clone, Debug)]
pub struct Bat {
    pub swinging: bool,
    pub bbox: Rect
}

impl specs::Component for Bat {
    type Storage = specs::HashMapStorage<Bat>;
}

#[derive(Clone, Debug)]
pub struct Ball {
    pub bbox: Rect,
    pub pos: Point,
    pub angle: f32,
    pub velocity: f32,
    pub out_of_bounds: bool
}

impl specs::Component for Ball {
    type Storage = specs::HashMapStorage<Ball>;
}


#[derive(Clone, Debug)]
pub struct OuterSpace {
    pub y: f32,
}

impl specs::Component for OuterSpace {
    type Storage = specs::HashMapStorage<OuterSpace>;
}

#[derive(Clone, Debug)]
pub struct Ground {
    pub y: f32,
}

impl specs::Component for Ground {
    type Storage = specs::HashMapStorage<Ground>;
}
