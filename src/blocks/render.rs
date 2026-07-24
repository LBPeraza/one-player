use std::f32::consts::FRAC_PI_4;
use std::f32::consts::{FRAC_PI_2, PI};

use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;
use serde::Deserialize;

use crate::blocks::*;
use crate::board::*;
use crate::config::Config;
use crate::game::GameState;

pub(super) struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                apply_block_transforms.run_if(in_state(GameState::Playing)),
                render_blocks
                    .run_if(in_state(GameState::Playing).and_then(resource_changed::<Config>)),
            ),
        )
        .add_observer(on_add_block);
    }
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Resource)]
#[serde(default)]
pub struct RenderConfig {
    /// Movement speed, in blocks/second
    move_speed: f32,
    /// Size of block borders as scale of block size (0.0 - 0.5)
    border_scale: f32,
    /// Size of block highlights as scale of block border
    highlight_scale: f32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            move_speed: 5.,
            border_scale: 1. / 16.,
            highlight_scale: 1.5,
        }
    }
}

fn apply_block_transforms(
    mut commands: Commands,
    blocks: Query<(Entity, Ref<CellCoordinate>, &mut Transform, Option<&Moving>), With<Block>>,
    time: Res<Time>,
    config: Res<Config>,
) {
    let config_changed = config.is_changed();
    for (entity, cell, mut tf, moving) in blocks {
        if !(config_changed || cell.is_changed() || moving.is_some()) {
            continue;
        }
        let CellCoordinate { x, y } = *cell;
        debug!("Applying translation to move block to ({x}, {y})");
        let target = config.block_size * (Vec3::new(x as f32, y as f32, 0.));
        if moving.is_none() {
            tf.translation = target;
            continue;
        }
        let move_distance = config.block_size * config.render.move_speed * time.delta_secs();
        let diff = target - tf.translation;
        if diff.length() <= move_distance {
            tf.translation = target;
            commands.trigger(ReachedTarget::new(entity));
            continue;
        }
        tf.translation += diff.normalize() * move_distance;
    }
}

fn on_add_block(add: On<Add, Block>, mut commands: Commands) {
    commands
        .entity(add.entity)
        .observe(on_pick_block)
        .observe(on_unpick_block);
}

#[derive(Component)]
struct BlockVisual;

fn render_blocks(
    mut commands: Commands,
    visuals: Query<Entity, With<BlockVisual>>,
    blocks: Query<(Entity, &Block)>,
    config: Res<Config>,
) {
    for visual in visuals {
        commands.entity(visual).despawn();
    }
    for (entity, block) in blocks {
        commands.entity(entity).with_children(|builder| {
            let occupied_cells = block
                .shape
                .occupied_cells(CellCoordinate::default())
                .collect::<Vec<_>>();
            // Render cell "bodies"
            for cell in occupied_cells.iter() {
                builder.spawn((
                    BlockVisual,
                    ShapeBundle::rect(
                        &ShapeConfig {
                            color: block.color,
                            thickness: 0.,
                            corner_radii: get_corner_radii(cell, &occupied_cells)
                                * config.block_size
                                * config.render.border_scale,
                            transform: Transform::from_translation(
                                cell.center(config.block_size).extend(0.),
                            ),
                            ..ShapeConfig::default_2d()
                        },
                        Vec2::splat(config.block_size),
                    ),
                ));
            }
            let light = block.color.lighter(0.25);
            let dark = block.color.darker(0.1);

            for cell in occupied_cells.iter() {
                render_cell_boundaries(builder, cell, &occupied_cells, light, dark, *config);
            }
        });
    }
}

fn get_corner_radii(cell: &CellCoordinate, occupied_cells: &[CellCoordinate]) -> Vec4 {
    let has_cell_up = occupied_cells.contains(&(*cell + (0, 1)));
    let has_cell_right = occupied_cells.contains(&(*cell + (1, 0)));
    let has_cell_down = occupied_cells.contains(&(*cell + (0, -1)));
    let has_cell_left = occupied_cells.contains(&(*cell + (-1, 0)));
    // start bottom right, clockwise
    let cr = |radius| if radius { 1. } else { 0. };
    Vec4::new(
        cr(!(has_cell_down || has_cell_right)),
        cr(!(has_cell_down || has_cell_left)),
        cr(!(has_cell_up || has_cell_left)),
        cr(!(has_cell_up || has_cell_right)),
    )
}

