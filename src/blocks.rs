use bevy::prelude::*;

mod picking;
mod render;

pub use picking::PickedQuadrant;

pub struct BlocksPlugin;

impl Plugin for BlocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((picking::PickingPlugin, render::RenderPlugin))
            .add_systems(Startup, spawn_test_blocks);
    }
}

fn spawn_test_blocks() {}

#[derive(Default, Clone, PartialEq, Eq, Component)]
pub struct Block {
    pub shape: BlockShape,
}

#[derive(Clone, PartialEq, Eq)]
pub struct BlockShape {
    pub width: usize,
    cells: Vec<bool>,
}

impl Default for BlockShape {
    fn default() -> Self {
        Self {
            width: 1,
            cells: vec![true],
        }
    }
}
