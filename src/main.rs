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
        .insert_resource(TrailSettings {
            spawn_timer: Timer::from_seconds(0.05, TimerMode::Repeating),
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                spawn_ghost_trail.after(GameSet::FollowMouse),
                update_ghost_trail,
                cursor_position_system.in_set(GameSet::CursorPositionSystem),
                follow_mouse.in_set(GameSet::FollowMouse),
                float_ghost.in_set(GameSet::FloatGhost),
                fade_ghost.in_set(GameSet::FadeGhost),
                exit_system.in_set(GameSet::ExitSystem),
            ),
        )
        .init_resource::<CursorPosition>()
        .insert_resource(PlayerInventory {
            candies: 0,
            rare_items: Vec::new(),
        })
        .add_systems(Startup, spawn_houses)
        .add_systems(
            Update,
            (
                ghost_house_interaction,
                animate_floating_text,
            ),
        )
        .run();
}

#[derive(Resource, Default)]
struct CursorPosition {
    position: Vec2,
}

#[derive(Component)]
enum GhostState {
    Normal,
    Faded,
    // Could add more states like Invisible, Attacking, etc.
}

#[derive(Component)]
struct Ghost {
    speed: f32,
    rotation_speed: f32,
    state: GhostState,  // Add state to Ghost component
}

#[derive(Component)]
struct FloatingAnimation {
    original_y: f32,
    amplitude: f32,    // How far it floats up/down
    frequency: f32,    // How fast it floats
}

#[derive(Component)]
struct FadeEffect {
    timer: Timer,
}

#[derive(Component)]
struct GhostTrail {
    lifetime: Timer,
}

#[derive(Resource)]
struct TrailSettings {
    spawn_timer: Timer,
}

fn ease_out_cubic(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(3)
}

#[derive(Component)]
struct House {
    light_status: bool,
    loot_type: LootType,
    interaction_timer: Timer,
}

#[allow(dead_code)]
#[derive(Component)]
struct Collectable {
    item_type: LootType,
    value: u32,
}

#[derive(Resource)]
struct PlayerInventory {
    candies: u32,
    rare_items: Vec<LootType>,
}

#[derive(Clone, PartialEq, Debug)]
enum LootType {
    Candy,
    RareItem(String), // e.g., "Ancient Spellbook", "Magic Crystal"
    SpecialTreat(String), // e.g., "Homemade Cookies", "Golden Chocolate"
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
        Ghost { 
            speed: 10.0,
            rotation_speed: 5.0,
            state: GhostState::Normal,
        },
        FloatingAnimation { 
            original_y: 0.0,
            amplitude: 10.0,
            frequency: 2.0,
        },
        FadeEffect {
            timer: Timer::from_seconds(3.0, TimerMode::Repeating),
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
        let current = Vec3::new(
            ghost_transform.translation.x,
            anim.original_y,
            ghost_transform.translation.z
        );
        
        let direction = target - current;
        
        if direction.length() > 0.1 {
            let target_rotation = Quat::from_rotation_z(direction.y.atan2(direction.x) + std::f32::consts::FRAC_PI_2);
            let rotation_t = ease_out_cubic(time.delta_seconds() * ghost.rotation_speed);
            ghost_transform.rotation = ghost_transform.rotation.slerp(target_rotation, rotation_t);
            
            let speed_factor = (direction.length() * 0.01).min(1.0);
            let scale = 0.2 * (1.0 + speed_factor * 0.1);
            ghost_transform.scale = Vec3::splat(scale);
        }
        
        let movement_t = ease_out_cubic(time.delta_seconds() * ghost.speed);
        let new_pos = current.lerp(target, movement_t);
        ghost_transform.translation.x = new_pos.x;
        anim.original_y = new_pos.y;
    }
}

fn float_ghost(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &FloatingAnimation)>,
) {
    for (mut transform, anim) in query.iter_mut() {
        // Combine two sine waves for more organic movement
        let primary_wave = (time.elapsed_seconds() * anim.frequency).sin() * anim.amplitude;
        let secondary_wave = (time.elapsed_seconds() * (anim.frequency * 2.5)).sin() * (anim.amplitude * 0.3);
        transform.translation.y = anim.original_y + primary_wave + secondary_wave;
    }
}

