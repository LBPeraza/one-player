use bevy::{ecs::entity::EntityHashSet, platform::collections::HashMap, prelude::*};

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash, Component)]
pub struct CellCoordinate {
    pub x: i32,
    pub y: i32,
}

impl CellCoordinate {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn from_world(world_position: Vec2, cell_size: f32) -> Self {
        let normalized = world_position + Vec2::splat(cell_size / 2.);
        Self {
            x: (normalized.x / cell_size).floor() as i32,
            y: (normalized.y / cell_size).floor() as i32,
        }
    }

    pub fn center(&self, cell_size: f32) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32) * cell_size
    }
}

impl std::ops::Add<(i32, i32)> for CellCoordinate {
    type Output = Self;

    fn add(self, (dx, dy): (i32, i32)) -> Self::Output {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

#[derive(Resource, Default)]
pub struct Board {
    occupants: HashMap<CellCoordinate, EntityHashSet>,
}

impl Board {
    pub fn occupants_at(&self, position: &CellCoordinate) -> Option<&EntityHashSet> {
        self.occupants
            .get(position)
            .filter(|entities| !entities.is_empty())
    }

    /// Add entity to occupants at position and return the entities that were there before.
    pub fn add_occupant(&mut self, position: CellCoordinate, entity: Entity) -> Vec<Entity> {
        let occupants = self.occupants.entry(position).or_default();
        let current = occupants.iter().copied().filter(|e| e != &entity).collect();
        occupants.insert(entity);
        current
    }

    pub fn pop_occupant(&mut self, position: &CellCoordinate, entity: &Entity) -> bool {
        let Some(entities) = self.occupants.get_mut(position) else {
            return false;
        };
        let popped = entities.remove(entity);
        if entities.is_empty() {
            self.occupants.remove(position);
        }
        popped
    }
}

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Board>();
    }
}
