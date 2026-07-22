use std::f32::consts::FRAC_PI_4;
use std::f32::consts::{FRAC_PI_2, PI};

use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::ecs::relationship::Relationship;
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
        let occupied_cells = block
            .shape
            .occupied_cells(CellCoordinate::default())
            .collect::<Vec<_>>();
        // Render cell "bodies"
        for cell in occupied_cells.iter() {
            builder.spawn(ShapeBundle::rect(
                &ShapeConfig {
                    color: block.color,
                    thickness: 0.,
                    corner_radii: get_corner_radii(cell, &occupied_cells),
                    transform: Transform::from_translation(cell.center().extend(0.)),
                    ..ShapeConfig::default_2d()
                },
                Vec2::splat(CELL_SIZE), // - 2.),
            ));
        }
        let light = block.color.lighter(0.25);
        let dark = block.color.darker(0.1);

        for cell in occupied_cells.iter() {
            render_cell_boundaries(builder, cell, &occupied_cells, light, dark);
        }
    });
}

const CORNER_RADIUS: f32 = CELL_SIZE / 10.;

fn get_corner_radii(cell: &CellCoordinate, occupied_cells: &Vec<CellCoordinate>) -> Vec4 {
    let has_cell_up = occupied_cells.contains(&(*cell + (0, 1)));
    let has_cell_right = occupied_cells.contains(&(*cell + (1, 0)));
    let has_cell_down = occupied_cells.contains(&(*cell + (0, -1)));
    let has_cell_left = occupied_cells.contains(&(*cell + (-1, 0)));
    // start bottom right, clockwise
    let cr = |radius| if radius { CORNER_RADIUS } else { 0. };
    Vec4::new(
        cr(!(has_cell_down || has_cell_right)),
        cr(!(has_cell_down || has_cell_left)),
        cr(!(has_cell_up || has_cell_left)),
        cr(!(has_cell_up || has_cell_right)),
    )
}

const BOUNDARY_DISTANCE: f32 = (CELL_SIZE - CORNER_RADIUS) / 2.;

