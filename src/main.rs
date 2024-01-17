use std::{collections::HashMap, time::Duration};

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{
        settings::{Backends, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    window::PrimaryWindow,
};

struct Defaults;

impl Plugin for Defaults {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                backends: Some(Backends::VULKAN),
                ..default()
            }),
        }));
    }
}

fn main() {
    App::new()
        .add_plugins(Defaults)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                track_mouse_system,
                place_tile_system,
                population_system,
                start_game_system,
            ),
        )
        .run();
}

struct CellData {
    alive: bool,
    entity: Entity,
}

#[derive(Resource, Default)]
struct MousePosition(Vec2);

#[derive(Resource)]
struct GameState {
    running: bool,
}

#[derive(Resource)]
struct Grid {
    cells: HashMap<String, CellData>,
}

#[derive(Resource)]
struct PopulationTimer {
    timer: Timer,
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct CursorIndicator;

#[derive(Component)]
struct Cell;

#[derive(Component)]
struct GridLine;

const CELL_SIZE: f32 = 8.0;
const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;

fn setup(mut cmds: Commands, asset_server: Res<AssetServer>) {
    cmds.init_resource::<MousePosition>();

    cmds.insert_resource(GameState { running: false });
    cmds.insert_resource(Grid {
        cells: HashMap::new(),
    });
    cmds.insert_resource(PopulationTimer {
        timer: Timer::new(Duration::from_secs_f32(0.15), TimerMode::Repeating),
    });

    cmds.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::WHITE),
            },
            ..Default::default()
        },
        MainCamera,
    ));

    let h_bars = WINDOW_WIDTH / (CELL_SIZE as i32);
    for i in 0..=h_bars {
        cmds.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(1.0, WINDOW_HEIGHT as f32)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(
                    (i as f32) * CELL_SIZE - ((h_bars as f32 / 2.0) * CELL_SIZE),
                    0.0,
                    0.0,
                )),
                ..default()
            },
            GridLine,
        ));
    }

    let v_bars = WINDOW_HEIGHT / (CELL_SIZE as i32);
    for i in 0..=v_bars {
        cmds.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(WINDOW_WIDTH as f32, 1.0)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    (i as f32) * CELL_SIZE - ((v_bars as f32 / 2.0) * CELL_SIZE),
                    0.0,
                )),
                ..default()
            },
            GridLine,
        ));
    }

    cmds.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                ..default()
            },
            ..default()
        },
        CursorIndicator,
    ));

    cmds.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::End,
            justify_content: JustifyContent::End,
            ..default()
        },
        ..default()
    })
    .with_children(|parent| {
        parent
            .spawn(ButtonBundle {
                style: Style {
                    width: Val::Px(150.0),
                    height: Val::Px(65.0),
                    border: UiRect::all(Val::Px(5.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                border_color: BorderColor(Color::BLACK),
                background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Start",
                    TextStyle {
                        font: asset_server.load("fonts/Roboto.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ));
            });
    });
}

fn track_mouse_system(
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut q_cursor_indicator: Query<&mut Transform, With<CursorIndicator>>,
    mut mouse_position: ResMut<MousePosition>,
    game_state: Res<GameState>,
) {
    if game_state.running {
        return;
    }

    let (camera, camera_transform) = q_camera.single();

    let window = q_window.single();
    let mut cursor_indicator = q_cursor_indicator.single_mut();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mouse_position.0 = world_position;
        cursor_indicator.translation = Vec3::new(world_position.x, world_position.y, 0.0)
    }
}

fn place_tile_system(
    mut cmds: Commands,
    mouse_position: Res<MousePosition>,
    btn: Res<Input<MouseButton>>,
    game_state: Res<GameState>,
    mut grid: ResMut<Grid>,
) {
    if game_state.running {
        return;
    }

    if btn.just_pressed(MouseButton::Left) {
        let x = mouse_position.0.x;
        let y = mouse_position.0.y;

        let half_width = WINDOW_WIDTH / 2;
        if (x as i32) < -half_width || (x as i32) > half_width {
            return;
        }

        let half_height = WINDOW_HEIGHT / 2;
        if (y as i32) < -half_height || (y as i32) > half_height {
            return;
        }

        let get_center_offset = |position| {
            if position > 0.0 {
                (CELL_SIZE as i32) / 2
            } else {
                -(CELL_SIZE as i32) / 2
            }
        };

        let x = ((x / CELL_SIZE) as i32 * (CELL_SIZE as i32) + get_center_offset(x)) as f32;
        let y = ((y / CELL_SIZE) as i32 * (CELL_SIZE as i32) + get_center_offset(y)) as f32;

        let entity = cmds
            .spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::BLACK,
                        custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
                    ..default()
                },
                Cell,
            ))
            .id();

        grid.cells.insert(
            format!("{}:{}", x, y),
            CellData {
                alive: true,
                entity,
            },
        );
    }
}

