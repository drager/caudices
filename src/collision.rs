use super::{Position};
use character::Character;
use nalgebra::{Isometry2, Vector2};
use ncollide2d::world::{
    CollisionObjectHandle, CollisionWorld,
};
use physics::{CollisionHandle, CollisionNormal, MovingState, PhysicsSystem, Velocity};
use specs::{
    Component, Entities, Join, LazyUpdate, Read, ReadStorage, System, VecStorage, Write,
    WriteStorage,
};

impl Component for CollisionObjectData {
    type Storage = VecStorage<Self>;
}

pub fn moving_state_from_collision_normal(
    collision_normal: &CollisionNormal,
) -> Option<MovingState> {
    if collision_normal.x.abs() > collision_normal.y.abs() {
        if collision_normal.x > 0. {
            Some(MovingState::Right)
        } else {
            Some(MovingState::Left)
        }
    } else {
        //println!("Normal {:?}", collision_normal);
        if collision_normal.y > 0. {
            Some(MovingState::Bottom)
        } else {
            Some(MovingState::Top)
        }
    }
}

pub fn change_velocity(moving_state: &MovingState, velocity: &mut Velocity) {
    match moving_state {
        MovingState::Left => velocity.x = velocity.x.max(0.),
        MovingState::Right => velocity.x = velocity.x.min(0.),
        MovingState::Top => velocity.y = velocity.y.max(0.),
        MovingState::Bottom => velocity.y = velocity.y.min(0.),
    }
}

pub struct CollisionSystem;

impl<'a> System<'a> for CollisionSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, CollisionHandle>,
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, Character>,
        ReadStorage<'a, Position>,
        Write<'a, Collision>,
        Read<'a, LazyUpdate>,
    );

    fn run(
        &mut self,
        (
            entities,
            mut collision_objects,
            mut velocity_storage,
            _character_storage,
            position_storage,
            mut collision_world,
            _updater,
        ): Self::SystemData,
    ) {
        let joined_entities = (&entities, &mut collision_objects)
            .join()
            .map(|(entity, collision_data)| {
                let character_entity = collision_data.character_entity;
                if let Some(_character_position) = position_storage.get(character_entity) {
                    if let Some(ref mut velocity) = velocity_storage.get_mut(character_entity) {
                        let _block_position = collision_data
                            .collision_data
                            .collision_object
                            .position()
                            .translation
                            .vector;

                        let collision_normals = &collision_data.collision_data.collision_normals;

                        collision_normals
                            .iter()
                            .map(|collision_normal| {
                                moving_state_from_collision_normal(&collision_normal)
                            })
                            .filter_map(|moving_state_opt| moving_state_opt)
                            .for_each(|moving_state| change_velocity(&moving_state, velocity));

                        Some(entity)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .filter_map(|entity_opt| entity_opt)
            .collect::<Vec<specs::Entity>>();

        joined_entities.iter().for_each(|e| {
            if let Some(data) = collision_objects.get(*e) {
                if let Some(_character_position) = position_storage.get(data.character_entity) {
                    let has_changed = PhysicsSystem::has_changed(
                        data.collision_data.contact_event,
                        &mut collision_world,
                    );

                    if has_changed {
                        println!("Removing E {:?}", e);
                        _updater.remove::<CollisionHandle>(*e);
                    }
                }
            }
        });
    }
}

#[derive(Clone, Debug)]
pub struct CollisionObjectData {
    pub name: &'static str,
    pub velocity: Option<Vector2<f32>>,
}

impl CollisionObjectData {
    pub fn new(
        name: &'static str,
        velocity: Option<Vector2<f32>>,
    ) -> CollisionObjectData {
        let init_velocity = if let Some(velocity) = velocity {
            Some(velocity)
        } else {
            None
        };

        CollisionObjectData {
            name: name,
            velocity: init_velocity,
        }
    }
}

#[derive(Default)]
pub struct Collision {
    pub world: Option<CollisionWorld<f32, CollisionObjectData>>,
    pub character_handle: Option<CollisionObjectHandle>,
    pub character_position: Option<Isometry2<f32>>,
}

impl Collision {
    pub fn set_character_position(&mut self, position: &Position) -> &mut Self {
        let position = position.0;
        self.character_position = Some(Isometry2::new(
            Vector2::new(position.x, position.y),
            nalgebra::zero(),
        ));
        self
    }
}
