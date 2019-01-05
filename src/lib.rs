#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate serde_derive;
extern crate cfg_if;
extern crate futures;
extern crate hibitset;
extern crate nalgebra;
extern crate ncollide2d;
extern crate quicksilver;
extern crate serde;
extern crate serde_json;
extern crate specs;
extern crate time;

/*extern crate console_error_panic_hook;*/
//pub use console_error_panic_hook::set_once as set_panic_hook;

mod character;
mod collision;
mod log;
pub mod map;
mod physics;
mod stages;
mod utils;

use character::{Character, CharacterPosition};
use collision::{Collision, CollisionSystem};
use futures::future;
use physics::{DeltaTime, PhysicsSystem, Position, Velocity};
//use log::log;
use map::{Block, BlockSystem, Map, Stage, StageCreator};
use quicksilver::{
    geom::{Rectangle, Shape, Vector},
    graphics::{Animation, Background::Img, Color, Font, FontStyle, Image},
    input::{ButtonState, Key},
    lifecycle::{run, Asset, Settings as QuickSilverSettings, State, Window},
    load_file, Future, Result,
};
use specs::{Builder, Component, Dispatcher, DispatcherBuilder, Join, VecStorage, World};
use std::time::Duration;

const WINDOW_WIDTH: u16 = 600;
const WINDOW_HEIGHT: u16 = 600;

#[derive(Debug, Deserialize, Serialize)]
pub enum GameState {
    Active,
    Paused,
    Over,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DrawState {
    Drawed,
    Undrawed,
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Active
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScreenState {
    pub current_stage: u16,
    pub current_level: u16,
    pub game_state: GameState,
    pub draw_state: DrawState,
}

impl Default for ScreenState {
    fn default() -> Self {
        ScreenState {
            current_stage: 1,
            current_level: 1,
            game_state: GameState::Active,
            draw_state: DrawState::Undrawed,
        }
    }
}

impl Component for ScreenState {
    type Storage = VecStorage<Self>;
}

pub struct Screen<'a> {
    world: World,
    time_elapsed: Duration,
    settings: Settings,
    game_asset: GameAsset,
    dispatcher: Dispatcher<'a, 'a>,
}

impl<'a> Screen<'a> {
    fn draw_character(
        window: &mut Window,
        position: &Position,
        character_animation: &Animation,
    ) -> Result<()> {
        let current_frame = character_animation.current_frame();
        window.draw(
            &current_frame.area().with_center(position.0),
            Img(&current_frame),
        );
        Ok(())
    }

    fn draw_time_left(
        window: &mut Window,
        time_elapsed: &Duration,
        map: &Map,
        asset_font: &mut Asset<Font>,
    ) -> Result<()> {
        let font_style = FontStyle::new(72.0, Color::WHITE);
        let map_time = map.time / 1000;
        let time_elapsed_as_secs = time_elapsed.as_secs();

        if time_elapsed_as_secs <= map_time {
            asset_font.execute(|font| {
                let _ = font
                    .render(&format!("{}", map_time - time_elapsed_as_secs), &font_style)
                    .map(|text| {
                        window.draw(&text.area().with_center((70, 50)), Img(&text));
                    });
                Ok(())
            })
        } else {
            Ok(())
        }
    }

    fn draw_blocks(
        window: &mut Window,
        map: &Map,
        block_asset: &mut Asset<Image>,
    ) -> Vec<Result<()>> {
        map.blocks_with_position
            .iter()
            .map(|block_with_position| {
                let position = block_with_position.position.0;

                block_asset.execute(|image| {
                    window.draw(&image.area().with_center(position), Img(&image));
                    Ok(())
                })
            })
            .collect::<Vec<Result<_>>>()
    }

    fn handle_right_key_for_character<'b>(
        window: &mut Window,
        position: &mut Position,
        character_animation: &mut Animation,
        animation_positions: &Vec<CharacterPosition>,
    ) {
        Self::handle_key_for_character(
            position,
            character_animation,
            animation_positions,
            &window.keyboard()[Key::Right],
            // TODO: Get position increament from settings.
            |position| position.0.x += 1.,
        );
    }

