extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate stdweb;
extern crate cfg_if;
extern crate quicksilver;
#[macro_use]
extern crate serde_derive;
extern crate specs;
extern crate time;

/*extern crate console_error_panic_hook;*/
//pub use console_error_panic_hook::set_once as set_panic_hook;

mod character;
mod log;
pub mod map;
mod stages;
mod utils;

use character::Character;
use log::log;
use map::{BlockSystem, Stage, StageCreator};
use quicksilver::{
    geom::{Rectangle, Shape, Vector},
    graphics::{
        Animation,
        Background::{self, Img},
        Color, Font, FontStyle, Image,
    },
    input::{ButtonState, Key},
    lifecycle::{run, Asset, Settings as QuickSilverSettings, State, Window},
    Result,
};
use specs::{
    Builder, Component, Dispatcher, DispatcherBuilder, Join, Read, ReadStorage, System, VecStorage,
    World, WriteStorage,
};
use std::time::Duration;

const WINDOW_WIDTH: u16 = 800;
const WINDOW_HEIGHT: u16 = 600;

#[derive(Debug, Deserialize, Serialize)]
pub struct GameState {
    pub current_stage: u16,
    pub current_level: u16,
    pub game_over: bool,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            current_stage: 1,
            current_level: 1,
            game_over: false,
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
    character_animation: Animation,
    settings: Settings,
}

impl Screen {
    fn handle_right_key_for_character(
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
            |position| position.position.x += 2.5,
        );
    }

    fn handle_left_key_for_character(
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
            |position| position.position.x -= 2.5,
        );
    }

    fn handle_down_key_for_character(
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
            |position| position.position.y += 2.5,
        );
    }

    fn handle_up_key_for_character(
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
            |position| position.position.y -= 2.5,
        );
    }

    fn handle_key_for_character<P: FnOnce(&mut Position) -> ()>(
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
                log(&format!("{:?}", current_frame_area));
                match current_frame_area {
                    Rectangle {
                        pos: Vector { x, .. },
                        ..
                    } => if let Some(start_position) = start_position {
                        if let CharacterPosition::Start(rectangle) = start_position {
                            if x == rectangle.pos.x {
                                character_animation.tick();

                                log("Animation ticking...");
                            }
                        }
                    },
                }
            }
            ButtonState::Released => match current_frame_area {
                Rectangle {
                    pos: Vector { x, .. },
                    ..
                } => if let Some(moving_position) = moving_position {
                    if let CharacterPosition::Moving(rectangle) = moving_position {
                        if x == rectangle.pos.x {
                            character_animation.tick();

                            log("Animation ticking for moving...");
                        }
                    }
                },
            },
            _ => {}
        }
    }

    fn load_assets(animation_positions: Vec<Rectangle>, settings: &Settings) -> GameAsset {
        let frame_delay = 1;
        let mut character_animation = None;

        let mali_font = Asset::new(Font::load(settings.mali_font_path.to_owned()));
        let mut character_sprites =
            Asset::new(Image::load(settings.character_sprites_path.to_owned()));

        let _ = character_sprites.execute(|character_image| {
            let animation = Animation::from_spritesheet(
                character_image.to_owned(),
                animation_positions,
                frame_delay,
            );
            character_animation = Some(animation);
            Ok(())
        });

        GameAsset {
            mali_font,
            character_animation,
        }
    }
}

#[derive(Debug)]
struct GameAsset {
    mali_font: Asset<Font>,
    character_animation: Option<Animation>,
}

#[derive(Debug)]
struct Settings {
    animation_positions: Vec<CharacterPosition>,
    mali_font_path: String,
    character_sprites_path: String,
}

#[derive(Debug)]
enum CharacterPosition {
    Start(Rectangle),
    Moving(Rectangle),
}