fn render_cell_boundaries<R: Relationship>(
    builder: &mut RelatedSpawnerCommands<R>,
    cell: &CellCoordinate,
    occupied_cells: &[CellCoordinate],
    light: Color,
    dark: Color,
    config: Config,
) {
    let cell_center = cell.center(config.block_size);
    let top = !occupied_cells.contains(&(*cell + (0, 1)));
    let bottom = !occupied_cells.contains(&(*cell + (0, -1)));
    let left = !occupied_cells.contains(&(*cell + (-1, 0)));
    let right = !occupied_cells.contains(&(*cell + (1, 0)));

    let shape_config = |color, (x, y)| ShapeConfig {
        color,
        thickness: 0.,
        transform: Transform::from_translation((cell_center + Vec2::new(x, y)).extend(0.5)),
        cap: Cap::None,
        ..ShapeConfig::default_2d()
    };

    let border_size = config.block_size * config.render.border_scale;
    let edge_distance = (config.block_size - border_size) / 2.;
    let radius_to_radius = config.block_size - 2. * border_size;

    if top {
        builder.spawn((
            BlockVisual,
            ShapeBundle::rect(
                &shape_config(light, (0., edge_distance)),
                Vec2::new(radius_to_radius, border_size),
            ),
        ));
    }
    if left {
        builder.spawn((
            BlockVisual,
            ShapeBundle::rect(
                &shape_config(light, (-edge_distance, 0.)),
                Vec2::new(border_size, radius_to_radius),
            ),
        ));
    }
    if right {
        builder.spawn((
            BlockVisual,
            ShapeBundle::rect(
                &shape_config(dark, (edge_distance, 0.)),
                Vec2::new(border_size, radius_to_radius),
            ),
        ));
    }
    if bottom {
        builder.spawn((
            BlockVisual,
            ShapeBundle::rect(
                &shape_config(dark, (0., -edge_distance)),
                Vec2::new(radius_to_radius, border_size),
            ),
        ));
    }

    if top && left {
        builder.spawn((
            BlockVisual,
            ShapeBundle::arc(
                &shape_config(
                    light,
                    (
                        -config.block_size / 2. + border_size,
                        config.block_size / 2. - border_size,
                    ),
                ),
                border_size,
                -FRAC_PI_2,
                0.,
            ),
        ));
    }
    if bottom && right {
        builder.spawn((
            BlockVisual,
            ShapeBundle::arc(
                &shape_config(
                    dark,
                    (
                        config.block_size / 2. - border_size,
                        -config.block_size / 2. + border_size,
                    ),
                ),
                border_size,
                FRAC_PI_2,
                PI,
            ),
        ));
    }

    if top ^ left {
        builder.spawn((
            BlockVisual,
            ShapeBundle::rect(
                &shape_config(light, (-edge_distance, edge_distance)),
                Vec2::splat(border_size),
            ),
        ));
    }
    if bottom ^ right {
        builder.spawn((
            BlockVisual,
            ShapeBundle::rect(
                &shape_config(dark, (edge_distance, -edge_distance)),
                Vec2::splat(border_size),
            ),
        ));
    }

    if top ^ right {
        builder.spawn((
            BlockVisual,
            ShapeBundle::rect(
                &shape_config(
                    if top { light } else { dark },
                    (edge_distance, edge_distance),
                ),
                Vec2::splat(border_size),
            ),
        ));
    }
    if left ^ bottom {
        builder.spawn((
            BlockVisual,
            ShapeBundle::rect(
                &shape_config(
                    if left { light } else { dark },
                    (-edge_distance, -edge_distance),
                ),
                Vec2::splat(border_size),
            ),
        ));
    }

    if top && right {
        let top_right = (
            config.block_size / 2. - border_size,
            config.block_size / 2. - border_size,
        );
        builder.spawn((
            BlockVisual,
            ShapeBundle::arc(&shape_config(light, top_right), border_size, 0., FRAC_PI_4),
        ));
        builder.spawn((
            BlockVisual,
            ShapeBundle::arc(
                &shape_config(dark, top_right),
                border_size,
                FRAC_PI_4,
                FRAC_PI_2,
            ),
        ));
    }
    if bottom && left {
        let bottom_left = (
            -config.block_size / 2. + border_size,
            -config.block_size / 2. + border_size,
        );
        builder.spawn((
            BlockVisual,
            ShapeBundle::arc(
                &shape_config(dark, bottom_left),
                border_size,
                PI,
                PI + FRAC_PI_4,
            ),
        ));
        builder.spawn((
            BlockVisual,
            ShapeBundle::arc(
                &shape_config(light, bottom_left),
                border_size,
                PI + FRAC_PI_4,
                3. * FRAC_PI_2,
            ),
        ));
    }

    if !(left || top || occupied_cells.contains(&(*cell + (-1, 1)))) {
        builder.spawn((
            BlockVisual,
            ShapeBundle::arc(
                &shape_config(light, (-config.block_size / 2., config.block_size / 2.)),
                border_size,
                FRAC_PI_2,
                PI,
            ),
        ));
    }
    if !(right || bottom || occupied_cells.contains(&(*cell + (1, -1)))) {
        builder.spawn((
            BlockVisual,
            ShapeBundle::arc(
                &shape_config(dark, (config.block_size / 2., -config.block_size / 2.)),
                border_size,
                -FRAC_PI_2,
                0.,
            ),
        ));
    }

    if !(top || right || occupied_cells.contains(&(*cell + (1, 1)))) {
        let top_right = (config.block_size / 2., config.block_size / 2.);
        builder.spawn((
            BlockVisual,
            ShapeBundle::arc(
                &shape_config(light, top_right),
                border_size,
                PI,
                PI + FRAC_PI_4,
            ),
        ));
        builder.spawn((
            BlockVisual,
            ShapeBundle::arc(
                &shape_config(dark, top_right),
                border_size,
                PI + FRAC_PI_4,
                3. * FRAC_PI_2,
            ),
        ));
    }
    if !(left || bottom || occupied_cells.contains(&(*cell + (-1, -1)))) {
        let bottom_left = (-config.block_size / 2., -config.block_size / 2.);
        builder.spawn((
            BlockVisual,
            ShapeBundle::arc(&shape_config(dark, bottom_left), border_size, 0., FRAC_PI_4),
        ));
        builder.spawn((
            BlockVisual,
            ShapeBundle::arc(
                &shape_config(light, bottom_left),
                border_size,
                FRAC_PI_4,
                FRAC_PI_2,
            ),
        ));
    }
}

