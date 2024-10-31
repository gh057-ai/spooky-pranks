use bevy::{prelude::*, window::PrimaryWindow};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Spooky Pranks!".into(),
                resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15))) // Dark background
        .add_systems(Startup, setup)
        .add_systems(Update, (
            follow_mouse,
            cursor_position_system,
        ))
        .init_resource::<CursorPosition>()
        .run();
}

#[derive(Resource, Default)]
struct CursorPosition {
    position: Vec2,
}

#[derive(Component)]
struct Ghost;

fn setup(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Ghost (temporarily using a sprite shape until we have assets)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.8, 0.8, 0.8, 0.8),
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..default()
        },
        Ghost,
    ));
}

fn cursor_position_system(
    mut cursor_position: ResMut<CursorPosition>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();
    
    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        cursor_position.position = world_position;
    }
}

fn follow_mouse(
    cursor_position: Res<CursorPosition>,
    mut ghost_query: Query<&mut Transform, With<Ghost>>,
    time: Res<Time>,
) {
    if let Ok(mut ghost_transform) = ghost_query.get_single_mut() {
        let target = cursor_position.position.extend(ghost_transform.translation.z);
        let current = ghost_transform.translation;
        
        // Smooth following
        let new_pos = current.lerp(target, time.delta_seconds() * 10.0);
        ghost_transform.translation = new_pos;
    }
}
