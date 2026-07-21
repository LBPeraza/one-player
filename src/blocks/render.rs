use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

use crate::blocks::*;
use crate::board::*;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_block_transforms)
            .add_observer(on_add_block)
            .add_observer(on_pick_block)
            .add_observer(on_unpick_block);
    }
}

fn apply_block_transforms(
    blocks: Query<(&CellCoordinate, &mut Transform), (With<Block>, Changed<CellCoordinate>)>,
) {
    for (CellCoordinate { x, y }, mut tf) in blocks {
        debug!("Applying translation to move block to ({x}, {y})");
        tf.translation = CELL_SIZE * (Vec3::new(*x as f32, *y as f32, 0.));
    }
}

fn on_add_block(add: On<Add, Block>, mut commands: Commands, query: Query<&Block>) {
    let block = query
        .get(add.entity)
        .expect("entity should have Block component");
    commands.entity(add.entity).with_children(|builder| {
        for cell in block.shape.occupied_cells(CellCoordinate::default()) {
            builder.spawn(ShapeBundle::rect(
                &ShapeConfig {
                    color: block.color,
                    thickness: 0.,
                    transform: Transform::from_translation(cell.center().extend(0.)),
                    ..ShapeConfig::default_2d()
                },
                Vec2::splat(CELL_SIZE),
            ));
        }
    });
}

#[derive(Component)]
struct HoverIndicator;

fn on_unpick_block(
    remove: On<Remove, PickedQuadrant>,
    mut commands: Commands,
    mut query: Query<&mut Transform>,
    children: Query<Entity, With<HoverIndicator>>,
) {
    let mut tf = query
        .get_mut(remove.entity)
        .expect("query should return transform");
    tf.translation.z = 0.;
    for child in children {
        commands.entity(child).despawn();
    }
}

fn on_pick_block(
    add: On<Add, PickedQuadrant>,
    mut commands: Commands,
    mut query: Query<(&Block, &PickedQuadrant, &mut Transform)>,
) {
    let (block, quadrant, mut tf) = query
        .get_mut(add.entity)
        .expect("entity should have Block, PickedQuadrant, and Transform components");
    let indicator_color = block.color.rotate_hue(180.);
    tf.translation.z = 1.;
    let rotation = match quadrant {
        PickedQuadrant::EAST => 0.,
        PickedQuadrant::NORTH => FRAC_PI_2,
        PickedQuadrant::SOUTH => 3. * FRAC_PI_2,
        PickedQuadrant::WEST => PI,
    };
    commands.entity(add.entity).with_children(|builder| {
        builder.spawn((
            HoverIndicator,
            ShapeBundle::rect(
                &ShapeConfig {
                    transform: Transform::from_translation(-Vec3::Z),
                    color: indicator_color,
                    corner_radii: Vec4::splat(CELL_SIZE / 12. + 2.),
                    ..ShapeConfig::default_2d()
                },
                Vec2::splat(CELL_SIZE * 1.1),
            ),
        ));
        builder.spawn((
            HoverIndicator,
            ShapeBundle::circle(
                &ShapeConfig {
                    transform: Transform::from_translation(
                        Quat::from_rotation_z(rotation) * (Vec3::X * CELL_SIZE / 3. + Vec3::Z),
                    ),
                    color: indicator_color,
                    ..ShapeConfig::default_2d()
                },
                CELL_SIZE / 10.,
            ),
        ));
    });
}
