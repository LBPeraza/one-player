use std::f32::consts::{FRAC_PI_2, PI};

use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

use crate::board::*;
// use bevy_pancam::{PanCam, PanCamPlugin};

mod board;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Shape2dPlugin::default(), BoardPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (update_transforms, pick_block, update_hover))
        .add_observer(on_add_block)
        .add_observer(on_push_block)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    spawn_blocks(&mut commands);
}

#[derive(Component)]
struct Block {
    _width: usize,
    _shape: Vec<bool>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Component)]
enum PushDirection {
    NORTH,
    EAST,
    SOUTH,
    WEST,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            _width: 1,
            _shape: vec![true],
        }
    }
}

impl Block {
    fn _size(self) -> [usize; 2] {
        [self._width, self._shape.len() / self._width]
    }
}

impl std::ops::Add<&PushDirection> for &CellCoordinate {
    type Output = CellCoordinate;

    fn add(self, rhs: &PushDirection) -> Self::Output {
        match rhs {
            PushDirection::EAST => CellCoordinate {
                x: self.x + 1,
                ..*self
            },
            PushDirection::NORTH => CellCoordinate {
                y: self.y + 1,
                ..*self
            },
            PushDirection::SOUTH => CellCoordinate {
                y: self.y - 1,
                ..*self
            },
            PushDirection::WEST => CellCoordinate {
                x: self.x - 1,
                ..*self
            },
        }
    }
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

fn pick_block(
    mut commands: Commands,
    button_input: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    board: Res<Board>,
    hovered: Query<(Entity, &PushDirection), With<Block>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let (camera, camera_transform) = *camera;
    let Some(cursor_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    // .map(|cursor_position| CellCoordinate::from_world(cursor_position))
    else {
        remove_all_hovered(&mut commands, hovered);
        return;
    };
    let cell = CellCoordinate::from_world(cursor_position);
    let cursor_in_cell = cell.cell_position(cursor_position);
    let direction = match (
        cursor_in_cell.y >= cursor_in_cell.x,
        cursor_in_cell.y >= -cursor_in_cell.x,
    ) {
        (false, false) => PushDirection::SOUTH,
        (false, true) => PushDirection::EAST,
        (true, false) => PushDirection::WEST,
        (true, true) => PushDirection::NORTH,
    };
    let hovers = board.occupants_at(&cell);
    remove_hovered_except(&mut commands, hovered, &hovers, direction);
    for hover in hovers.iter() {
        commands.entity(*hover).insert_if_new(direction);
    }
    if button_input.just_pressed(MouseButton::Left) {
        for hover in hovers {
            commands.trigger(PushBlock {
                entity: hover,
                direction,
            });
        }
    }
}

fn remove_all_hovered<T: QueryData>(
    commands: &mut Commands,
    hovered: Query<(Entity, T), With<Block>>,
) {
    for (e, _) in hovered {
        commands.entity(e).remove::<PushDirection>();
    }
}

fn remove_hovered_except(
    commands: &mut Commands,
    hovered: Query<(Entity, &PushDirection), With<Block>>,
    except_entities: &Vec<Entity>,
    except_direction: PushDirection,
) {
    for (e, d) in hovered {
        if !(except_entities.contains(&e) && *d == except_direction) {
            commands.entity(e).remove::<PushDirection>();
        }
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
    hovered: Query<(Entity, &PushDirection), With<Block>>,
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
                    PushDirection::EAST => 0.,
                    PushDirection::NORTH => FRAC_PI_2,
                    PushDirection::WEST => PI,
                    PushDirection::SOUTH => FRAC_PI_2 * 3.,
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

#[derive(EntityEvent)]
struct PushBlock {
    entity: Entity,
    direction: PushDirection,
}

fn on_push_block(
    push: On<PushBlock>,
    mut commands: Commands,
    query: Query<&CellCoordinate, With<Block>>,
    mut board: ResMut<Board>,
) {
    let Ok(position) = query.get(push.entity) else {
        return;
    };
    board.pop_occupant(position, &push.entity);
    let new_position = position + &push.direction;
    let pushed = board.occupants_at(&new_position);
    board.add_occupant(new_position, push.entity);
    for pushed_block in pushed {
        commands.trigger(PushBlock {
            entity: pushed_block,
            direction: push.direction,
        })
    }
    commands.entity(push.entity).insert(new_position);
}
