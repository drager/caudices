use log::log;
use quicksilver::graphics::Color;
use serde_json;
use specs::{
    Component, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadStorage, System, VecStorage,
    WriteStorage,
};
use utils::{self, de_color};
use Position;
use {GameState, ScreenState};

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
        Read<'a, ScreenState>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (entities, _stones, screen_state, updater): Self::SystemData) {
        let new_stage = entities.create();
        log("Running stage_creator");
        let result = utils::load_json_from_file("stages.json").execute(|json_file| {
            log(&format!("FILE: {:?}", json_file));
            let stages = parse_json(&json_file).expect("Could not load the maps");
            log(&format!("{:?}", stages));
            stages
                .into_iter()
                .filter(|stage| stage.stage == screen_state.current_stage)
                .for_each(|stage| {
                    updater.insert(new_stage, stage);
                });

            Ok(())
        });
        log(&format!("{:?}", result));
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
    pub time: u64,
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
