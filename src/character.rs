use quicksilver::geom::Rectangle;
use specs::{Component, HashMapStorage};

#[derive(Debug)]
pub struct Character {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug)]
pub enum CharacterPosition {
    Start(Rectangle),
    Moving(Rectangle),
}

impl Default for Character {
    fn default() -> Self {
        Character {
            width: 50.,
            height: 50.,
        }
    }
}

impl Component for Character {
    type Storage = HashMapStorage<Self>;
}
