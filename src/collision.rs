use super::{Position, World as EntitiesWorld};
use character::Character;
use nalgebra::{Isometry2, Vector2};
use ncollide2d::events::ProximityEvent;
use ncollide2d::query::Proximity;
use ncollide2d::shape::{Ball, Cuboid, ShapeHandle};
use ncollide2d::world::{
    CollisionGroups, CollisionObjectHandle, CollisionWorld, GeometricQueryType,
};
use specs::Join;
use std::cell::Cell;

#[derive(Clone)]
struct CollisionObjectData {
    pub name: &'static str,
    pub velocity: Option<Cell<Vector2<f32>>>,
}

impl CollisionObjectData {
    pub fn new(name: &'static str, velocity: Option<Vector2<f32>>) -> CollisionObjectData {
        let init_velocity = if let Some(velocity) = velocity {
            Some(Cell::new(velocity))
        } else {
            None
        };

        CollisionObjectData {
            name: name,
            velocity: init_velocity,
        }
    }
}

fn handle_proximity_event(
    world: &CollisionWorld<f32, CollisionObjectData>,
    event: &ProximityEvent,
) {
    let co1 = world.collision_object(event.collider1).unwrap();
    let co2 = world.collision_object(event.collider2).unwrap();

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
}

pub struct Collision {
    world: CollisionWorld<f32, CollisionObjectData>,
    character_handle: CollisionObjectHandle,
    character_position: Isometry2<f32>,
}

impl Collision {
    pub fn new(entities_world: &mut EntitiesWorld) -> Self {
        let characters = entities_world.read_storage::<Character>();
        let positions = entities_world.read_storage::<Position>();

        let isometry_positions = entities_world
            .entities()
            .join()
            .map(|entity| {
                positions.get(entity).map(|position| {
                    let position = position.position;
                    Isometry2::new(Vector2::new(position.x, position.y), nalgebra::zero())
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
                println!("Character pos {:?}", position);
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

        let rect_data_purple = CollisionObjectData::new("purple", None);
        let rect_data_blue = CollisionObjectData::new("blue", None);
        let rect_data_green = CollisionObjectData::new("green", None);
        let rect_data_yellow = CollisionObjectData::new("yellow", None);
        let character_data = CollisionObjectData::new("character", Some(Vector2::new(32.0, 5.0)));

        // Collision world 0.02 optimization margin and small object identifiers.
        let mut collision_world = CollisionWorld::new(0.02);

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
        /*});*/

        collision_world.add(
            isometry_positions[0],
            rect.clone(),
            others_groups,
            proximity_query,
            rect_data_purple,
        );
        collision_world.add(
            isometry_positions[1],
            rect.clone(),
            others_groups,
            proximity_query,
            rect_data_blue,
        );
        collision_world.add(
            isometry_positions[2],
            rect.clone(),
            others_groups,
            proximity_query,
            rect_data_green,
        );
        collision_world.add(
            isometry_positions[3],
            rect.clone(),
            others_groups,
            proximity_query,
            rect_data_yellow,
        );

        let character = ShapeHandle::new(Ball::new(0.5f32));

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

    pub fn update(collision: &mut Self) {
        let Collision {
            world: collision_world,
            character_handle,
            character_position,
        } = collision;

        // Poll and handle events.
        for event in collision_world.proximity_events() {
            handle_proximity_event(&collision_world, event)
        }

        // Submit the position update to the world.
        collision_world.set_position(*character_handle, *character_position);

        collision_world.update();
    }
}
