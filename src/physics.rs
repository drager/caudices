use character::Character;
use collision::{Collision, CollisionObjectData};
use nalgebra::{Isometry2, Vector2};
use ncollide2d::{
    events::ContactEvent,
    shape::{Cuboid, ShapeHandle},
    world::{
        CollisionGroups, CollisionObject, CollisionObjectHandle, CollisionWorld, GeometricQueryType,
    },
};
use specs::{
    Component, Entities, Join, LazyUpdate, Read, ReadStorage, System, VecStorage, Write,
    WriteStorage,
};
use Settings;

pub type CollisionNormal = nalgebra::Unit<
    nalgebra::Matrix<
        f32,
        nalgebra::U2,
        nalgebra::U1,
        nalgebra::ArrayStorage<f32, nalgebra::U2, nalgebra::U1>,
    >,
>;

pub struct ContactData {
    pub contact_event: ContactEvent,
    pub collision_normals: Vec<CollisionNormal>,
}

#[derive(Debug, PartialEq)]
pub enum MovingState {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Default, Debug)]
pub struct DeltaTime(pub f32);

impl PhysicsSystem {
    pub fn init_collision_world<'a>() -> Collision {
        let collision_world = CollisionWorld::new(0.);

        Collision {
            world: Some(collision_world),
            character_handle: None,
            character_position: None,
        }
    }

    pub fn handle_contact_event<'a, 'b>(
        collision_world: &'a CollisionWorld<f32, CollisionObjectData>,
        event: &'a ContactEvent,
    ) -> ContactData {
        let get_collision_object = |collision_object_handle: &CollisionObjectHandle| -> Option<&CollisionObject<f32, CollisionObjectData>> {
            collision_world.collision_object(*collision_object_handle)
        };

        let (character_collision_object, second_collision_object) = match event {
            ContactEvent::Started(character_collision_object, second_collision_object) => (
                get_collision_object(character_collision_object),
                get_collision_object(second_collision_object),
            ),
            ContactEvent::Stopped(character_collision_object, second_collision_object) => (
                get_collision_object(character_collision_object),
                get_collision_object(second_collision_object),
            ),
        };

        let collision_normals = character_collision_object
            .and_then(|character_collision_object| {
                second_collision_object.map(|second_collision_object| {
                    let mut collector = vec![];
                    collision_world
                        .contact_pair(
                            character_collision_object.handle(),
                            second_collision_object.handle(),
                        )
                        .map(|contact_manifold_generator| {
                            contact_manifold_generator.contacts(&mut collector)
                        });

                    collector
                        .iter()
                        .map(|contact_manifold| {
                            let deepest_contact = contact_manifold.deepest_contact().unwrap();
                            let contact_normal = character_collision_object.position().inverse()
                                * deepest_contact.contact.normal;
                            let feature_1 = deepest_contact.kinematic.feature1();

                            println!("Normal {:?}", contact_normal);
                            println!("Feature 1 {:?}", feature_1);
                            println!("depth {:?}", deepest_contact.contact.depth);
                            println!("Feature 2 {:?}", deepest_contact.kinematic.feature2());
                            println!("pos in handle {:?}", character_collision_object.position());

                            let _co1_pos = character_collision_object.position().translation.vector;
                            let _co2_pos = second_collision_object.position().translation.vector;
                            println!("Character position: {:?}", _co1_pos);
                            println!("Box position: {:?}", _co2_pos);

                            contact_normal
                        })
                        .collect::<Vec<CollisionNormal>>()
                })
            })
            .unwrap_or_else(|| vec![]);

        ContactData {
            contact_event: *event,
            collision_normals,
        }
    }

    pub fn setup_handles<'a>(
        settings: &Settings,
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
                    println!("BLOCK POS {:?}", position);
                    Some(Isometry2::new(position.0, nalgebra::zero()))
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

        let rect_data = CollisionObjectData::new("rect", None);
        let character_data = CollisionObjectData::new("character", Some(Vector2::new(32.0, 12.0)));

        let margin = 2.0;
        let rect_half_extent = settings.block_size.x - margin;
        let character_half_extent_width = settings.character_size.x - margin;
        let character_half_extent_height = settings.character_size.y - margin;

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
                    rect_data.clone(),
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
    ) -> Vec<ContactData> {
        let collision = collision_world.set_character_position(position);
        let events = collision
            .character_handle
            .and_then(move |character_handle| {
                collision.character_position.map(|character_position| {
                    if let Some(ref mut world) = collision.world {
                        world.set_position(character_handle, character_position);

                        // Poll and handle events.
                        let events = world
                            .contact_events()
                            .iter()
                            .map(|event| Self::handle_contact_event(&world, event))
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
}

pub struct PhysicsSystem;

impl Component for CollisionHandle {
    type Storage = VecStorage<Self>;
}

pub struct CollisionHandle {
    pub collision_data: ContactData,
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
            _,
            updater,
        ) = data;

        (&entities, &velocity_storage, &mut position_storage)
            .join()
            .for_each(|(entity, velocity, position)| {
                position.0.x += velocity.0.x * delta.0;
                position.0.y += velocity.0.y * delta.0;

                if let Some(_character) = character_storage.get(entity) {
                    let collision_events = Self::update_collision(position, &mut collision_world);

                    collision_events.into_iter().for_each(|event| {
                        match event.contact_event {
                            ContactEvent::Started(_, _) => {
                                println!("Velocity: {:?}", velocity);
                                println!("Delta: {:?}", delta.0);
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
                    });
                }
            });
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Position(pub Vector2<f32>);

impl Component for Position {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
pub struct Velocity(pub Vector2<f32>);

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}
