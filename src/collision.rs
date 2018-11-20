use super::{Position, World as EntitiesWorld};
use character::Character;
use nalgebra::{Isometry2, Vector2};
use ncollide2d::events::ProximityEvent;
use ncollide2d::query::Proximity;
use ncollide2d::shape::{Ball, Cuboid, ShapeHandle};
use ncollide2d::world::{
    CollisionGroups, CollisionObject, CollisionObjectHandle, CollisionWorld, GeometricQueryType,
};
use specs::{
    Component, Entities, Join, LazyUpdate, Read, ReadStorage, System, VecStorage, WriteStorage,
};
use std::cell::Cell;

impl Component for CollisionObjectData {
    type Storage = VecStorage<Self>;
}

pub struct CollisionSystem;

impl<'a> System<'a> for CollisionSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, CollisionObjectData>,
        //ReadStorage<'a, Position>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (entities, mut collision_objects, updater): Self::SystemData) {
        /*let e = entities.create();*/

        (&entities, &mut collision_objects)
            .join()
            .for_each(|(entity, collision_object)| {
                //updater.insert(collision_object, position.clone());
            });
    }
}

type MatrixPosition = nalgebra::MatrixArray<f32, nalgebra::U2, nalgebra::U1>;

#[derive(Clone, Debug)]
pub struct CollisionObjectData {
    pub name: &'static str,
    pub velocity: Option<Vector2<f32>>,
    pub position: Option<MatrixPosition>,
}

impl CollisionObjectData {
    pub fn new(
        name: &'static str,
        velocity: Option<Vector2<f32>>,
        position: Option<MatrixPosition>,
    ) -> CollisionObjectData {
        let init_velocity = if let Some(velocity) = velocity {
            Some(velocity)
        } else {
            None
        };

        CollisionObjectData {
            name: name,
            velocity: init_velocity,
            position,
        }
    }
}

// TODO: Better return type.
fn handle_proximity_event<'a, 'b>(
    collision_world: &'a CollisionWorld<f32, CollisionObjectData>,
    event: &'a ProximityEvent,
    entities_world: &'b EntitiesWorld,
) -> (
    ProximityEvent,
    CollisionObject<f32, CollisionObjectData>,
    String,
    /*specs::Storage<*/
    //'b,
    //CollisionObjectData,
    //specs::shred::FetchMut<'b, specs::storage::MaskedStorage<CollisionObjectData>>,
    /*>,*/
) {
    //let mut collision_storage = entities_world.write_storage::<CollisionObjectData>();
    //let entities = entities_world.entities();

    let co1 = collision_world.collision_object(event.collider1).unwrap();
    let co2 = collision_world.collision_object(event.collider2).unwrap();
    // TODO: This shouldn't be needed to do. A reference should be able to return.
    let co3 = CollisionObject::new(
        co1.handle(),
        co1.proxy_handle(),
        co1.position().clone(),
        co1.shape().clone(),
        co1.collision_groups().clone(),
        co1.query_type(),
        co1.data().clone(),
    );

    let area_name = if co1.data().velocity.is_none() {
        co1.data().name
    } else {
        co2.data().name
    };

    if event.new_status == Proximity::Intersecting {
        println!("Collision detected for area {}", area_name);
    } else if event.new_status == Proximity::Disjoint {
        println!("No longer colliding for area {}", area_name);
    }

    (
        *event,
        co3,
        area_name.to_string(), /*, collision_storage*/
    )
}

pub struct Collision {
    world: CollisionWorld<f32, CollisionObjectData>,
    character_handle: CollisionObjectHandle,
    character_position: Isometry2<f32>,
}

