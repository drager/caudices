use super::{Position, World as EntitiesWorld};
use character::Character;
use nalgebra::{Isometry2, Vector2};
use ncollide2d::events::{ContactEvent, ProximityEvent};
use ncollide2d::query::Proximity;
use ncollide2d::shape::{Cuboid, ShapeHandle};
use ncollide2d::world::{
    CollisionGroups, CollisionObject, CollisionObjectHandle, CollisionWorld, GeometricQueryType,
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
    //println!("Surface area {:?}", surface_area);
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
        let x = (&entities, &mut collision_objects)
            .join()
            .map(|(entity, collision_data)| {
                let character_entity = collision_data.character_entity;
                if let Some(character_position) = position_storage.get(character_entity) {
                    if let Some(ref mut velocity) = velocity_storage.get_mut(character_entity) {
                        let block_position = collision_data
                            .collision_data
                            .collision_object
                            .position()
                            .translation
                            .vector;

                        let surface_area = (block_position.x + 25.)
                            .min(character_position.0.x + 20.)
                            - (block_position.x).max(character_position.0.x);

                        //println!("Surface area {:?}", surface_area);

                        let collision_normals = &collision_data.collision_data.collision_normals;

                        //println!("normals {:?}", collision_normals.len());
                        /*if let Some(collision_normal) =*/
                        //collision_data.collision_data.collision_normal
                        //{
                        //let moving_state =
                        //moving_state_from_collision_normal(&collision_normal);

                        //if let Some(moving_state) = moving_state {
                        //change_velocity(&moving_state, velocity);
                        //}
                        /*}*/

                        collision_normals
                            .iter()
                            .map(|collision_normal| {
                                moving_state_from_collision_normal(&collision_normal.0)
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
                //println!("Got collision in system {:?}", block_position);

                //updater.insert(collision_object, position.clone());
            })
            .filter_map(|entity_opt| entity_opt)
            .collect::<Vec<specs::Entity>>();

        x.iter().for_each(|e| {
            if let Some(data) = collision_objects.get(*e) {
                if let Some(character_position) = position_storage.get(data.character_entity) {
                    let has_changed = PhysicsSystem::has_changed(
                        data.collision_data.proximity_event,
                        &mut collision_world,
                    );
                    /*let new_collisions =*/
                    //PhysicsSystem::update_collision(character_position, &mut collision_world);

                    //new_collisions.iter().for_each(|new_collision| {
                    //if new_collision.proximity_event.new_status == Proximity::Disjoint
                    //&& new_collision.proximity_event.collider2.0
                    //== data.collision_data.proximity_event.collider2.0
                    //{
                    //println!("NEW COL {:?}", new_collision.proximity_event);

                    //_updater.remove::<CollisionHandle>(*e);
                    //}
                    /*});*/
                    //println!("Data {:?}", data.collision_data.proximity_event);
                    //if data.collision_data.proximity_event.new_status == Proximity::Disjoint {
                    if has_changed {
                        println!("Removing E {:?}", e);
                        _updater.remove::<CollisionHandle>(*e);
                    }
                }
            }
        });
    }
}

type MatrixPosition = nalgebra::MatrixArray<f32, nalgebra::U2, nalgebra::U1>;

#[derive(Clone, Debug)]
pub struct CollisionObjectData {
    pub name: &'static str,
    pub velocity: Option<Vector2<f32>>,
    pub position: Option<MatrixPosition>,
    pub character_position: Option<Vector2<f32>>,
}

impl CollisionObjectData {
    pub fn new(
        name: &'static str,
        velocity: Option<Vector2<f32>>,
        position: Option<MatrixPosition>,
        character_position: Option<Vector2<f32>>,
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
            character_position,
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
