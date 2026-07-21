use bevy::prelude::*;

mod picking;
mod pushing;
mod render;

pub use picking::PickedQuadrant;
pub use pushing::PushBlock;

pub struct BlocksPlugin;

impl Plugin for BlocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            pushing::PushingPlugin,
            picking::PickingPlugin,
            render::RenderPlugin,
        ));
    }
}

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
