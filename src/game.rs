use bevy::prelude::*;
pub struct GamePlugin;

use crate::blocks::*;
use crate::board::*;
use crate::camera::*;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((BoardPlugin, BlocksPlugin, CameraPlugin))
            .add_observer(on_add_block);
    }
}

fn on_add_block(
    add: On<Add, Block>,
    query: Query<(&Block, &CellCoordinate)>,
    mut board: ResMut<Board>,
) {
    let (block, cell) = query.get(add.entity).unwrap();
    for occupied in block.shape.occupied_cells(*cell) {
        board.add_occupant(occupied, add.entity);
    }
}
