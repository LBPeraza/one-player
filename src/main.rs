use std::f32::consts::{FRAC_PI_2, PI};

use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::query::QueryData;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;
// use bevy_pancam::{PanCam, PanCamPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Shape2dPlugin::default()))
        .init_resource::<TileMap>()
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

const BLOCK_SIZE: f32 = 128.;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Component)]
struct TilePosition {
    x: i32,
    y: i32,
}

impl TilePosition {
    fn tile_center(&self) -> Vec2 {
        return Vec2::new(self.x as f32, self.y as f32) * BLOCK_SIZE;
    }
}

impl std::ops::Add<&PushDirection> for &TilePosition {
    type Output = TilePosition;

    fn add(self, rhs: &PushDirection) -> Self::Output {
        match rhs {
            PushDirection::EAST => TilePosition {
                x: self.x + 1,
                ..*self
            },
            PushDirection::NORTH => TilePosition {
                y: self.y + 1,
                ..*self
            },
            PushDirection::SOUTH => TilePosition {
                y: self.y - 1,
                ..*self
            },
            PushDirection::WEST => TilePosition {
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
                TilePosition { x, y },
                ShapeBundle::rect(
                    &ShapeConfig {
                        color,
                        corner_radii: Vec4::splat(BLOCK_SIZE / 12.),
                        thickness: 0.,
                        ..ShapeConfig::default_2d()
                    },
                    Vec2::splat(BLOCK_SIZE - 2.0),
                ),
            ));
        }
    }
}

fn update_transforms(
    blocks: Query<(&TilePosition, &mut Transform), (With<Block>, Changed<TilePosition>)>,
) {
    for (TilePosition { x, y }, mut tf) in blocks {
        tf.translation = BLOCK_SIZE * (Vec3::X * *x as f32 + Vec3::Y * *y as f32);
    }
}

fn pick_block(
    mut commands: Commands,
    button_input: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    map: Res<TileMap>,
    hovered: Query<(Entity, &PushDirection), With<Block>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let (camera, camera_transform) = *camera;
    let Some((hovers, direction)) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
        .and_then(|cursor_position| map.get_entity(cursor_position))
    else {
        remove_all_hovered(&mut commands, hovered);
        return;
    };
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

fn on_add_block(add: On<Add, Block>, query: Query<&TilePosition>, mut map: ResMut<TileMap>) {
    let tile = query.get(add.entity).unwrap();
    map.insert(*tile, add.entity);
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
                        Vec2::new(BLOCK_SIZE / 2., 0.),
                        Vec2::new(BLOCK_SIZE / 4., BLOCK_SIZE / 8.),
                        Vec2::new(BLOCK_SIZE / 4., -BLOCK_SIZE / 8.),
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
    query: Query<&TilePosition, With<Block>>,
    mut map: ResMut<TileMap>,
) {
    let Ok(position) = query.get(push.entity) else {
        return;
    };
    map.pop(position, &push.entity);
    let new_position = position + &push.direction;
    let pushed = map.get_tile(&new_position);
    map.insert(new_position, push.entity);
    for pushed_block in pushed {
        commands.trigger(PushBlock {
            entity: pushed_block,
            direction: push.direction,
        })
    }
    // if let Some(pushed) = map.get_tile(&new_position) {
    //     commands.trigger(PushBlock {
    //         entity: *pushed,
    //         direction: push.direction,
    //     });
    // }
    commands.entity(push.entity).insert(new_position);
}

#[derive(Resource, Default)]
struct TileMap {
    map: HashMap<TilePosition, EntityHashSet>,
}

impl TileMap {
    fn get_tile(&self, tile: &TilePosition) -> Vec<Entity> {
        if let Some(entities) = self.map.get(tile) {
            entities.iter().copied().collect()
        } else {
            vec![]
        }
    }

    fn get_entity(&self, pos: Vec2) -> Option<(Vec<Entity>, PushDirection)> {
        let normalized = pos + Vec2::splat(BLOCK_SIZE / 2.);
        let tile = TilePosition {
            x: (normalized.x / BLOCK_SIZE).floor() as i32,
            y: (normalized.y / BLOCK_SIZE).floor() as i32,
        };
        let entities = self.get_tile(&tile);
        if entities.is_empty() {
            return None;
        }
        let in_tile = pos - tile.tile_center();
        let quadrant = match (in_tile.y >= in_tile.x, in_tile.y >= -in_tile.x) {
            (false, false) => PushDirection::SOUTH,
            (false, true) => PushDirection::EAST,
            (true, false) => PushDirection::WEST,
            (true, true) => PushDirection::NORTH,
        };
        Some((entities, quadrant))
    }

    fn pop(&mut self, tile: &TilePosition, entity: &Entity) -> bool {
        let Some(entities) = self.map.get_mut(tile) else {
            return false;
        };
        entities.remove(entity)
    }

    fn insert(&mut self, tile: TilePosition, entity: Entity) -> bool {
        self.map.entry(tile).or_default().insert(entity)
    }
}
