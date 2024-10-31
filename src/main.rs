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
            float_ghost,
        ))
        .init_resource::<CursorPosition>()
        .run();
}

#[derive(Resource, Default)]
struct CursorPosition {
    position: Vec2,
}

#[derive(Component)]
struct Ghost {
    speed: f32,
}

#[derive(Component)]
struct FloatingAnimation {
    original_y: f32,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Ghost with actual sprite
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/ghost.png"),
            transform: Transform::from_xyz(0.0, 0.0, 1.0)
                .with_scale(Vec3::splat(0.2)), // Changes size of ghost from 2.0 to 0.2
            sprite: Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 0.8), // Slightly transparent
                ..default()
            },
            ..default()
        },
        Ghost { speed: 10.0 },
        FloatingAnimation {
            original_y: 0.0,
        },
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
    mut ghost_query: Query<(&Ghost, &mut Transform, &mut FloatingAnimation)>,
    time: Res<Time>,
) {
    if let Ok((ghost, mut ghost_transform, mut anim)) = ghost_query.get_single_mut() {
        let target = cursor_position.position.extend(ghost_transform.translation.z);
        let current = ghost_transform.translation;
        
        let new_pos = current.lerp(target, time.delta_seconds() * ghost.speed);
        ghost_transform.translation = new_pos;
        anim.original_y = new_pos.y;
    }
}

fn float_ghost(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &FloatingAnimation)>,
) {
    for (mut transform, anim) in query.iter_mut() {
        let offset = (time.elapsed_seconds() * 2.0).sin() * 10.0;
        transform.translation.y = anim.original_y + offset;
    }
}
