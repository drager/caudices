use character::Character;
use collision::{self, Collision, CollisionObjectData};
use nalgebra::base::Matrix;
use nalgebra::{Isometry2, Vector2};
use ncollide2d::events::{ContactEvent, ProximityEvent};
use ncollide2d::query::Proximity;
use ncollide2d::shape::{Ball, Compound, Cuboid, ShapeHandle};
use ncollide2d::world::{
    CollisionGroups, CollisionObject, CollisionObjectHandle, CollisionWorld, GeometricQueryType,
};
use quicksilver::geom::Vector;
use specs::{
    Builder, Component, Entities, Join, LazyUpdate, Read, ReadStorage, Resources, System,
    VecStorage, Write, WriteStorage,
};

pub type CollisionNormal = nalgebra::Unit<
    nalgebra::Matrix<
        f32,
        nalgebra::U2,
        nalgebra::U1,
        nalgebra::MatrixArray<f32, nalgebra::U2, nalgebra::U1>,
    >,
>;

pub struct ProximityData {
    pub proximity_event: ContactEvent,
    pub collision_object: CollisionObject<f32, CollisionObjectData>,
    pub collision_normals: Vec<(CollisionNormal, f32)>,
}

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
        let collision_world = CollisionWorld::new(0.);

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

    pub fn handle_proximity_event<'a, 'b>(
        collision_world: &'a CollisionWorld<f32, CollisionObjectData>,
        event: &'a ContactEvent,
    ) -> ProximityData {
        //println!("handle proximity");

        let (co1, co2) = match event {
            ContactEvent::Started(co1, co2) => (
                collision_world.collision_object(*co1).unwrap(),
                collision_world.collision_object(*co2).unwrap(),
            ),
            ContactEvent::Stopped(co1, co2) => (
                collision_world.collision_object(*co1).unwrap(),
                collision_world.collision_object(*co2).unwrap(),
            ),
        };
        //let co1 = collision_world.collision_object(event.collider1).unwrap();
        //let co2 = collision_world.collision_object(event.collider2).unwrap();

        // TODO: This shouldn't be needed to do. A reference should be able to return.
        let co3 = CollisionObject::new(
            co2.handle(),
            co2.proxy_handle(),
            co2.position().clone(),
            co2.shape().clone(),
            co2.collision_groups().clone(),
            co2.query_type(),
            co2.data().clone(),
        );

        let mut collector = vec![];
        collision_world
            .contact_pair(co1.handle(), co3.handle())
            .map(|a| a.contacts(&mut collector));

        let collision_normals = collector
            .iter()
            .map(|c| {
                //println!("COLLECTOR {:?}", c.deepest_contact());
                let deepest_contact = c.deepest_contact().unwrap();
                let local_space = co1.position().inverse() * deepest_contact.contact.normal;
                let f1 = deepest_contact.kinematic.feature1();
                println!("Normal {:?}", local_space);
                println!("Feature 1 {:?}", f1);
                println!("depth {:?}", deepest_contact.contact.depth);
                println!("Feature 2 {:?}", deepest_contact.kinematic.feature2());
                println!("pos in handle {:?}", co1.position());
                let co1_pos = co1.position().translation.vector;
                let co2_pos = co2.position().translation.vector;

                let surface_area =
                    (co1_pos.x + 25.).min(co2_pos.x + 7.) - (co1_pos.x).max(co2_pos.x);

                let surface_area2 =
                    (co1_pos.y + 25.).min(co2_pos.y + 7.) - (co1_pos.y).max(co2_pos.y);

                /*println!("Surface {:?}", surface_area);*/
                //println!("Surface 2 {:?}", surface_area2);

                (local_space, surface_area2)
            })
            .collect::<Vec<(CollisionNormal, f32)>>();

        ProximityData {
            proximity_event: *event,
            collision_object: co3,
            collision_normals,
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

        // TODO: We should get the size from settings somewhere and then just divide it by 2.
        let margin = 2.0;
        let rect_half_extent = 25.0 - margin;
        let character_half_extent_width = 25.0 - margin;
        let character_half_extent_height = 25.0 - margin;

        let contacts_query = GeometricQueryType::Contacts(margin, 0.);
        let rect = ShapeHandle::new(Cuboid::new(Vector2::new(
            rect_half_extent,
            rect_half_extent,
        )));

        // TODO: When Capsule implements Shape we should use it instead of a Cuboid.
        // https://github.com/rustsim/ncollide/issues/175
        let character = ShapeHandle::new(Cuboid::new(Vector2::new(
            character_half_extent_width,
            character_half_extent_height,
        )));
        /*        let character = ShapeHandle::new(Compound::new(vec![*/
        //(
        //Isometry2::new(Vector2::new(32.0, 12.0), nalgebra::zero()),
        //ShapeHandle::new(Ball::new(0.5f32)),
        //),
        //(
        //Isometry2::new(Vector2::new(32.0, 12.0), nalgebra::zero()),
        //ShapeHandle::new(Ball::new(0.5f32)),
        //),
        /*]));*/

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
                    contacts_query,
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

    pub fn has_changed<'a>(proximity_event: ContactEvent, collision_world: &mut Collision) -> bool {
        if let Some(ref world) = collision_world.world {
            let co2 = match proximity_event {
                ContactEvent::Started(_, co2) => co2,
                ContactEvent::Stopped(_, co2) => co2,
            };
            world
                .contact_events()
                .iter()
                .find(|event| match event {
                    ContactEvent::Started(_, old_co2) => old_co2.0 == co2.0,
                    ContactEvent::Stopped(_, old_co2) => old_co2.0 == co2.0,
                })
                .map(|old_event| {
                    println!("CHANGED {:?}", old_event);
                    match old_event {
                        ContactEvent::Started(_, _) => false,
                        ContactEvent::Stopped(_, _) => true,
                    }
                })
                .unwrap_or_else(|| false)
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
                collision.character_position.map(|character_position| {
                    if let Some(ref mut world) = collision.world {
                        world.set_position(character_handle, character_position);
                        //println!("LEN {:?}", world.proximity_events().iter().len());
                        // Poll and handle events.
                        let events = world
                            .contact_events()
                            .iter()
                            .map(|event| Self::handle_proximity_event(&world, event))
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
        //let half_size = 35.;

        //if let Some(collision_pos) = collision.position {
        let collision_x = block_position.x;
        let collision_y = block_position.y;
        /*        println!("char X {:?}", character_x);*/
        //println!("char Y {:?}", character_y);
        //println!("Col X {:?}", collision_x);
        /*println!("Col Y {:?}", collision_y);*/

        let object_width = block_position.x + 25.;
        let object_height = block_position.y + 25.;
        let character_width = 12.;
        let character_height = 12.;

        if character_x <= object_width && character_y >= object_height {
            //println!("BOTTOM");
            vec![MovingState::Right, MovingState::Left, MovingState::Bottom]
        } else if character_x >= object_width && character_y <= object_height {
            //println!("TOP");
            vec![MovingState::Right, MovingState::Left, MovingState::Top]
        } else if character_x <= object_width && character_y <= object_height {
            //println!("LEFT");
            vec![MovingState::Left, MovingState::Top, MovingState::Bottom]
        } else if character_x >= object_width && character_y >= object_height {
            //println!("RIGHT");
            vec![MovingState::Right, MovingState::Top, MovingState::Bottom]
        } else {
            vec![]
        }
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
                        match event.proximity_event {
                            ContactEvent::Started(_, _) => {
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
                            _ => {}
                        };
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