    fn handle_left_key_for_character<'b>(
        window: &mut Window,
        position: &mut Position,
        character_animation: &mut Animation,
        animation_positions: &Vec<CharacterPosition>,
    ) {
        Self::handle_key_for_character(
            position,
            character_animation,
            animation_positions,
            &window.keyboard()[Key::Left],
            // TODO: Get position increament from settings.
            |position| position.0.x -= 1.,
        );
    }

    fn handle_down_key_for_character<'b>(
        window: &mut Window,
        position: &mut Position,
        character_animation: &mut Animation,
        animation_positions: &Vec<CharacterPosition>,
    ) {
        Self::handle_key_for_character(
            position,
            character_animation,
            animation_positions,
            &window.keyboard()[Key::Down],
            // TODO: Get position increament from settings.
            |position| position.0.y += 1.,
        );
    }

    fn handle_up_key_for_character<'b>(
        window: &mut Window,
        position: &mut Position,
        character_animation: &mut Animation,
        animation_positions: &Vec<CharacterPosition>,
    ) {
        Self::handle_key_for_character(
            position,
            character_animation,
            animation_positions,
            &window.keyboard()[Key::Up],
            // TODO: Get position increament from settings.
            |position| position.0.y -= 1.0,
        );
    }

    fn handle_key_for_character<'b, P: FnOnce(&mut Position) -> ()>(
        position: &mut Position,
        character_animation: &mut Animation,
        animation_positions: &Vec<CharacterPosition>,
        key_state: &ButtonState,
        position_changer: P,
    ) {
        let mut animation_positions_iter = animation_positions.into_iter();
        let start_position = animation_positions_iter.find(|pos| match pos {
            CharacterPosition::Start(_) => true,
            _ => false,
        });
        let moving_position = animation_positions_iter.find(|pos| match pos {
            CharacterPosition::Moving(_) => true,
            _ => false,
        });

        let current_frame_area = character_animation.current_frame().area();

        match key_state {
            ButtonState::Pressed | ButtonState::Held => {
                position_changer(position);
                //log(&format!("{:?}", current_frame_area));
                match current_frame_area {
                    Rectangle {
                        pos: Vector { x, .. },
                        ..
                    } => {
                        if let Some(start_position) = start_position {
                            if let CharacterPosition::Start(rectangle) = start_position {
                                if x == rectangle.pos.x {
                                    character_animation.tick();

                                    //log("Animation ticking...");
                                }
                            }
                        }
                    }
                }
            }
            ButtonState::Released => match current_frame_area {
                Rectangle {
                    pos: Vector { x, .. },
                    ..
                } => {
                    if let Some(moving_position) = moving_position {
                        if let CharacterPosition::Moving(rectangle) = moving_position {
                            if x == rectangle.pos.x {
                                character_animation.tick();

                                //log("Animation ticking for moving...");
                            }
                        }
                    }
                }
            },
            _ => {}
        }
    }

    fn load_fonts(settings: &Settings) -> Asset<Font> {
        Asset::new(Font::load(settings.mali_font_path.to_owned()))
    }

    fn load_character_asset(
        animation_positions: Vec<Rectangle>,
        settings: &Settings,
    ) -> Asset<Animation> {
        let frame_delay = 1;

        let character_image =
            Image::load(settings.character_sprites_path.to_owned()).map(move |character_image| {
                Animation::from_spritesheet(
                    character_image.to_owned(),
                    animation_positions,
                    frame_delay,
                )
            });

        Asset::new(character_image)
    }

    fn load_block_asset(settings: &Settings) -> Asset<Image> {
        Asset::new(Image::load(settings.block_asset_path.to_owned()))
    }

    fn load_stages(settings: &Settings) -> Asset<Vec<Stage>> {
        let stages_file =
            load_file(settings.stages_json_path.to_owned()).and_then(move |stages_bytes| {
                let stages = map::parse_json(&stages_bytes);
                future::result(stages.map_err(|_err| {
                    quicksilver::Error::ContextError("Couldn't parse json.".to_owned())
                }))
            });

        Asset::new(stages_file)
    }
}

