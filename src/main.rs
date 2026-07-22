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
    let blocks = [
        (Block::default(), CellCoordinate::new(0, 0)),
        (
            Block::default().with_shape(BlockShape::new([[true], [true]])),
            CellCoordinate::new(1, 0),
        ),
        (
            Block::default().with_shape(BlockShape::new([[true, false], [true, true]])),
            CellCoordinate::new(0, 1),
        ),
        (
            Block::default().with_shape(BlockShape::new([
                [true; 6],
                [true, false, false, false, false, true],
                [true, false, false, false, false, true],
                [true, false, false, false, false, true],
                [true, false, false, false, false, true],
                [true; 6],
            ])),
            CellCoordinate::new(-1, -1),
        ),
        (
            Block::default().with_shape(BlockShape::new([[true; 3]; 3])),
            CellCoordinate::new(-4, -1),
        ),
    ];
    let hue_interval = 360. / (blocks.len() + 1) as f32;
    for (i, (block, cell)) in blocks.into_iter().enumerate() {
        commands.spawn((
            block.with_color(Color::hsv(hue_interval * i as f32, 0.8, 0.8)),
            cell,
            Transform::default(),
            Visibility::default(),
        ));
    }
}