impl State for Screen {
    fn new() -> Result<Self> {
        let animation_start_position = Rectangle::new(Vector::new(35, 46), Vector::new(29, 22));
        let animation_moving_position = Rectangle::new(Vector::new(69, 46), Vector::new(29, 22));
        let animation_positions = vec![animation_start_position, animation_moving_position];

        let settings = Settings {
            animation_positions: vec![
                CharacterPosition::Start(animation_start_position),
                CharacterPosition::Moving(animation_moving_position),
            ],
            mali_font_path: "mali/Mali-Regular.ttf".to_owned(),
            character_sprites_path: "character_sprites.png".to_owned(),
        };

        let GameAsset {
            mali_font,
            character_animation,
        } = Screen::load_assets(animation_positions, &settings);

        let mut world = World::new();

        world.add_resource(DeltaTime(10.05));
        world.register::<Character>();

        let mut dispatcher: Dispatcher = DispatcherBuilder::new()
            .with(StageCreator, "stage_creator", &[])
            .with(PhysicsSystem, "physics_system", &[])
            .with(BlockSystem, "block_system", &["physics_system"])
            .build();

        dispatcher.setup(&mut world.res);

        dispatcher.dispatch(&world.res);

        world
            .create_entity()
            .with(Velocity { x: 0.1, y: 0.2 })
            .with(Position::new(130., 330.))
            .with(Character::default())
            .build();

        world.maintain();

        let screen = Screen {
            world,
            time_elapsed: Duration::new(0, 0),
            mali_font,
            character_animation: character_animation.expect("Couldn't get Character animation"),
            settings,
        };

        Ok(screen)
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        self.time_elapsed += Duration::from_millis(10);

        let mut game_state = self.world.write_resource::<GameState>();
        let stages = self.world.read_storage::<Stage>();
        let mut positions = self.world.write_storage::<Position>();
        let entities = self.world.entities();

        let character_animation = &mut self.character_animation;
        let animation_positions = &self.settings.animation_positions;
        let time_elapsed = self.time_elapsed;

        entities.join().for_each(move |entity| {
            if let Some(stage) = stages.get(entity) {
                stage
                    .maps
                    .iter()
                    .find(|map| map.level == game_state.current_level)
                    .map(|map| {
                        if time_elapsed.as_secs() >= map.time / 1000 {
                            game_state.game_over = true;
                        }
                    });
            };

            if let Some(position) = positions.get_mut(entity) {
                Screen::handle_right_key_for_character(
                    window,
                    position,
                    character_animation,
                    animation_positions,
                );

                Screen::handle_left_key_for_character(
                    window,
                    position,
                    character_animation,
                    animation_positions,
                );

                Screen::handle_down_key_for_character(
                    window,
                    position,
                    character_animation,
                    animation_positions,
                );

                Screen::handle_up_key_for_character(
                    window,
                    position,
                    character_animation,
                    animation_positions,
                );
            }
        });

        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;

        let entities = self.world.entities();
        let stages = self.world.read_storage::<Stage>();
        let game_state = self.world.read_resource::<GameState>();
        let positions = self.world.read_storage::<Position>();

        let font_style = FontStyle::new(72.0, Color::WHITE);

        let time_elapsed = self.time_elapsed;
        let mali_font = &mut self.mali_font;
        let character_animation = &self.character_animation;

        entities.join().for_each(|entity| {
            if let Some(position) = positions.get(entity) {
                let current_frame = character_animation.current_frame();
                window.draw(
                    &current_frame.area().with_center(position.position),
                    Img(&current_frame),
                );
            }
            if let Some(stage) = stages.get(entity) {
                if !game_state.game_over {
                    stage
                        .maps
                        .iter()
                        .find(|map| map.level == game_state.current_level)
                        .map(|map| {
                            let _ = mali_font.execute(|font| {
                                let _ = font
                                    .render(
                                        &format!("{}", map.time / 1000 - time_elapsed.as_secs()),
                                        &font_style,
                                    ).map(|text| {
                                        window.draw(&text.area().with_center((70, 50)), Img(&text));
                                    });
                                Ok(())
                            });
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
                                });
                        });
                } else {
                    let _ = mali_font.execute(|font| {
                        let _ = font.render("Game over", &font_style).map(|text| {
                            window.draw(
                                &text
                                    .area()
                                    .with_center((WINDOW_WIDTH / 2, WINDOW_HEIGHT / 2)),
                                Img(&text),
                            );
                        });

                        Ok(())
                    });
                }
            };
        });

        Ok(())
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
