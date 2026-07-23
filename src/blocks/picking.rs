use bevy::picking::backend::prelude::*;
use bevy::prelude::*;

use crate::board::{Board, CellCoordinate};

pub struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, pick_blocks);
    }
}

fn pick_blocks(ray_map: Res<RayMap>, board: Res<Board>, mut hits: MessageWriter<PointerHits>) {
    for (&ray_id, ray) in ray_map.iter() {
        let cursor_world = ray.origin.truncate();
        let Some(picked_blocks) = board.occupants_at(&CellCoordinate::from_world(cursor_world))
        else {
            continue;
        };
        let mut picks = Vec::with_capacity(picked_blocks.len());
        for picked_block in picked_blocks {
            picks.push((
                *picked_block,
                HitData::new(
                    ray_id.camera,
                    0.,
                    Some(cursor_world.extend(0.)),
                    Some(Vec3::Z),
                ),
            ));
        }
        hits.write(PointerHits::new(ray_id.pointer, picks, 0.));
    }
}
