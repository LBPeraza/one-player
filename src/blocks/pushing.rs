use bevy::ecs::entity::EntityHashSet;
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
    from_coordinate: CellCoordinate,
    direction: PushDirection,
}

impl PushBlock {
    pub fn from_pick(
        entity: Entity,
        from_coordinate: CellCoordinate,
        picked_quadrant: PickedQuadrant,
    ) -> Self {
        Self {
            entity,
            from_coordinate,
            direction: picked_quadrant.into(),
        }
    }
}

fn on_push_block(
    push_block: On<PushBlock>,
    mut commands: Commands,
    query: Query<(&Block, &CellCoordinate)>,
    mut board: ResMut<Board>,
) {
    let Ok((block, cell)) = query.get(push_block.entity) else {
        return;
    };
    if *cell != push_block.from_coordinate {
        debug!("Already moved; skipping");
        return;
    }
    for occupied_cell in block.shape.occupied_cells(*cell) {
        if !board.pop_occupant(&occupied_cell, &push_block.entity) {
            warn!(
                "Occupant {} not found in board at {occupied_cell:?}",
                push_block.entity
            );
            return;
        }
    }
    let mut pushed_blocks = EntityHashSet::new();
    for occupied_cell in block.shape.occupied_cells(*cell) {
        let new_cell = &occupied_cell + &push_block.direction;
        pushed_blocks.extend(board.add_occupant(new_cell, push_block.entity));
    }
    debug!("Multi-pushing {} blocks", pushed_blocks.len());
    for pushed_block in pushed_blocks {
        let Ok((_, from_coordinate)) = query.get(pushed_block) else {
            warn!("Block {pushed_block} has no CellCoordinate component");
            continue;
        };
        commands.trigger(PushBlock {
            entity: pushed_block,
            from_coordinate: *from_coordinate,
            direction: push_block.direction,
        });
    }
    commands
        .entity(push_block.entity)
        .insert(cell + &push_block.direction);
}
