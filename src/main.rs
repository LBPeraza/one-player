use std::f32::consts::{FRAC_PI_2, PI};

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

mod blocks;
mod board;

use crate::blocks::*;
use crate::board::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Shape2dPlugin::default(),
            BoardPlugin,
            BlocksPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_transforms,
                update_hover,
                push_block.run_if(input_just_pressed(MouseButton::Left)),
            ),
        )
        .add_observer(on_add_block)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    spawn_blocks(&mut commands);
}

fn spawn_blocks(commands: &mut Commands) {
    for x in 0..4 {
        for y in 0..3 {
            let color = if (x + y) % 2 == 0 {
                Color::BLACK
            } else {
                Color::WHITE
            };
            commands.spawn((
                Block::default(),
                CellCoordinate { x, y },
                ShapeBundle::rect(
                    &ShapeConfig {
                        color,
                        corner_radii: Vec4::splat(CELL_SIZE / 12.),
                        thickness: 0.,
                        ..ShapeConfig::default_2d()
                    },
                    Vec2::splat(CELL_SIZE - 2.0),
                ),
            ));
        }
    }
}

fn update_transforms(
    blocks: Query<(&CellCoordinate, &mut Transform), (With<Block>, Changed<CellCoordinate>)>,
) {
    for (CellCoordinate { x, y }, mut tf) in blocks {
        tf.translation = CELL_SIZE * (Vec3::X * *x as f32 + Vec3::Y * *y as f32);
    }
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

#[derive(Component)]
struct PushArrow;

fn update_hover(
    mut commands: Commands,
    shapes: ShapeCommands,
    old_arrows: Query<Entity, With<PushArrow>>,
    hovered: Query<(Entity, &PickedQuadrant), With<Block>>,
) {
    for old in old_arrows {
        commands.entity(old).despawn();
    }
    for (e, q) in hovered {
        commands
            .entity(e)
            .with_shape_children(shapes.config(), |builder| {
                builder.translate(Vec3::Z);
                builder.rotate_z(match q {
                    PickedQuadrant::EAST => 0.,
                    PickedQuadrant::NORTH => FRAC_PI_2,
                    PickedQuadrant::WEST => PI,
                    PickedQuadrant::SOUTH => FRAC_PI_2 * 3.,
                });
                builder.color = Color::srgb(0., 1., 0.);
                builder
                    .triangle(
                        Vec2::new(CELL_SIZE / 2., 0.),
                        Vec2::new(CELL_SIZE / 4., CELL_SIZE / 8.),
                        Vec2::new(CELL_SIZE / 4., -CELL_SIZE / 8.),
                    )
                    .insert(PushArrow);
            });
    }
}
