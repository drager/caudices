use nalgebra::Vector2;
use quicksilver::graphics::Color;
use serde_json;
use specs::{
    prelude::Resources, Builder, Component, Entities, HashMapStorage, LazyUpdate, Read,
    ReadStorage, System, VecStorage, World, WriteStorage,
};
use utils::de_color;
use Position;
use ScreenState;
use Settings;
use WINDOW_HEIGHT;
use WINDOW_WIDTH;

pub fn find_current_map(stages: Vec<Stage>, state: &ScreenState) -> Option<Map> {
    stages
        .into_iter()
        .find(|stage| stage.stage == state.current_stage)
        .and_then(|stage| {
            stage
                .maps
                .into_iter()
                .find(|map| map.level == state.current_level)
        })
}

pub fn create_base_map_entities(
    world: &mut World,
    settings: &Settings,
) -> Result<(), quicksilver::Error> {
    for width_index in 0..=(WINDOW_WIDTH / 50) - 2 {
        // The top.
        world
            .create_entity()
            .with(Block::default())
            .with(Position(Vector2::new(
                (width_index + 1) as f32 * 50.,
                settings.header_height,
            )))
            .build();
        // The bottom.
        world
            .create_entity()
            .with(Block::default())
            .with(Position(Vector2::new(
                (width_index + 1) as f32 * 50.,
                (WINDOW_HEIGHT as u16 - 50).into(),
            )))
            .build();
    }

    for height_index in 2..=(WINDOW_HEIGHT / 50) - 2 {
        // Left side.
        world
            .create_entity()
            .with(Block::default())
            .with(Position(Vector2::new(50., (height_index + 1) as f32 * 50.)))
            .build();

        // Right side
        world
            .create_entity()
            .with(Block::default())
            .with(Position(Vector2::new(
                (WINDOW_WIDTH as u16 - 50).into(),
                (height_index + 1) as f32 * 50.,
            )))
            .build();
    }

    Ok(())
}

pub fn parse_json(json_slice: &[u8]) -> Result<Vec<Stage>, serde_json::error::Error> {
    serde_json::from_slice::<Vec<Stage>>(json_slice)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Default for Size {
    fn default() -> Self {
        Size {
            width: 50.0,
            height: 50.0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Block {
    pub can_be_moved: bool,
    #[serde(default)]
    pub size: Size,

    #[serde(deserialize_with = "de_color")]
    pub color: Color,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            can_be_moved: false,
            size: Size {
                width: 50.0,
                height: 50.0,
            },
            color: Color::RED,
        }
    }
}

impl Component for Block {
    type Storage = HashMapStorage<Self>;
}

pub struct BlockSystem;

impl<'a> System<'a> for BlockSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Block>,
        ReadStorage<'a, Position>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (_entities, _blocks, _positions, _updater): Self::SystemData) {}
}

impl Component for Stage {
    type Storage = VecStorage<Self>;
}

pub struct StageCreator;

impl<'a> System<'a> for StageCreator {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Stage>,
        Read<'a, ScreenState>,
        Read<'a, LazyUpdate>,
    );

    fn setup(&mut self, res: &mut Resources) {
        use specs::prelude::SystemData;
        Self::SystemData::setup(res);
    }

    fn run(&mut self, (_entities, _stones, _screen_state, _updater): Self::SystemData) {}
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Stage {
    pub stage: u16,
    pub maps: Vec<Map>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Map {
    pub level: u16,
    pub time: u64,
    #[serde(rename = "blocks")]
    pub blocks_with_position: Vec<BlockAndPosition>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BlockAndPosition {
    #[serde(flatten)]
    pub block: Block,

    #[serde(flatten)]
    pub position: Position,
}