#[derive(Component)]
struct HoverIndicator;

fn on_unpick_block(
    out: On<Pointer<Out>>,
    mut commands: Commands,
    mut query: Query<&mut Transform>,
    existing_children: Query<&Children>,
    indicators: Query<(), With<HoverIndicator>>,
) {
    query
        .get_mut(out.entity)
        .expect("entity should have Transform component")
        .translation
        .z = 0.;
    let Ok(children) = existing_children.get(out.entity) else {
        return;
    };
    for child in children {
        if indicators.contains(*child) {
            commands.entity(*child).despawn();
        }
    }
}

fn on_pick_block(
    over: On<Pointer<Over>>,
    mut commands: Commands,
    mut query: Query<(&Block, &mut Transform)>,
    existing_children: Query<&Children>,
    indicators: Query<(), With<HoverIndicator>>,
    config: Res<Config>,
) {
    if existing_children
        .get(over.entity)
        .is_ok_and(|children| children.iter().any(|child| indicators.contains(child)))
    {
        return;
    }
    let (block, mut tf) = query
        .get_mut(over.entity)
        .expect("entity should have Block and Transform components");
    let indicator_color = block.color.rotate_hue(180.);
    tf.translation.z = 2.;
    let occupied_cells = block.shape.occupied_cells(default()).collect::<Vec<_>>();
    let border_size = config.block_size * config.render.border_scale;
    let highlight_radius = border_size * config.render.highlight_scale;
    for occupied_cell in occupied_cells.iter() {
        commands.spawn((
            BlockVisual,
            HoverIndicator,
            ChildOf(over.entity),
            ShapeBundle::rect(
                &ShapeConfig {
                    transform: Transform::from_translation(Vec3::new(
                        config.block_size * occupied_cell.x as f32,
                        config.block_size * occupied_cell.y as f32,
                        -0.5,
                    )),
                    color: indicator_color,
                    corner_radii: get_corner_radii(occupied_cell, &occupied_cells)
                        * highlight_radius,
                    ..ShapeConfig::default_2d()
                },
                Vec2::splat(config.block_size + highlight_radius),
            ),
        ));
    }
}