fn fade_ghost(
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(&mut Handle<Image>, &mut FadeEffect, &mut Ghost)>,
) {
    for (mut texture, mut fade, mut ghost) in query.iter_mut() {
        fade.timer.tick(time.delta());
        
        if fade.timer.just_finished() {
            match ghost.state {
                GhostState::Normal => {
                    ghost.state = GhostState::Faded;
                    *texture = asset_server.load("sprites/ghost_faded.png");
                }
                GhostState::Faded => {
                    ghost.state = GhostState::Normal;
                    *texture = asset_server.load("sprites/ghost.png");
                }
            }
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

fn spawn_ghost_trail(
    mut commands: Commands,
    time: Res<Time>,
    mut trail_settings: ResMut<TrailSettings>,
    ghost_query: Query<(&Transform, &Sprite), With<Ghost>>,
) {
    trail_settings.spawn_timer.tick(time.delta());

    if trail_settings.spawn_timer.just_finished() {
        if let Ok((ghost_transform, ghost_sprite)) = ghost_query.get_single() {
            // Randomize trail scale and rotation slightly
            let random_scale = 0.95 + (rand::random::<f32>() * 0.1);
            let random_rotation = ghost_transform.rotation * Quat::from_rotation_z(rand::random::<f32>() * 0.1 - 0.05);
            
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgba(1.0, 1.0, 1.0, 0.8),
                        ..ghost_sprite.clone()
                    },
                    transform: Transform {
                        translation: ghost_transform.translation,
                        rotation: random_rotation,
                        scale: ghost_transform.scale * random_scale,
                    },
                    ..default()
                },
                GhostTrail {
                    lifetime: Timer::from_seconds(0.8, TimerMode::Once),
                },
            ));
        }
    }
}

fn update_ghost_trail(
    mut commands: Commands,
    time: Res<Time>,
    mut trail_query: Query<(Entity, &mut Sprite, &mut GhostTrail)>,
) {
    for (entity, mut sprite, mut trail) in trail_query.iter_mut() {
        trail.lifetime.tick(time.delta());
        
        // Fade out the trail using the timer's fraction
        let alpha = 1.0 - trail.lifetime.fraction();
        sprite.color = sprite.color.with_alpha(alpha);
        
        if trail.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_houses(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Spawn multiple houses with different states and loot
    let house_positions = [
        (Vec2::new(300.0, 200.0), true),  // Light on
        (Vec2::new(-300.0, 200.0), false), // Light off
        (Vec2::new(0.0, -200.0), true),    // Light on
    ];

    for (pos, light_status) in house_positions {
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load(if light_status { 
                    "sprites/house_lit.png" 
                } else { 
                    "sprites/house_dark.png" 
                }),
                transform: Transform::from_xyz(pos.x, pos.y, 0.0),
                ..default()
            },
            House {
                light_status,
                loot_type: match rand::random::<f32>() {
                    x if x < 0.2 => LootType::RareItem("Magic Crystal".to_string()),
                    x if x < 0.3 => LootType::SpecialTreat("Homemade Cookies".to_string()),
                    _ => LootType::Candy,
                },
                interaction_timer: Timer::from_seconds(3.0, TimerMode::Once),
            },
        ));
    }
}

fn ghost_house_interaction(
    mut commands: Commands,
    mut ghost_query: Query<(&Transform, &mut Ghost)>,
    mut houses_query: Query<(&Transform, &mut House)>,
    time: Res<Time>,
    mut inventory: ResMut<PlayerInventory>,
) {
    let ghost_range = 50.0; // Interaction range

    if let Ok((ghost_transform, _)) = ghost_query.get_single_mut() {
        for (house_transform, mut house) in houses_query.iter_mut() {
            if !house.light_status {
                continue; // Skip dark houses
            }

            let distance = ghost_transform.translation.distance(house_transform.translation);
            
            if distance < ghost_range {
                house.interaction_timer.tick(time.delta());

                if house.interaction_timer.just_finished() {
                    // Collect loot
                    match &house.loot_type {
                        LootType::Candy => {
                            inventory.candies += 1;
                            // Spawn floating text or particle effect
                            spawn_floating_text(&mut commands, house_transform.translation, "+1 Candy");
                        },
                        LootType::RareItem(item) => {
                            inventory.rare_items.push(LootType::RareItem(item.clone()));
                            spawn_floating_text(&mut commands, house_transform.translation, 
                                &format!("Rare Item: {}", item));
                        },
                        LootType::SpecialTreat(treat) => {
                            inventory.rare_items.push(LootType::SpecialTreat(treat.clone()));
                            spawn_floating_text(&mut commands, house_transform.translation, 
                                &format!("Special: {}", treat));
                        },
                    }
                    
                    // Reset house loot
                    house.loot_type = LootType::Candy;
                    house.interaction_timer.reset();
                }
            }
        }
    }
}

fn spawn_floating_text(
    commands: &mut Commands,
    position: Vec3,
    text: &str,
) {
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                text,
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_xyz(position.x, position.y + 30.0, 10.0),
            ..default()
        },
        FloatingText {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            initial_position: position,
        },
    ));
}

#[derive(Component)]
struct FloatingText {
    timer: Timer,
    initial_position: Vec3,
}

fn animate_floating_text(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Text, &mut FloatingText)>,
) {
    for (entity, mut transform, mut text, mut floating) in query.iter_mut() {
        floating.timer.tick(time.delta());
        
        // Float upward and fade out
        let progress = floating.timer.fraction();
        transform.translation.y = floating.initial_position.y + (50.0 * progress);
        
        let alpha = 1.0 - progress;
        if let Some(section) = text.sections.first_mut() {
            section.style.color = section.style.color.with_alpha(alpha);
        }

        if floating.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
