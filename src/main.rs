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
        .add_systems(Update, (my_cursor_system, place_tile_system))
        .run();
}

pub struct Defaults;

#[derive(Resource, Default)]
struct MousePosition(Vec2);

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct CursorIndicator;

const CELL_SIZE: f32 = 8.0;
const CELL_SIZE_INT: i32 = CELL_SIZE as i32;

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

fn setup(mut cmds: Commands) {
    cmds.spawn((Camera2dBundle::default(), MainCamera));

    let h_bars = 1280 / CELL_SIZE_INT;
    for i in 0..=h_bars {
        cmds.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(1.0, 720.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                (i as f32) * CELL_SIZE - ((h_bars as f32 / 2.0) * CELL_SIZE),
                0.0,
                0.0,
            )),
            ..default()
        });
    }

    let v_bars = 720 / CELL_SIZE_INT;
    for i in 0..=v_bars {
        cmds.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(1280.0, 1.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                0.0,
                (i as f32) * CELL_SIZE - ((v_bars as f32 / 2.0) * CELL_SIZE),
                0.0,
            )),
            ..default()
        });
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
}

fn my_cursor_system(
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
) {
    if buttons.just_pressed(MouseButton::Left) {
        let get_center_offset = |position| {
            if position > 0.0 {
                CELL_SIZE_INT / 2
            } else {
                -CELL_SIZE_INT / 2
            }
        };

        cmds.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::PURPLE,
                custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                ((mouse_position.0.x / CELL_SIZE) as i32 * CELL_SIZE_INT
                    + get_center_offset(mouse_position.0.x)) as f32,
                ((mouse_position.0.y / CELL_SIZE) as i32 * CELL_SIZE_INT
                    + get_center_offset(mouse_position.0.y)) as f32,
                0.0,
            )),
            ..default()
        });
    }
}
