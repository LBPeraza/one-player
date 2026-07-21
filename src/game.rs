use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
pub struct GamePlugin;

use crate::blocks::*;
use crate::board::*;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((BoardPlugin, BlocksPlugin))
            .add_systems(Startup, setup_camera)
            .add_systems(
                Update,
                push_block.run_if(input_just_pressed(MouseButton::Left)),
            )
            .add_observer(on_add_block);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn push_block(mut commands: Commands, picked: Query<(Entity, &PickedQuadrant), With<Block>>) {
    for (entity, quadrant) in picked {
        commands.trigger(PushBlock::from_pick(entity, *quadrant));
    }
}

fn on_add_block(add: On<Add, Block>, query: Query<&CellCoordinate>, mut board: ResMut<Board>) {
    let tile = query.get(add.entity).unwrap();
    board.add_occupant(*tile, add.entity);
}
