use bevy::{
    prelude::*,
    window::PrimaryWindow,
    app::AppExit,
    input::keyboard::KeyCode,
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum GameSet {
    FollowMouse,
    CursorPositionSystem,
    FloatGhost,
    FadeGhost,
    ExitSystem,
}

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
        .add_systems(
            Update,
            (
                cursor_position_system.in_set(GameSet::CursorPositionSystem),
                follow_mouse.in_set(GameSet::FollowMouse),
                float_ghost.in_set(GameSet::FloatGhost),
                fade_ghost.in_set(GameSet::FadeGhost),
                exit_system.in_set(GameSet::ExitSystem),
            ),
        )
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

#[derive(Component)]
struct FadeEffect {
    timer: Timer,
    is_faded: bool,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/ghost.png"),
            transform: Transform::from_xyz(0.0, 0.0, 1.0)
                .with_scale(Vec3::splat(0.2)),
            sprite: Sprite {
                color: Color::WHITE,
                ..default()
            },
            ..default()
        },
        Ghost { speed: 10.0 },
        FloatingAnimation { original_y: 0.0 },
        FadeEffect {
            timer: Timer::from_seconds(3.0, TimerMode::Repeating),
            is_faded: false,
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

fn fade_ghost(
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(&mut Handle<Image>, &mut FadeEffect)>,
) {
    for (mut texture, mut fade) in query.iter_mut() {
        fade.timer.tick(time.delta());
        
        if fade.timer.just_finished() {
            fade.is_faded = !fade.is_faded;
            *texture = if fade.is_faded {
                asset_server.load("sprites/ghost_faded.png")
            } else {
                asset_server.load("sprites/ghost.png")
            };
        }
    }
}

fn exit_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit::Success);
    }
}
