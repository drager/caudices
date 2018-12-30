use character::Character;
use collision::{self, Collision, CollisionObjectData, ProximityData};
use nalgebra::base::Matrix;
use nalgebra::{Isometry2, Vector2};
use ncollide2d::events::ProximityEvent;
use ncollide2d::query::Proximity;
use ncollide2d::shape::{Cuboid, ShapeHandle, Ball};
use ncollide2d::world::{
    CollisionGroups, CollisionObject, CollisionObjectHandle, CollisionWorld, GeometricQueryType,
};
use quicksilver::geom::Vector;
use specs::{
    Builder, Component, Entities, Join, LazyUpdate, Read, ReadStorage, Resources, System,
    VecStorage, Write, WriteStorage,
};

#[derive(Debug, PartialEq)]
pub enum MovingState {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Default)]
pub struct DeltaTime(pub f32);

impl PhysicsSystem {
    pub fn init_collision_world<'a>() -> Collision {
        // Collision world 0.02 optimization margin and small object identifiers.
        let collision_world = CollisionWorld::new(0.02);

        //let (character_position, character_handle) = Self::setup_handles(
        //&mut collision_world,
        //velocity_storage,
        //position_storage,
        //character_storage,
        /*);*/

        Collision {
            world: Some(collision_world),
            character_handle: None,
            character_position: None,
        }
    }

    pub fn setup_handles<'a>(
        entities: &Entities<'a>,
        collision: &'a mut Collision,
        velocity_storage: &ReadStorage<'a, Velocity>,
        position_storage: &ReadStorage<'a, Position>,
        character_storage: &ReadStorage<'a, Character>,
    ) {
        let isometry_positions = (entities, position_storage)
            .join()
            .map(|(entity, position)| {
                if let None = character_storage.get(entity) {
                    let position = position.0;
                    println!("BLOCK POS {:?}", position);
                    Some(Isometry2::new(
                        Vector2::new(position.x, position.y),
                        nalgebra::zero(),
                    ))
                } else {
                    None
                }
            })
            .filter_map(|x| x)
            .collect::<Vec<Isometry2<_>>>();

        let character_position = (velocity_storage, character_storage, position_storage)
            .join()
            .map(|(_velocity, _character, position)| {
                let character_position = position.0;
                //println!("Character pos {:?}", position);
                Isometry2::new(
                    Vector2::new(character_position.x, character_position.y),
                    nalgebra::zero(),
                )
            })
            .collect::<Vec<Isometry2<_>>>();

        // The character is part of group 1 and can interact with everything.
        let mut character_groups = CollisionGroups::new();
        character_groups.set_membership(&[1]);

        // All the other objects are part of the group 2 and interact only with the character (but not with
        // each other).
        let mut others_groups = CollisionGroups::new();
        others_groups.set_membership(&[2]);
        others_groups.set_whitelist(&[1]);

        let rect_data_purple = CollisionObjectData::new("purple", None, None, None);
        let character_data =
            CollisionObjectData::new("character", Some(Vector2::new(32.0, 12.0)), None, None);

        let contacts_query = GeometricQueryType::Contacts(0.0, 0.0);
        let proximity_query = GeometricQueryType::Proximity(0.0);
        let rect = ShapeHandle::new(Cuboid::new(Vector2::new(25.0f32, 25.0)));

        // TODO: When Capsule implements Shape we should use it instead of a Cuboid.
        // https://github.com/rustsim/ncollide/issues/175
        let character = ShapeHandle::new(Cuboid::new(Vector2::new(17.0, 11.0)));
//let capsule = ShapeHandle::new(Ball::new(0.5f32), Ball::new(0.5f32));

        let character_position: Option<Isometry2<f32>> =
            character_position.get(0).map(|opt| opt.to_owned());

        let character_handle = if let Some(ref mut world) = collision.world {
            let handle = character_position.map(|character_position| {
                println!("Char pos {:?}", character_position);
                world.add(
                    character_position,
                    character,
                    character_groups,
                    contacts_query,
                    character_data,
                )
            });

            isometry_positions.iter().for_each(|position| {
                println!("iso pos {:?}", position);
                world.add(
                    *position,
                    rect.clone(),
                    others_groups,
                    proximity_query,
                    rect_data_purple.clone(),
                );
            });
            handle
        } else {
            None
        };

        collision.character_position = character_position;
        collision.character_handle = character_handle;
    }

    pub fn has_changed<'a>(
        proximity_event: ProximityEvent,
        collision_world: &mut Collision,
    ) -> bool {
        if let Some(ref w) = collision_world.world {
            let old_event = w
                .proximity_events()
                .iter()
                .find(|event| event.collider2.0 == proximity_event.collider2.0);
            if old_event.is_some() && proximity_event.new_status != old_event.unwrap().new_status {
                println!("CHANGED {:?}", old_event);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn update_collision<'a, 'b>(
        position: &Position,
        collision_world: &mut Collision,
    ) -> Vec<ProximityData> {
        let collision = collision_world.set_character_position(position);
        let events = collision
            .character_handle
            .and_then(move |character_handle| {
                //println!("Handle");
                collision.character_position.map(|character_position| {
                    if let Some(ref mut world) = collision.world {
                        world.set_position(character_handle, character_position);
                        //println!("LEN {:?}", world.proximity_events().iter().len());
                        // Poll and handle events.
                        let events = world
                            .proximity_events()
                            .iter()
                            .map(|event| collision::handle_proximity_event(&world, event))
                            .collect::<Vec<_>>();

                        // Submit the position update to the world.
                        world.update();
                        events
                    } else {
                        vec![]
                    }
                })
            })
            .unwrap_or_else(|| vec![]);

        events
    }

    pub fn get_start_moving_state() -> Vec<MovingState> {
        vec![
            MovingState::Left,
            MovingState::Right,
            MovingState::Top,
            MovingState::Bottom,
        ]
    }

    pub fn calculate_moving_state(
        block_position: &Matrix<
            f32,
            nalgebra::U2,
            nalgebra::U1,
            nalgebra::MatrixArray<f32, nalgebra::U2, nalgebra::U1>,
        >,
        character_position: &Position,
    ) -> Vec<MovingState> {
        //if let Some(position) = collision.character_position {
        let character_x = character_position.0.x;
        let character_y = character_position.0.y;
        let half_size = 35.;

        //if let Some(collision_pos) = collision.position {
        let collision_x = block_position.x;
        let collision_y = block_position.y;
/*        println!("char X {:?}", character_x);*/
        //println!("char Y {:?}", character_y);
        //println!("Col X {:?}", collision_x);
        /*println!("Col Y {:?}", collision_y);*/

        if (character_x - half_size) <= collision_x && (character_y - half_size) >= collision_y {
            println!("BOTTOM");
            vec![MovingState::Right, MovingState::Left, MovingState::Bottom]
        } else if (character_x + half_size) >= collision_x
            && (character_y + half_size) <= collision_y
        {
            println!("TOP");
            vec![MovingState::Right, MovingState::Left, MovingState::Top]
        } else if (character_x + half_size) <= collision_x
            && (character_y - half_size) <= collision_y
        {
            println!("LEFT");
            vec![MovingState::Left, MovingState::Top, MovingState::Bottom]
        }
        // Right side
        else if (character_x + half_size) >= collision_x
            && (character_y + half_size) >= collision_y
        {
            println!("RIGHT");
            vec![MovingState::Right, MovingState::Top, MovingState::Bottom]
        } else {
            vec![]
            //Self::get_start_moving_state()
        }
        /*} else {*/
        //Self::get_start_moving_state()
        /*}*/
        /*        } else {*/
        //Self::get_start_moving_state()
        /*}*/
    }
}

pub struct PhysicsSystem {
    pub collision_world: Option<Collision>,
    pub x: bool,
}

impl Component for CollisionHandle {
    type Storage = VecStorage<Self>;
}

//#[derive(Debug)]
//pub struct CollisionHandle(CollisionObjectHandle);
pub struct CollisionHandle {
    pub collision_data: ProximityData,
    pub character_entity: specs::Entity,
}

impl<'a> System<'a> for PhysicsSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, DeltaTime>,
        Write<'a, Collision>,
        ReadStorage<'a, Velocity>,
        ReadStorage<'a, Character>,
        WriteStorage<'a, Position>,
        //WriteStorage<'a, CollisionObjectData>,
        WriteStorage<'a, CollisionHandle>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            delta,
            mut collision_world,
            velocity_storage,
            character_storage,
            mut position_storage,
            mut collision_object_data_storage,
            updater,
        ) = data;
        //println!("delta {:?}", delta.0);

        //let delta = delta.0;

        //println!("Got physics world? {:?}", self.collision_world.is_some());

        (&entities, &velocity_storage, &mut position_storage)
            .join()
            .for_each(|(entity, velocity, position)| {
                //println!("Character phsycis {:?}", position);
                /*if let Some(_) = collision_object_data_storage.get(entity) {*/
                //println!("Got collision");
                /*} else {*/
                position.0.x += velocity.x * delta.0;
                position.0.y += velocity.y * delta.0;
                //}
                if let Some(character) = character_storage.get(entity) {
                    let collision_events = Self::update_collision(position, &mut collision_world);
                    collision_events.into_iter().for_each(|event| {
                        match event.proximity_event.new_status {
                            Proximity::Intersecting => {
                                /*                                updater*/
                                //.create_entity(&entities)
                                ////.with(character)
                                ////.with(*velocity)
                                //.with(CollisionHandle {
                                //collision_data: event,
                                //character_entity: entity,
                                //})
                                /*.build();*/

                                //let collision_entity = entities.create();
                                //println!("Event {:?}", event.1);
                                //

                                //let cid = event.proximity_event.collider2;
                                //println!("CID {:?}", cid);
                                let collision_entity = entities.create();
                                println!("Creating collision object");
                                updater.insert(
                                    collision_entity,
                                    CollisionHandle {
                                        collision_data: event,
                                        character_entity: entity,
                                    },
                                );
                            }
                            /*Proximity::Disjoint => {*/
                            //println!("Disjoint");
                            //updater.insert(
                            //collision_entity,
                            //CollisionHandle {
                            //collision_data: event,
                            //character_entity: entity,
                            //},
                            //);
                            ////updater.remove::<CollisionHandle>(collision_entity);
                            //[>updater.insert(<]
                            ////collision_entity,
                            ////CollisionHandle {
                            ////collision_data: event,
                            ////character_entity: entity,
                            ////},
                            //[>);<]
                            ////None
                            /*}*/
                            _ => {}
                        };
                        //if let Some(obj) = obj {

                        //}
                        //println!("Got event {:?}", event.0);
                    });
                }
            });
    }

    //fn setup(&mut self, res: &mut Resources) {
    //use specs::prelude::SystemData;

    //Self::SystemData::setup(res);
    //let mut r: Write<Collision> = Write::fetch(&*res);

    //let velo: ReadStorage<Velocity> = ReadStorage::fetch(&*res);
    //let pos: ReadStorage<Position> = ReadStorage::fetch(&*res);
    //let character: ReadStorage<Character> = ReadStorage::fetch(&*res);
    //if let Some(ref mut c_world) = r.world {
    //PhysicsSystem::setup_handles(c_world, &velo, &pos, &character);
    //}
    /*}*/

    //fn setup(&mut self, res: &mut specs::Resources) {
    //println!("SELF {}", self.);
    //if let None = self.collision_world {
    //let world =
    //Self::init_collisions(&velocity_storage, &position_storage, &character_storage);
    //self.collision_world = Some(world);
    //}
    /*}*/

    //fn setup(&mut self, res: &mut Resources) {
    //use specs::prelude::SystemData;
    //Self::SystemData::setup(res);
    //let velo: ReadStorage<Velocity> = ReadStorage::fetch(&*res);
    //let pos: ReadStorage<Position> = ReadStorage::fetch(&*res);
    //let character: ReadStorage<Character> = ReadStorage::fetch(&*res);
    //let world = self.init_collision_world();
    //self.collision_world = Some(world);
    /*}*/
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Position(pub Vector);

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Position(Vector::new(x, y))
    }
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}
