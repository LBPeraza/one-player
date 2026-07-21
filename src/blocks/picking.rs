use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;

use crate::board::{Board, CellCoordinate};

pub struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, pick_blocks);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Component)]
pub enum PickedQuadrant {
    NORTH,
    EAST,
    SOUTH,
    WEST,
}

fn pick_blocks(
    mut commands: Commands,
    camera: Single<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    board: Res<Board>,
    picked: Query<(Entity, &PickedQuadrant)>,
) {
    let Some(cursor) = windows
        .single()
        .ok()
        .and_then(|window| window.cursor_position())
        .and_then(|cursor| {
            let (cam, cam_transform) = *camera;
            cam.viewport_to_world_2d(cam_transform, cursor).ok()
        })
    else {
        remove_all_picked(&mut commands, picked);
        return;
    };

    let picked_cell = CellCoordinate::from_world(cursor);
    let Some(picked_blocks) = board.occupants_at(&picked_cell) else {
        remove_all_picked(&mut commands, picked);
        return;
    };

    let cursor_in_cell = picked_cell.cell_position(cursor);
    let picked_quadrant = match (
        cursor_in_cell.y >= cursor_in_cell.x,
        cursor_in_cell.y >= -cursor_in_cell.x,
    ) {
        (false, false) => PickedQuadrant::SOUTH,
        (false, true) => PickedQuadrant::EAST,
        (true, false) => PickedQuadrant::WEST,
        (true, true) => PickedQuadrant::NORTH,
    };

    remove_picked_except(&mut commands, picked, picked_blocks, &picked_quadrant);
    for pick in picked_blocks {
        commands.entity(*pick).insert_if_new(picked_quadrant);
    }
}

fn remove_all_picked(commands: &mut Commands, picked: Query<(Entity, &PickedQuadrant)>) {
    for (e, _) in picked {
        commands.entity(e).remove::<PickedQuadrant>();
    }
}

fn remove_picked_except(
    commands: &mut Commands,
    picked: Query<(Entity, &PickedQuadrant)>,
    except_picked: &EntityHashSet,
    except_quadrant: &PickedQuadrant,
) {
    for (e, quadrant) in picked {
        if !(except_picked.contains(&e) && except_quadrant == quadrant) {
            commands.entity(e).remove::<PickedQuadrant>();
        }
    }
}
