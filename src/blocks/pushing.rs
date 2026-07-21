use bevy::prelude::*;

use crate::blocks::{Block, picking::PickedQuadrant};
use crate::board::*;

pub struct PushingPlugin;

impl Plugin for PushingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_push_block);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PushDirection {
    NORTH,
    EAST,
    SOUTH,
    WEST,
}

impl std::ops::Add<&PushDirection> for &CellCoordinate {
    type Output = CellCoordinate;

    fn add(self, rhs: &PushDirection) -> Self::Output {
        match rhs {
            PushDirection::EAST => CellCoordinate {
                x: self.x + 1,
                ..*self
            },
            PushDirection::NORTH => CellCoordinate {
                y: self.y + 1,
                ..*self
            },
            PushDirection::SOUTH => CellCoordinate {
                y: self.y - 1,
                ..*self
            },
            PushDirection::WEST => CellCoordinate {
                x: self.x - 1,
                ..*self
            },
        }
    }
}

impl From<PickedQuadrant> for PushDirection {
    fn from(value: PickedQuadrant) -> Self {
        match value {
            PickedQuadrant::EAST => Self::EAST,
            PickedQuadrant::NORTH => Self::NORTH,
            PickedQuadrant::SOUTH => Self::SOUTH,
            PickedQuadrant::WEST => Self::WEST,
        }
    }
}

#[derive(EntityEvent)]
pub struct PushBlock {
    entity: Entity,
    direction: PushDirection,
}

impl PushBlock {
    pub fn from_pick(entity: Entity, picked_quadrant: PickedQuadrant) -> Self {
        Self {
            entity,
            direction: picked_quadrant.into(),
        }
    }
}

fn on_push_block(
    push_block: On<PushBlock>,
    mut commands: Commands,
    query: Query<&CellCoordinate, With<Block>>,
    mut board: ResMut<Board>,
) {
    let Ok(cell) = query.get(push_block.entity) else {
        return;
    };
    if !board.pop_occupant(cell, &push_block.entity) {
        warn!(
            "Occupant {} not found in board at {cell:?}",
            push_block.entity
        );
        return;
    }
    let new_cell = cell + &push_block.direction;
    for pushed in board.add_occupant(new_cell, push_block.entity) {
        commands.trigger(PushBlock {
            entity: pushed,
            direction: push_block.direction,
        });
    }
    commands.entity(push_block.entity).insert(new_cell);
}
