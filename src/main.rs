use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

mod blocks;
mod board;
mod game;

use crate::blocks::*;
use crate::board::*;
use crate::game::GamePlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Shape2dPlugin::default(), GamePlugin))
        .add_systems(Startup, spawn_test_blocks)
        .run();
}

fn spawn_test_blocks(mut commands: Commands) {
    for x in 0..4 {
        for y in 0..3 {
            commands.spawn((
                Block::default(),
                CellCoordinate { x, y },
                Transform::default(),
                Visibility::default(),
            ));
        }
    }
}
