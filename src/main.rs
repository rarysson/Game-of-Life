use std::time::Duration;

use bevy::{
    prelude::*,
    render::{
        settings::{Backends, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    window::PrimaryWindow,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .init_resource::<MousePosition>()
        .add_plugins(Defaults)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                track_mouse_system,
                place_tile_system,
                move_tiles_system,
                button_system,
            ),
        )
        .run();
}

pub struct Defaults;

#[derive(Resource, Default)]
struct MousePosition(Vec2);

#[derive(Resource)]
struct Config {
    running: bool,
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct CursorIndicator;

#[derive(Component)]
struct Tile;

#[derive(Component)]
struct TileTime {
    timer: Timer,
}

#[derive(Component)]
struct GridLine;

const CELL_SIZE: f32 = 8.0;
const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);

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

fn setup(mut cmds: Commands, asset_server: Res<AssetServer>) {
    cmds.spawn((Camera2dBundle::default(), MainCamera));
    cmds.spawn(TileTime {
        timer: Timer::new(Duration::from_secs_f32(0.5), TimerMode::Repeating),
    });
    cmds.insert_resource(Config { running: false });

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
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                border_color: BorderColor(Color::BLACK),
                background_color: NORMAL_BUTTON.into(),
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
    mut mouse_position: ResMut<MousePosition>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut q_indicator: Query<&mut Transform, With<CursorIndicator>>,
) {
    let (camera, camera_transform) = q_camera.single();

    let window = q_window.single();
    let mut indicator = q_indicator.single_mut();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mouse_position.0 = world_position;
        indicator.translation = Vec3::new(world_position.x, world_position.y, 0.0)
    }
}

fn place_tile_system(
    mouse_position: Res<MousePosition>,
    buttons: Res<Input<MouseButton>>,
    mut cmds: Commands,
    config: Res<Config>,
) {
    if config.running {
        return;
    }

    if buttons.just_pressed(MouseButton::Left) {
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

        cmds.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(
                    ((x / CELL_SIZE) as i32 * (CELL_SIZE as i32) + get_center_offset(x)) as f32,
                    ((y / CELL_SIZE) as i32 * (CELL_SIZE as i32) + get_center_offset(y)) as f32,
                    0.0,
                )),
                ..default()
            },
            Tile,
        ));
    }
}

fn move_tiles_system(
    mut q_tiles: Query<(Entity, &mut Transform), With<Tile>>,
    mut q_timer: Query<&mut TileTime>,
    time: Res<Time>,
    mut cmds: Commands,
    config: Res<Config>,
) {
    if !config.running {
        return;
    }

    let mut tile_timer = q_timer.single_mut();

    tile_timer.timer.tick(time.delta());

    if tile_timer.timer.finished() {
        for (e, mut transform) in q_tiles.iter_mut() {
            let half_width = WINDOW_WIDTH / 2;
            if (transform.translation.x as i32) < half_width - (CELL_SIZE as i32) {
                transform.translation += Vec3::new(CELL_SIZE, 0.0, 0.0);
            } else {
                cmds.entity(e).despawn();
            }
        }
    }
}

fn button_system(
    mut interaction_query: Query<(Entity, &Interaction)>,
    q_tile_lines: Query<(Entity, &GridLine)>,
    mut cmds: Commands,
    mut config: ResMut<Config>,
) {
    for (e, interaction) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                cmds.entity(e).despawn_recursive();
                config.running = true;

                for (e_line, _) in q_tile_lines.iter() {
                    cmds.entity(e_line).despawn();
                }
            }
            _ => (),
        }
    }
}
