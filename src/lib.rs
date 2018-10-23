extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate quicksilver;
extern crate specs;
extern crate time;

pub mod map;
mod stages;
mod utils;

use map::{BlockSystem, Stage, StageCreator};
use quicksilver::{
    geom::{Rectangle, Shape, Vector},
    graphics::{
        Background::{self, Img},
        Color, Font, FontStyle,
    },
    lifecycle::{run, Asset, Settings, State, Window},
    Result,
};
use specs::{
    Component, Dispatcher, DispatcherBuilder, Join, Read, ReadStorage, System, VecStorage, World,
    WriteStorage,
};
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize)]
pub struct GameState {
    pub current_stage: u16,
    pub current_level: u16,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            current_stage: 1,
            current_level: 1,
        }
    }
}

impl Component for GameState {
    type Storage = VecStorage<Self>;
}

#[derive(Default)]
struct DeltaTime(f32);

struct PhysicsSystem;

impl<'a> System<'a> for PhysicsSystem {
    type SystemData = (
        Read<'a, DeltaTime>,
        ReadStorage<'a, Velocity>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (delta, velocity, mut position) = data;

        let delta = delta.0;

        (&velocity, &mut position)
            .join()
            .for_each(|(velocity, position)| {
                position.position.x += velocity.x * delta;
                position.position.y += velocity.y * delta;
            })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Position {
    position: Vector,
}

impl Position {
    fn new(x: f32, y: f32) -> Self {
        Position {
            position: Vector::new(x, y),
        }
    }
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
struct Velocity {
    x: f32,
    y: f32,
}

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}

pub struct Screen {
    world: World,
    time_elapsed: Duration,
    mali_font: Asset<Font>,
}

impl State for Screen {
    fn new() -> Result<Self> {
        let mut world = World::new();

        let mali_font = Asset::new(Font::load("mali/Mali-Regular.ttf"));

        world.add_resource(DeltaTime(10.05));
        world.add_resource(GameState::default);
        let mut dispatcher: Dispatcher = DispatcherBuilder::new()
            .with(StageCreator, "stage_creator", &[])
            .with(PhysicsSystem, "physics_system", &[])
            .with(BlockSystem, "block_system", &["physics_system"])
            .build();

        dispatcher.setup(&mut world.res);

        //let _ = map::load_json_from_file("stages.json").execute(|json_file| {
        //let stages = map::parse_json(&json_file).expect("Could not load the maps");
        /*let stages = get_stages();*/
        //println!("{:?}", stages);

        //stages.into_iter().for_each(|stage| {
        //stage.maps.into_iter().for_each(|map| {
        //map.blocks_with_position
        //.into_iter()
        //.for_each(|block_with_position| {
        //println!("Creating block: {:?}", block_with_position);
        //world
        //.create_entity()
        //.with(block_with_position.block)
        //.with(block_with_position.position)
        //.build();
        //});
        //});
        /*});*/

        //Ok(())
        //});

        dispatcher.dispatch(&world.res);

        world.maintain();

        let screen = Screen {
            world,
            time_elapsed: Duration::new(0, 0),
            mali_font,
        };

        Ok(screen)
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        self.time_elapsed += Duration::from_millis(window.update_rate() as u64);

        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;

        /*let positions = self.world.read_storage::<Position>();*/
        /*let blocks = self.world.read_storage::<Block>();*/
        let entities = self.world.entities();
        let stages = self.world.read_storage::<Stage>();

        for entity in entities.join() {
            if let Some(stage) = stages.get(entity) {
                stage
                    .maps
                    .iter()
                    //.filter(|map| map.level == game_state.current_level)
                    .for_each(|map| {
                        map.blocks_with_position
                            .iter()
                            .for_each(|block_with_position| {
                                let block = &block_with_position.block;
                                let position = block_with_position.position.position;
                                window.draw(
                                    &Rectangle::new(
                                        position,
                                        (block.size.width, block.size.height),
                                    ),
                                    Background::Col(block.color),
                                );
                            })
                    });
            };
        }

        let time_elapsed = self.time_elapsed;

        self.mali_font.execute(move |font| {
            let style = FontStyle::new(72.0, Color::WHITE);
            let text = font.render(&format!("{}", time_elapsed.as_secs()), &style)?;
            window.draw(&text.area().with_center((70, 50)), Img(&text));
            Ok(())
        })
    }
}

pub fn start() {
    run::<Screen>(
        "Caudices",
        Vector::new(800, 600),
        Settings {
            update_rate: 1000.0,
            ..Settings::default()
        },
    );
}