fn render_cell_boundaries<R: Relationship>(
    builder: &mut RelatedSpawnerCommands<R>,
    cell: &CellCoordinate,
    occupied_cells: &Vec<CellCoordinate>,
    light: Color,
    dark: Color,
) {
    let top = !occupied_cells.contains(&(*cell + (0, 1)));
    let bottom = !occupied_cells.contains(&(*cell + (0, -1)));
    let left = !occupied_cells.contains(&(*cell + (-1, 0)));
    let right = !occupied_cells.contains(&(*cell + (1, 0)));

    let shape_config = |color, (x, y)| ShapeConfig {
        color,
        thickness: 0.,
        transform: Transform::from_translation((cell.center() + Vec2::new(x, y)).extend(0.5)),
        cap: Cap::None,
        ..ShapeConfig::default_2d()
    };

    if top {
        builder.spawn(ShapeBundle::rect(
            &shape_config(light, (0., BOUNDARY_DISTANCE)),
            Vec2::new(CELL_SIZE - 2. * CORNER_RADIUS, CORNER_RADIUS),
        ));
    }
    if left {
        builder.spawn(ShapeBundle::rect(
            &shape_config(light, (-BOUNDARY_DISTANCE, 0.)),
            Vec2::new(CORNER_RADIUS, CELL_SIZE - 2. * CORNER_RADIUS),
        ));
    }
    if right {
        builder.spawn(ShapeBundle::rect(
            &shape_config(dark, (BOUNDARY_DISTANCE, 0.)),
            Vec2::new(CORNER_RADIUS, CELL_SIZE - 2. * CORNER_RADIUS),
        ));
    }
    if bottom {
        builder.spawn(ShapeBundle::rect(
            &shape_config(dark, (0., -BOUNDARY_DISTANCE)),
            Vec2::new(CELL_SIZE - 2. * CORNER_RADIUS, CORNER_RADIUS),
        ));
    }

    if top && left {
        builder.spawn(ShapeBundle::arc(
            &shape_config(
                light,
                (
                    -CELL_SIZE / 2. + CORNER_RADIUS,
                    CELL_SIZE / 2. - CORNER_RADIUS,
                ),
            ),
            CORNER_RADIUS,
            -FRAC_PI_2,
            0.,
        ));
    }
    if bottom && right {
        builder.spawn(ShapeBundle::arc(
            &shape_config(
                dark,
                (
                    CELL_SIZE / 2. - CORNER_RADIUS,
                    -CELL_SIZE / 2. + CORNER_RADIUS,
                ),
            ),
            CORNER_RADIUS,
            FRAC_PI_2,
            PI,
        ));
    }

    if top ^ left {
        builder.spawn(ShapeBundle::rect(
            &shape_config(light, (-BOUNDARY_DISTANCE, BOUNDARY_DISTANCE)),
            Vec2::splat(CORNER_RADIUS),
        ));
    }
    if bottom ^ right {
        builder.spawn(ShapeBundle::rect(
            &shape_config(dark, (BOUNDARY_DISTANCE, -BOUNDARY_DISTANCE)),
            Vec2::splat(CORNER_RADIUS),
        ));
    }

    if top ^ right {
        builder.spawn(ShapeBundle::rect(
            &shape_config(
                if top { light } else { dark },
                (BOUNDARY_DISTANCE, BOUNDARY_DISTANCE),
            ),
            Vec2::splat(CORNER_RADIUS),
        ));
    }
    if left ^ bottom {
        builder.spawn(ShapeBundle::rect(
            &shape_config(
                if left { light } else { dark },
                (-BOUNDARY_DISTANCE, -BOUNDARY_DISTANCE),
            ),
            Vec2::splat(CORNER_RADIUS),
        ));
    }

    if top && right {
        let top_right = (
            CELL_SIZE / 2. - CORNER_RADIUS,
            CELL_SIZE / 2. - CORNER_RADIUS,
        );
        builder.spawn(ShapeBundle::arc(
            &shape_config(light, top_right),
            CORNER_RADIUS,
            0.,
            FRAC_PI_4,
        ));
        builder.spawn(ShapeBundle::arc(
            &shape_config(dark, top_right),
            CORNER_RADIUS,
            FRAC_PI_4,
            FRAC_PI_2,
        ));
    }
    if bottom && left {
        let bottom_left = (
            -CELL_SIZE / 2. + CORNER_RADIUS,
            -CELL_SIZE / 2. + CORNER_RADIUS,
        );
        builder.spawn(ShapeBundle::arc(
            &shape_config(dark, bottom_left),
            CORNER_RADIUS,
            PI,
            PI + FRAC_PI_4,
        ));
        builder.spawn(ShapeBundle::arc(
            &shape_config(light, bottom_left),
            CORNER_RADIUS,
            PI + FRAC_PI_4,
            3. * FRAC_PI_2,
        ));
    }

    if !(left || top || occupied_cells.contains(&(*cell + (-1, 1)))) {
        builder.spawn(ShapeBundle::arc(
            &shape_config(light, (-CELL_SIZE / 2., CELL_SIZE / 2.)),
            CORNER_RADIUS,
            FRAC_PI_2,
            PI,
        ));
    }
    if !(right || bottom || occupied_cells.contains(&(*cell + (1, -1)))) {
        builder.spawn(ShapeBundle::arc(
            &shape_config(dark, (CELL_SIZE / 2., -CELL_SIZE / 2.)),
            CORNER_RADIUS,
            -FRAC_PI_2,
            0.,
        ));
    }

    if !(top || right || occupied_cells.contains(&(*cell + (1, 1)))) {
        let top_right = (CELL_SIZE / 2., CELL_SIZE / 2.);
        builder.spawn(ShapeBundle::arc(
            &shape_config(light, top_right),
            CORNER_RADIUS,
            PI,
            PI + FRAC_PI_4,
        ));
        builder.spawn(ShapeBundle::arc(
            &shape_config(dark, top_right),
            CORNER_RADIUS,
            PI + FRAC_PI_4,
            3. * FRAC_PI_2,
        ));
    }
    if !(left || bottom || occupied_cells.contains(&(*cell + (-1, -1)))) {
        let bottom_left = (-CELL_SIZE / 2., -CELL_SIZE / 2.);
        builder.spawn(ShapeBundle::arc(
            &shape_config(dark, bottom_left),
            CORNER_RADIUS,
            0.,
            FRAC_PI_4,
        ));
        builder.spawn(ShapeBundle::arc(
            &shape_config(light, bottom_left),
            CORNER_RADIUS,
            FRAC_PI_4,
            FRAC_PI_2,
        ));
    }
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
    tf.translation.z = 2.;
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