fn population_system(
    mut cmds: Commands,
    mut q_cells: Query<(&Transform, &mut Visibility), With<Cell>>,
    clock: Res<Time>,
    game_state: Res<GameState>,
    mut grid: ResMut<Grid>,
    mut population_timer: ResMut<PopulationTimer>,
) {
    if !game_state.running {
        return;
    }

    population_timer.timer.tick(clock.delta());

    if population_timer.timer.finished() {
        let mut alive_cells = HashMap::new();

        for (key, cell) in &grid.cells {
            if cell.alive {
                alive_cells.insert(
                    key.clone(),
                    CellData {
                        alive: cell.alive,
                        entity: cell.entity,
                    },
                );
            }
        }

        for (cell_transform, mut cell_visibility) in &mut q_cells {
            let neighbors = count_cell_neighbors(&cell_transform.translation, &alive_cells);
            let key = format!(
                "{}:{}",
                cell_transform.translation.x, cell_transform.translation.y
            );

            if neighbors < 2 || neighbors > 3 {
                if let Some(cell) = grid.cells.get_mut(&key) {
                    cell.alive = false;
                    *cell_visibility = Visibility::Hidden;
                }
            }

            let dead_cells = get_dead_cells(&cell_transform.translation, &alive_cells);

            for dead_cell_position in &dead_cells {
                let neighbors = count_cell_neighbors(dead_cell_position, &alive_cells);
                let key = format!("{}:{}", dead_cell_position.x, dead_cell_position.y);

                if neighbors == 3 {
                    if let Some(cell) = grid.cells.get_mut(&key) {
                        cell.alive = true;
                        cmds.entity(cell.entity).insert(Visibility::Visible);
                    } else {
                        let entity = cmds
                            .spawn((
                                SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::BLACK,
                                        custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                                        ..default()
                                    },
                                    transform: Transform::from_translation(Vec3::new(
                                        dead_cell_position.x,
                                        dead_cell_position.y,
                                        0.0,
                                    )),
                                    ..default()
                                },
                                Cell,
                            ))
                            .id();

                        grid.cells.insert(
                            key,
                            CellData {
                                alive: true,
                                entity,
                            },
                        );
                    }
                }
            }
        }
    }
}

fn count_cell_neighbors(cell_position: &Vec3, alive_cells: &HashMap<String, CellData>) -> i32 {
    let Vec3 { x, y, .. } = cell_position;
    let is_cell_alive = |x, y| {
        if let Some(cell) = alive_cells.get(&format!("{}:{}", x, y)) {
            cell.alive
        } else {
            false
        }
    };

    let n = is_cell_alive(*x, *y + CELL_SIZE) as i32;
    let s = is_cell_alive(*x, *y - CELL_SIZE) as i32;
    let w = is_cell_alive(*x - CELL_SIZE, *y) as i32;
    let e = is_cell_alive(*x + CELL_SIZE, *y) as i32;
    let ne = is_cell_alive(*x + CELL_SIZE, *y + CELL_SIZE) as i32;
    let nw = is_cell_alive(*x - CELL_SIZE, *y + CELL_SIZE) as i32;
    let se = is_cell_alive(*x + CELL_SIZE, *y - CELL_SIZE) as i32;
    let sw = is_cell_alive(*x - CELL_SIZE, *y - CELL_SIZE) as i32;

    n + s + w + e + ne + nw + se + sw
}

fn get_dead_cells(cell_position: &Vec3, alive_cells: &HashMap<String, CellData>) -> Vec<Vec3> {
    let Vec3 { x, y, z } = cell_position;
    let is_cell_alive = |x, y| {
        if let Some(cell) = alive_cells.get(&format!("{}:{}", x, y)) {
            cell.alive
        } else {
            false
        }
    };

    let neighbors = vec![
        Vec3::new(*x, *y + CELL_SIZE, *z),
        Vec3::new(*x, *y - CELL_SIZE, *z),
        Vec3::new(*x - CELL_SIZE, *y, *z),
        Vec3::new(*x + CELL_SIZE, *y, *z),
        Vec3::new(*x + CELL_SIZE, *y + CELL_SIZE, *z),
        Vec3::new(*x - CELL_SIZE, *y + CELL_SIZE, *z),
        Vec3::new(*x + CELL_SIZE, *y - CELL_SIZE, *z),
        Vec3::new(*x - CELL_SIZE, *y - CELL_SIZE, *z),
    ];

    neighbors
        .into_iter()
        .filter(|n| !is_cell_alive(n.x, n.y))
        .collect()
}

fn start_game_system(
    mut cmds: Commands,
    q_grid_lines: Query<Entity, With<GridLine>>,
    q_cursor_indicator: Query<Entity, With<CursorIndicator>>,
    mut q_interaction: Query<(Entity, &Interaction)>,
    mut game_state: ResMut<GameState>,
) {
    for (entity, interaction) in &mut q_interaction {
        match *interaction {
            Interaction::Pressed => {
                cmds.entity(entity).despawn_recursive();

                for e_line in q_grid_lines.iter() {
                    cmds.entity(e_line).despawn();
                }

                cmds.entity(q_cursor_indicator.single()).despawn();

                game_state.running = true;
            }
            _ => (),
        }
    }
}
