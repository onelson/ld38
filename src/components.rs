use ggez::graphics::{Point, Rect};
use specs;
use omn_labs::sprites::{AnimationClip, SpriteSheetData};


#[derive(Clone, Debug)]
pub struct GameFlow {
    pub active: super::GamePhase
}

impl specs::Component for GameFlow {
    type Storage = specs::HashMapStorage<GameFlow>;
}

#[derive(Clone, Debug)]
pub struct Pitcher {
    pub action_ttl: f32,
    pub active_clip: Option<AnimationClip>,
}

impl specs::Component for Pitcher {
    type Storage = specs::HashMapStorage<Pitcher>;
}

#[derive(Clone, Debug)]
pub struct PowerMeter {
    pub time: f32,
    pub power_level: f32,
    pub active_clip: Option<AnimationClip>,
    pub pointer_clip: AnimationClip
}

impl specs::Component for PowerMeter {
    type Storage = specs::HashMapStorage<PowerMeter>;
}

#[derive(Clone, Debug)]
pub struct Batter;

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