struct GameAsset {
    mali_font: Asset<Font>,
    character_asset: Asset<Animation>,
    block_asset: Asset<Image>,
    stages: Asset<Vec<Stage>>,
}

#[derive(Debug)]
struct Settings {
    animation_positions: Vec<CharacterPosition>,
    mali_font_path: String,
    character_sprites_path: String,
    block_asset_path: String,
    stages_json_path: String,
    header_height: f32,
}

fn find_current_map(stages: Vec<Stage>, state: &ScreenState) -> Option<Map> {
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

fn create_base_map_entities(world: &mut World, settings: &Settings) -> Result<()> {
    for width_index in 0..=(WINDOW_WIDTH / 50) - 2 {
        // The top.
        world
            .create_entity()
            .with(Block::default())
            .with(Position::new(
                (width_index + 1) as f32 * 50.,
                settings.header_height,
            ))
            .build();
        // The bottom.
        world
            .create_entity()
            .with(Block::default())
            .with(Position::new(
                (width_index + 1) as f32 * 50.,
                (WINDOW_HEIGHT as u16 - 50).into(),
            ))
            .build();
    }

    for height_index in 2..=(WINDOW_HEIGHT / 50) - 2 {
        // Left side.
        world
            .create_entity()
            .with(Block::default())
            .with(Position::new(50., (height_index + 1) as f32 * 50.))
            .build();

        // Right side
        world
            .create_entity()
            .with(Block::default())
            .with(Position::new(
                (WINDOW_WIDTH as u16 - 50).into(),
                (height_index + 1) as f32 * 50.,
            ))
            .build();
    }

    Ok(())
}

impl State for Screen<'static> {
    fn new() -> Result<Self> {
        let animation_start_position = Rectangle::new(Vector::new(0, 12), Vector::new(29, 21));
        let animation_moving_position = Rectangle::new(Vector::new(32, 12), Vector::new(28, 21));
        let animation_positions = vec![animation_start_position, animation_moving_position];

        let settings = Settings {
            animation_positions: vec![
                CharacterPosition::Start(animation_start_position),
                CharacterPosition::Moving(animation_moving_position),
            ],
            mali_font_path: "mali/Mali-Regular.ttf".to_owned(),
            character_sprites_path: "character_sprite_0_white.png".to_owned(),
            block_asset_path: "50x50.png".to_owned(),
            stages_json_path: "stages.json".to_owned(),
            header_height: 100.,
        };

        let mut world = World::new();

        let collisions = PhysicsSystem::init_collision_world();

        world.add_resource(DeltaTime(10.0));
        world.add_resource(collisions);

        let mut dispatcher: Dispatcher = DispatcherBuilder::new()
            .with(StageCreator, "stage_creator", &[])
            .with(CollisionSystem, "collision_system", &[])
            .with(PhysicsSystem, "physics_system", &["collision_system"])
            .with(BlockSystem, "block_system", &[])
            .build();

        dispatcher.setup(&mut world.res);

        world
            .create_entity()
            .with(Velocity { x: 0., y: 0. })
            .with(Position::new(130., 330.))
            .with(Character::default())
            .build();

        world
            .create_entity()
            .with(Position::new(150., 150.))
            .with(Block::default())
            .build();

        world
            .create_entity()
            .with(Position::new(150., 200.))
            .with(Block::default())
            .build();

        //create_base_map_entities(&mut world, &settings)?;

        dispatcher.dispatch(&world.res);

        world.maintain();

        PhysicsSystem::setup_handles(
            &world.entities(),
            &mut world.write_resource::<Collision>(),
            &world.read_storage::<Velocity>(),
            &world.read_storage::<Position>(),
            &world.read_storage::<Character>(),
        );

        let mali_font = Screen::load_fonts(&settings);
        let block_asset = Screen::load_block_asset(&settings);
        let character_asset = Screen::load_character_asset(animation_positions, &settings);
        let stages = Screen::load_stages(&settings);

        let game_asset = GameAsset {
            mali_font,
            block_asset,
            character_asset,
            stages,
        };

        let screen = Screen {
            world,
            time_elapsed: Duration::new(0, 0),
            settings,
            game_asset,
            dispatcher,
        };

        Ok(screen)
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        self.time_elapsed += Duration::from_millis(10);
        self.world.maintain();
        self.dispatcher.dispatch(&self.world.res);

        let mut screen_state = self.world.write_resource::<ScreenState>();
        let characters = self.world.read_storage::<Character>();
        let stages = self.world.read_storage::<Stage>();
        let mut positions = self.world.write_storage::<Position>();
        let mut velocity_storage = self.world.write_storage::<Velocity>();
        let entities = self.world.entities();

        let character_asset = &mut self.game_asset.character_asset;
        let animation_positions = &self.settings.animation_positions;
        let time_elapsed = self.time_elapsed;

        let handle_keys = |velocity: &mut Velocity| {
            let mut key_match =
                |key: Key,
                 on_press: Box<Fn(&mut Velocity) -> ()>,
                 on_release: Box<Fn(&mut Velocity) -> ()>| match window.keyboard()
                    [key]
                {
                    ButtonState::Pressed | ButtonState::Held => {
                        on_press(velocity);
                    }
                    ButtonState::Released => {
                        on_release(velocity);
                    }
                    ButtonState::NotPressed => {}
                };

            let velocity_change = 0.1;

            key_match(
                Key::Up,
                Box::new(move |velocity| velocity.y = -velocity_change),
                Box::new(|velocity| velocity.y = 0.),
            );
            key_match(
                Key::Down,
                Box::new(move |velocity| velocity.y = velocity_change),
                Box::new(|velocity| velocity.y = 0.),
            );
            key_match(
                Key::Left,
                Box::new(move |velocity| velocity.x = -velocity_change),
                Box::new(|velocity| velocity.x = 0.),
            );
            key_match(
                Key::Right,
                Box::new(move |velocity| velocity.x = velocity_change),
                Box::new(|velocity| velocity.x = 0.),
            );
        };

        entities.join().for_each(|entity| {
            if let Some(stage) = stages.get(entity) {
                stage
                    .maps
                    .iter()
                    .find(|map| map.level == screen_state.current_level)
                    .map(|map| {
                        if time_elapsed.as_secs() >= map.time / 1000 {
                            screen_state.game_state = GameState::Over;
                        }
                    });
            }

            if let Some(_character) = characters.get(entity) {
                if let Some(velocity) = velocity_storage.get_mut(entity) {
                    handle_keys(velocity);
                }
            }

            /*let start_moving_state = || {*/
            //vec![
            //MovingState::Left,
            //MovingState::Right,
            //MovingState::Top,
            //MovingState::Bottom,
            //]
            //};

            //// TODO: Move this logic elsewhere[>.*
            //let move_character =
            //|moving_states: Vec<_>,
            //window: &mut Window,
            //position: &mut Position,
            //character_animation: &mut Animation,
            //animation_positions: &Vec<CharacterPosition>| {
            //moving_states.iter().for_each(|moving_state| {
            ////println!("Moving state {:?}", moving_state);
            //match moving_state {
            //MovingState::Bottom => Screen::handle_down_key_for_character(
            //window,
            //position,
            //character_animation,
            //animation_positions,
            //),
            //MovingState::Left => Screen::handle_left_key_for_character(
            //window,
            //position,
            //character_animation,
            //animation_positions,
            //),
            //MovingState::Right => Screen::handle_right_key_for_character(
            //window,
            //position,
            //character_animation,
            //animation_positions,
            //),
            //MovingState::Top => Screen::handle_up_key_for_character(
            //window,
            //position,
            //character_animation,
            //animation_positions,
            //),
            //}
            //});
            //};

            //match screen_state.game_state {
            //GameState::Active => {
            //if let Some(position) = positions.get_mut(entity) {
            //if let Some(_character) = characters.get(entity) {
            //let _ = character_asset.execute(|character_animation| {
            //let moving_states = start_moving_state();
            //move_character(
            //moving_states,
            //window,
            //position,
            //character_animation,
            //animation_positions,
            //);

            //Ok(())
            //});
            //}
            //}
            //}
            //_ => {}
            /*}*/
        });

        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;
        //log(&format!("Fps: {}", window.average_fps()));

        let world = &mut self.world;
        //let mut collision_world = self.collision_world;
        let entities = world.entities();
        let characters = world.read_storage::<Character>();
        let screen_state = world.write_resource::<ScreenState>();
        let positions = world.read_storage::<Position>();
        let stages = world.write_storage::<Stage>();
        let blocks = world.read_storage::<Block>();

        let font_style = FontStyle::new(72.0, Color::WHITE);

        let time_elapsed = self.time_elapsed;
        let mali_font = &mut self.game_asset.mali_font;
        let block_asset = &mut self.game_asset.block_asset;
        let character_asset = &mut self.game_asset.character_asset;
        //let stages_asset = &mut self.game_asset.stages;

        if let DrawState::Undrawed = screen_state.draw_state {

            /*let _ = stages_asset.execute(|fetched_stages| {*/
            //fetched_stages.iter().for_each(|stage| {
            //let entity = entities.create();
            //let _ = stages.insert(entity, stage.to_owned());
            //});
            //screen_state.draw_state = DrawState::Drawed;

            //Ok(())
            /*});*/
        }

        /*if let None = self.collision_world {*/
        //self.collision_world = Some(Collision::new(&world));
        /*}*/

        let mut active_rendering = |entity: specs::Entity,
                                    window: &mut Window,
                                    block_asset: &mut Asset<Image>,
                                    mali_font: &mut Asset<Font>|
         -> Result<()> {
            if let Some(position) = positions.get(entity) {
                if let Some(_character) = characters.get(entity) {
                    block_asset.execute(|image| {
                        window.draw(&image.area().with_center(position.0), Img(&image));
                        Ok(())
                    });
                    /* character_asset.execute(|character_image| {*/
                    //Screen::draw_character(window, position, character_image)?;
                    //Ok(())
                    /*})?;*/
                }
            }

            positions.get(entity).and_then(|position| {
                blocks.get(entity).map(|_block| {
                    block_asset.execute(|image| {
                        window.draw(&image.area().with_center(position.0), Img(&image));
                        Ok(())
                    })
                })
            });

            let current_map = stages.get(entity).and_then(|stage| {
                stage
                    .maps
                    .iter()
                    .find(|map| map.level == screen_state.current_level)
            });

            match current_map {
                Some(map) => {
                    Screen::draw_time_left(window, &time_elapsed, map, mali_font)?;

                    Screen::draw_blocks(window, map, block_asset);
                }
                None => {}
            };

            Ok(())
        };

        entities
            .join()
            .map(|entity| match screen_state.game_state {
                GameState::Active => active_rendering(entity, window, block_asset, mali_font),
                GameState::Active => Ok(()),
                GameState::Over => {
                    active_rendering(entity, window, block_asset, mali_font)?;
                    mali_font.execute(|font| {
                        let _ = font.render("Game over", &font_style).map(|text| {
                            window.draw(
                                &text
                                    .area()
                                    .with_center((WINDOW_WIDTH / 2, WINDOW_HEIGHT / 2)),
                                Img(&text),
                            );
                        });

                        Ok(())
                    })
                }
                GameState::Paused => mali_font.execute(|font| {
                    let _ = font.render("Paused", &font_style).map(|text| {
                        window.draw(
                            &text
                                .area()
                                .with_center((WINDOW_WIDTH / 2, WINDOW_HEIGHT / 2)),
                            Img(&text),
                        );
                    });

                    Ok(())
                }),
            })
            .collect()
    }
}

pub fn start() {
    run::<Screen>(
        "Caudices",
        Vector::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        QuickSilverSettings {
            //update_rate: 1000.0,
            ..QuickSilverSettings::default()
        },
    );
}