impl Collision {
    pub fn new(entities_world: &EntitiesWorld) -> Self {
        let characters = entities_world.read_storage::<Character>();
        let positions = entities_world.read_storage::<Position>();

        let isometry_positions = entities_world
            .entities()
            .join()
            .map(|entity| {
                positions.get(entity).and_then(|position| {
                    // TODO: Refactor the Some/None part.
                    if let None = characters.get(entity) {
                        let position = position.position;
                        println!("BLOCK POS {:?}", position);
                        Some(Isometry2::new(
                            Vector2::new(position.x, position.y),
                            nalgebra::zero(),
                        ))
                    } else {
                        None
                    }
                })
            }).filter(|position| position.is_some())
            .map(|position| position.unwrap())
            .collect::<Vec<Isometry2<_>>>();

        let character_position = entities_world
            .entities()
            .join()
            .map(|entity| characters.get(entity).and_then(|_| positions.get(entity)))
            .filter(|character| character.is_some())
            .map(|character_position| {
                let position = character_position.unwrap().position;
                //println!("Character pos {:?}", position);
                Isometry2::new(Vector2::new(position.x, position.y), nalgebra::zero())
            }).collect::<Vec<Isometry2<_>>>();

        // The character is part of group 1 and can interact with everything.
        let mut character_groups = CollisionGroups::new();
        character_groups.set_membership(&[1]);

        // All the other objects are part of the group 2 and interact only with the character (but not with
        // each other).
        let mut others_groups = CollisionGroups::new();
        others_groups.set_membership(&[2]);
        others_groups.set_whitelist(&[1]);

        let rect_data_purple = CollisionObjectData::new("purple", None, None);
        let character_data =
            CollisionObjectData::new("character", Some(Vector2::new(12.0, 12.0)), None);

        // Collision world 0.02 optimization margin and small object identifiers.
        let mut collision_world = CollisionWorld::new(0.);

        let contacts_query = GeometricQueryType::Contacts(0.0, 0.0);
        let proximity_query = GeometricQueryType::Proximity(0.0);
        let rect = ShapeHandle::new(Cuboid::new(Vector2::new(25.0f32, 25.0)));

        /*isometry_positions.iter().for_each(|position| {*/
        //collision_world.add(
        //*position,
        //rect.clone(),
        //others_groups,
        //proximity_query,
        //rect_data_purple.clone(),
        //);
        //});

        println!("POSS {:?}", isometry_positions);

        collision_world.add(
            isometry_positions[0],
            rect.clone(),
            others_groups,
            proximity_query,
            rect_data_purple,
        );
        /*collision_world.add(*/
        //isometry_positions[1],
        //rect.clone(),
        //others_groups,
        //proximity_query,
        //rect_data_blue,
        //);
        //collision_world.add(
        //isometry_positions[2],
        //rect.clone(),
        //others_groups,
        //proximity_query,
        //rect_data_green,
        //);
        //collision_world.add(
        //isometry_positions[3],
        //rect.clone(),
        //others_groups,
        //proximity_query,
        //rect_data_yellow,
        /*);*/

        let character = ShapeHandle::new(Ball::new(11.5f32));

        let character_position: Isometry2<f32> = character_position[0];

        let character_handle = collision_world.add(
            character_position,
            character,
            character_groups,
            contacts_query,
            character_data,
        );

        Collision {
            world: collision_world,
            character_handle,
            character_position,
        }
    }

    pub fn set_character_position(&mut self, position: &Position) -> &mut Self {
        let position = position.position;
        self.character_position =
            Isometry2::new(Vector2::new(position.x, position.y), nalgebra::zero());
        self
    }
    /*Vec<*/
    //specs::Storage<
    //'b,
    //CollisionObjectData,
    //specs::shred::FetchMut<'b, specs::storage::MaskedStorage<CollisionObjectData>>,
    //>,
    /*>*/

    // TODO: Better return type, what is really needed here?
    // Also, put it in a struct.
    pub fn update<'a, 'b>(
        collision: &'a mut Self,
        entities_world: &'b EntitiesWorld,
    ) -> Vec<(
        ProximityEvent,
        CollisionObject<f32, CollisionObjectData>,
        String,
    )> {
        let Collision {
            world: collision_world,
            character_handle,
            character_position,
        } = collision;

        // Poll and handle events.
        let events = collision_world
            .proximity_events()
            .iter()
            .map(|event| handle_proximity_event(collision_world, event, entities_world))
            .collect::<Vec<_>>();

        // Submit the position update to the world.
        collision_world.set_position(*character_handle, *character_position);

        collision_world.update();

        events
    }
}
