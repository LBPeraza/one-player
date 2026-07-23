use bevy::prelude::*;

mod picking;
mod pushing;
mod render;

use crate::board::CellCoordinate;

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

#[derive(Clone, PartialEq, Component)]
pub struct Block {
    pub shape: BlockShape,
    pub color: Color,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            shape: default(),
            color: Color::hsv(0., 0., 0.75),
        }
    }
}

impl Block {
    pub fn with_shape(self, shape: BlockShape) -> Self {
        Self { shape, ..self }
    }

    pub fn with_color(self, color: Color) -> Self {
        Self { color, ..self }
    }
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

impl BlockShape {
    pub fn new<const W: usize, const H: usize>(shape: [[bool; W]; H]) -> Self {
        let mut cells = Vec::with_capacity(W * H);
        for row in shape {
            cells.extend_from_slice(&row);
        }
        Self { width: W, cells }
    }

    pub fn occupied_cells(&self, origin: CellCoordinate) -> impl Iterator<Item = CellCoordinate> {
        self.cells
            .iter()
            .enumerate()
            .filter(|(_, occupied)| **occupied)
            .map(move |(index, _)| {
                let local_x = index % self.width;
                let local_y = index / self.width;
                CellCoordinate {
                    x: origin.x + local_x as i32,
                    y: origin.y + local_y as i32,
                }
            })
    }
}
