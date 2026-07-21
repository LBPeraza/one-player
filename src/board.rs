use bevy::{ecs::entity::EntityHashSet, platform::collections::HashMap, prelude::*};

pub const CELL_SIZE: f32 = 128.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Component)]
pub struct CellCoordinate {
    pub x: i32,
    pub y: i32,
}

impl CellCoordinate {
    pub fn from_world(world_position: Vec2) -> Self {
        let normalized = world_position + Vec2::splat(CELL_SIZE / 2.);
        Self {
            x: (normalized.x / CELL_SIZE).floor() as i32,
            y: (normalized.y / CELL_SIZE).floor() as i32,
        }
    }

    pub fn center(self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32) * CELL_SIZE
    }

    pub fn cell_position(self, world_position: Vec2) -> Vec2 {
        world_position - self.center()
    }
}

#[derive(Resource, Default)]
pub struct Board {
    occupants: HashMap<CellCoordinate, EntityHashSet>,
}

impl Board {
    pub fn occupants_at(&self, position: &CellCoordinate) -> Option<&EntityHashSet> {
        self.occupants.get(position)
    }

    /// Add entity to occupants at position and return the entities that were there before.
    pub fn add_occupant(&mut self, position: CellCoordinate, entity: Entity) -> Vec<Entity> {
        let occupants = self.occupants.entry(position).or_default();
        let current = occupants.iter().copied().collect();
        occupants.insert(entity);
        current
    }

    pub fn pop_occupant(&mut self, position: &CellCoordinate, entity: &Entity) -> bool {
        let Some(entities) = self.occupants.get_mut(position) else {
            return false;
        };
        entities.remove(entity)
    }
}

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Board>();
    }
}
