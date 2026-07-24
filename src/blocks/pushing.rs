use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;

use crate::blocks::Block;
use crate::board::*;
use crate::config::Config;

pub struct PushingPlugin;

impl Plugin for PushingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_add_block).add_observer(on_push_block);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PushDirection {
    North,
    East,
    South,
    West,
}

impl PushDirection {
    fn cell_to_cell(from_cell: CellCoordinate, to_cell: CellCoordinate) -> Option<Self> {
        let dx = to_cell.x - from_cell.x;
        let dy = to_cell.y - from_cell.y;
        if dx == 0 && dy == 0 {
            None
        } else if dx.abs() > dy.abs() {
            Some(if dx > 0 { Self::East } else { Self::West })
        } else {
            Some(if dy > 0 { Self::North } else { Self::South })
        }
    }
}

impl std::ops::Add<PushDirection> for CellCoordinate {
    type Output = CellCoordinate;

    fn add(self, rhs: PushDirection) -> Self::Output {
        match rhs {
            PushDirection::East => CellCoordinate {
                x: self.x + 1,
                ..self
            },
            PushDirection::North => CellCoordinate {
                y: self.y + 1,
                ..self
            },
            PushDirection::South => CellCoordinate {
                y: self.y - 1,
                ..self
            },
            PushDirection::West => CellCoordinate {
                x: self.x - 1,
                ..self
            },
        }
    }
}

#[derive(EntityEvent)]
pub struct PushBlock {
    entity: Entity,
    from_coordinate: CellCoordinate,
    direction: PushDirection,
}

fn on_add_block(add: On<Add, Block>, mut commands: Commands) {
    commands
        .entity(add.entity)
        .observe(on_block_drag_start)
        .observe(on_block_drag)
        .observe(on_block_drag_end)
        .observe(on_reached_target);
}

#[derive(Component)]
struct DragOrigin(CellCoordinate);

fn on_block_drag_start(drag: On<Pointer<DragStart>>, mut commands: Commands, config: Res<Config>) {
    debug!("Starting drag on {}", drag.entity);
    let Some(drag_origin) = drag.hit.position else {
        return;
    };
    let origin_coordinate = CellCoordinate::from_world(drag_origin.truncate(), config.block_size);
    debug!("  origin: {}", drag_origin.truncate());
    debug!("  origin_coordinate: {:?}", origin_coordinate);
    commands
        .entity(drag.entity)
        .insert(DragOrigin(CellCoordinate::from_world(
            drag_origin.truncate(),
            config.block_size,
        )));
}

fn on_block_drag_end(drag: On<Pointer<DragEnd>>, mut commands: Commands) {
    commands.entity(drag.entity).try_remove::<DragOrigin>();
}

fn on_reached_target(reached: On<ReachedTarget>, mut commands: Commands) {
    commands.entity(reached.entity).remove::<Moving>();
}

#[derive(Component)]
pub struct Moving;

#[derive(EntityEvent)]
pub struct ReachedTarget {
    entity: Entity,
}

impl ReachedTarget {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

fn on_block_drag(
    drag: On<Pointer<Drag>>,
    mut commands: Commands,
    moving: Query<&Moving>,
    drag_origins: Query<&DragOrigin>,
    block_origins: Query<&CellCoordinate>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    config: Res<Config>,
) {
    if moving.contains(drag.entity) {
        return;
    }
    let Ok(DragOrigin(drag_origin)) = drag_origins.get(drag.entity) else {
        warn!("No drag origin on {}", drag.entity);
        return;
    };
    let (camera, camera_transform) = *camera;
    let Ok(world_cursor) =
        camera.viewport_to_world_2d(camera_transform, drag.pointer_location.position)
    else {
        return;
    };
    let target = CellCoordinate::from_world(world_cursor, config.block_size);
    if let Some(direction) = PushDirection::cell_to_cell(*drag_origin, target) {
        commands.trigger(PushBlock {
            entity: drag.entity,
            from_coordinate: *block_origins
                .get(drag.entity)
                .expect("Block should have CellCoordinate component"),
            direction,
        });
        commands
            .entity(drag.entity)
            .insert(DragOrigin(*drag_origin + direction));
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
        let new_cell = occupied_cell + push_block.direction;
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
        .insert((Moving, *cell + push_block.direction));
}
