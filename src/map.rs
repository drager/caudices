use quicksilver::graphics::Color;
use serde_json;
use specs::{
    Component, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadStorage, System, VecStorage,
    WriteStorage,
};
use stages;
use utils::de_color;
use GameState;
use Position;

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
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

    fn run(&mut self, (entities, mut blocks, positions, _updater): Self::SystemData) {
        (&entities, &mut blocks, &positions)
            .join()
            .for_each(|(_entity, _blocks, _position)| {
                let _block = entities.create();
            });
    }
}

impl Component for Stage {
    type Storage = VecStorage<Self>;
}

pub struct StageCreator;

impl<'a> System<'a> for StageCreator {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Stage>,
        Read<'a, GameState>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (entities, _stones, game_state, updater): Self::SystemData) {
        let new_stage = entities.create();
        let stages = stages::get_stages();
        println!("Stages: {:?}", stages);

        stages
            .into_iter()
            .filter(|stage| stage.stage == game_state.current_stage)
            .for_each(|stage| {
                updater.insert(new_stage, stage);
            });
    }
}

pub fn parse_json(json_slice: &[u8]) -> Result<Vec<Stage>, serde_json::error::Error> {
    serde_json::from_slice::<Vec<Stage>>(json_slice)
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Stage {
    pub stage: u16,
    pub maps: Vec<Map>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Map {
    pub level: u16,
    pub time: i64,
    #[serde(rename = "blocks")]
    pub blocks_with_position: Vec<BlockAndPosition>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockAndPosition {
    #[serde(flatten)]
    pub block: Block,

    #[serde(flatten)]
    pub position: Position,
}
