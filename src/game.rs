use bevy::prelude::*;
pub struct GamePlugin;

use crate::blocks::*;
use crate::board::*;
use crate::camera::*;
use crate::config::*;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum GameState {
    Loading,
    Playing,
}

impl ComputedStates for GameState {
    type SourceStates = ConfigLoadState;

    const ALLOW_SAME_STATE_TRANSITIONS: bool = false;

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        Some(match sources {
            ConfigLoadState::Loading => Self::Loading,
            ConfigLoadState::Ready => Self::Playing,
        })
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_computed_state::<GameState>()
            .add_plugins((BoardPlugin, BlocksPlugin, CameraPlugin, ConfigPlugin))
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
