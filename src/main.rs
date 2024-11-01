use bevy::{
    prelude::*,
    window::PrimaryWindow,
    app::AppExit,
    input::keyboard::KeyCode,
    input::mouse::MouseButton,
};
use std::fs;
use serde::{Serialize, Deserialize};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum GameSet {
    FollowMouse,
    CursorPositionSystem,
    FloatGhost,
    FadeGhost,
    ExitSystem,
}

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
}

#[derive(Component)]
struct MenuUI;

// Update these type definitions
type BulletQuery<'a> = Query<'a, 'static, (Entity, &'static mut Transform, &'static Bullet)>;
type BalloonQuery<'a> = Query<'a, 'static, (Entity, &'static Transform), With<BalloonPumpkin>>;

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
                spawn_ghost_trail,
                update_ghost_trail,
                cursor_position_system.in_set(GameSet::CursorPositionSystem),
                follow_mouse.in_set(GameSet::FollowMouse),
                float_ghost.in_set(GameSet::FloatGhost),
                fade_ghost.in_set(GameSet::FadeGhost),
                exit_system.in_set(GameSet::ExitSystem),
                update_house_display,
                save_game,
                load_game,
                switch_house_lights,
                update_score_text,
                update_particles,
                pause_system,
                menu_system.run_if(in_state(GameState::Menu)),
                ghost_house_interaction.run_if(in_state(GameState::Playing)),
                candy_deposit_system,
                animate_progress_particles,
                bullet_system,
                shoot_balloon,
            )
                .run_if(not(in_state(GameState::Paused)))
                .chain(),
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
        .init_state::<GameState>()
        .insert_state(GameState::Menu)
        .add_systems(OnEnter(GameState::Menu), setup_menu)
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

#[derive(Component, Clone, Copy)]
enum HouseState {
    Lit,
    Dark,
}

#[derive(Component)]
enum HouseType {
    First,
}

#[derive(Component)]
struct House {
    state: HouseState,
    house_type: HouseType,
    light_status: bool,
    interaction_timer: Timer,
}

#[allow(dead_code)]
#[derive(Component)]
struct Collectable {
    item_type: LootType,
    value: u32,
}

#[derive(Resource, Serialize, Deserialize)]
struct PlayerInventory {
    candies: u32,
    rare_items: Vec<LootType>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
enum LootType {
    Candy,
    RareItem(String),
    SpecialTreat(String),
}

#[derive(Resource)]
struct HouseSprites {
    lit: Handle<Image>,
    dark: Handle<Image>,
}

#[derive(Component)]
struct BalloonPumpkin;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct Particle {
    velocity: Vec2,
    lifetime: Timer,
}

#[derive(Component)]
struct CandySack {
    capacity: u32,
    current: u32,
}

#[derive(Component)]
struct Pumpkin;  // Just use as a marker component

#[derive(Component)]
struct Bullet {
    speed: f32,
    direction: Vec2,
}

#[derive(Component)]
struct ProgressBar;

// Add this component to track if we've shown the message
#[derive(Component)]
struct FullSackMessage;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    // Update paths to match directory structure
    let house_sprites = HouseSprites {
        lit: asset_server.load("sprites/houses/house_lit.png"),
        dark: asset_server.load("sprites/houses/house_dark.png"),
    };
    commands.insert_resource(house_sprites);

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
        CandySack {
            capacity: 10,  // Can hold 10 candies before needing to deposit
            current: 0,
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

    commands.spawn((
        TextBundle::from_section(
            "Candies: 0",
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        ScoreText,
    ));

    // Add progress bar UI
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(30.0),
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        },
    )).with_children(|parent| {
        // Background bar
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Px(300.0),
                height: Val::Px(20.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: Color::srgb(0.2, 0.2, 0.2).into(),
            ..default()
        }).with_children(|parent| {
            // Progress fill
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: Color::srgb(0.8, 0.4, 0.0).into(),
                    ..default()
                },
                ProgressBar,
            ));
        });
    });
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
    // Grid configuration
    let rows = 3;
    let cols = 3;
    let spacing = 300.0; // Space between houses
    
    // Calculate starting position for top-left house
    // This centers the grid around (0,0)
    let start_x = -((cols - 1) as f32 * spacing) / 2.0;
    let start_y = -((rows - 1) as f32 * spacing) / 2.0;

    // Spawn houses in a grid
    for row in 0..rows {
        for col in 0..cols {
            // Skip the center position (for the pumpkin)
            if row == 1 && col == 1 {
                continue;
            }

            let x = start_x + (col as f32 * spacing);
            let y = start_y + (row as f32 * spacing);
            
            let light_status = rand::random::<bool>();
            
            // Debug print house spawn
            println!("Spawning house at ({}, {}), light status: {}", x, y, light_status);
            
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load(if light_status { 
                        "sprites/houses/house_lit.png" 
                    } else { 
                        "sprites/houses/house_dark.png" 
                    }),
                    transform: Transform::from_xyz(x, y, 0.0)
                        .with_scale(Vec3::splat(0.5)),
                    ..default()
                },
                House {
                    state: if light_status { HouseState::Lit } else { HouseState::Dark },
                    house_type: HouseType::First, // Simplified for testing
                    light_status,  // Make sure this is being set correctly
                    interaction_timer: Timer::from_seconds(3.0, TimerMode::Once),
                },
            ));
        }
    }

    // Spawn pumpkin in the center
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/pump_kin.png"),
            transform: Transform::from_xyz(0.0, 0.0, 0.0)
                .with_scale(Vec3::splat(0.4)),
            ..default()
        },
        Pumpkin,
    ));

    // Spawn balloon pumpkin in the center
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/balloon_pumpkin.png"),
            transform: Transform::from_xyz(0.0, 0.0, 0.0)
                .with_scale(Vec3::splat(0.4)),
            ..default()
        },
        BalloonPumpkin,
        FloatingAnimation {
            original_y: 0.0,
            amplitude: 15.0,    // How far it floats up/down
            frequency: 1.5,     // How fast it floats
        },
    ));
}

fn ghost_house_interaction(
    mut commands: Commands,
    mut ghost_query: Query<(&Transform, &mut Ghost, &mut CandySack)>,
    mut houses_query: Query<(&Transform, &mut House, &mut Sprite)>,
    mut inventory: ResMut<PlayerInventory>,
    message_query: Query<Entity, With<FullSackMessage>>, // Query to check if message exists
    time: Res<Time>,
) {
    let ghost_range = 100.0;

    if let Ok((ghost_transform, _, mut candy_sack)) = ghost_query.get_single_mut() {
        // Only show the message once when the sack becomes full and no message exists
        if candy_sack.current == candy_sack.capacity && message_query.is_empty() {
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section(
                        "Move to center pumpkin to deposit!",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    transform: Transform::from_xyz(0.0, 100.0, 10.0),
                    ..default()
                },
                FloatingText {
                    timer: Timer::from_seconds(1.0, TimerMode::Once),
                    initial_position: Vec3::new(0.0, 100.0, 10.0),
                },
                FullSackMessage,
            ));
        } else if candy_sack.current < candy_sack.capacity {
            // Remove the message if it exists and sack is no longer full
            for message_entity in message_query.iter() {
                commands.entity(message_entity).despawn_recursive();
            }
        }

        for (house_transform, mut house, mut sprite) in houses_query.iter_mut() {
            if !house.light_status {
                continue;
            }

            let distance = ghost_transform.translation.distance(house_transform.translation);
            
            if distance < ghost_range {
                // Visual feedback - house turns slightly green when in range
                sprite.color = Color::srgb(0.8, 1.0, 0.8);
                
                house.interaction_timer.tick(time.delta());
                
                // Debug print when timer is running
                if house.interaction_timer.fraction() > 0.0 {
                    println!("Trick or treating at house: {}%", house.interaction_timer.fraction() * 100.0);
                }

                if house.interaction_timer.just_finished() {
                    println!("Timer finished! Adding candy!");
                    candy_sack.current += 1;
                    inventory.candies += 1;
                    
                    // Spawn very visible text
                    spawn_floating_text(
                        &mut commands,
                        house_transform.translation,
                        &format!("Total Candies: {}", inventory.candies)
                    );
                    
                    // Reset timer
                    house.interaction_timer.reset();
                }
            } else {
                // Reset color when out of range
                sprite.color = Color::WHITE;
                house.interaction_timer.reset();
            }
        }
    }
}

fn spawn_floating_text(
    commands: &mut Commands,
    position: Vec3,
    text: &str,
) -> Entity {
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
    )).id()
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

fn update_house_display(
    mut house_query: Query<(&House, &mut Handle<Image>)>,
    house_sprites: Res<HouseSprites>,
) {
    for (house, mut sprite) in house_query.iter_mut() {
        let new_sprite = match (house.state, &house.house_type) {
            (HouseState::Lit, _) => house_sprites.lit.clone(),
            (HouseState::Dark, _) => house_sprites.dark.clone(),
        };
        *sprite = new_sprite;
    }
}

fn save_game(
    keyboard: Res<ButtonInput<KeyCode>>,
    inventory: Res<PlayerInventory>,
) {
    if keyboard.just_pressed(KeyCode::F5) {  // Save when F5 is pressed
        let save_data = serde_json::to_string(&*inventory).unwrap();
        fs::write("save_game.json", save_data).unwrap();
        println!("Game saved!");
    }
}

fn load_game(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<PlayerInventory>,
) {
    if keyboard.just_pressed(KeyCode::F9) {  // Load when F9 is pressed
        if let Ok(save_data) = fs::read_to_string("save_game.json") {
            if let Ok(loaded_inventory) = serde_json::from_str::<PlayerInventory>(&save_data) {
                *inventory = loaded_inventory;
                println!("Game loaded!");
            }
        }
    }
}

// Add a new system for light switching
fn switch_house_lights(
    time: Res<Time>,
    mut houses: Query<(&mut House, &mut Handle<Image>)>,
    house_sprites: Res<HouseSprites>,
) {
    // Switch lights every few seconds
    let switch_interval = 5.0; // Adjust this value to control frequency
    let time_since_startup = time.elapsed_seconds();
    
    if time_since_startup % switch_interval < time.delta_seconds() {
        // Randomly select houses to switch
        for (mut house, mut sprite) in houses.iter_mut() {
            if rand::random::<f32>() < 0.3 { // 30% chance to switch each house
                house.light_status = !house.light_status;
                house.state = if house.light_status { 
                    HouseState::Lit 
                } else { 
                    HouseState::Dark 
                };
                
                *sprite = if house.light_status {
                    house_sprites.lit.clone()
                } else {
                    house_sprites.dark.clone()
                };
            }
        }
    }
}

fn update_score_text(
    inventory: Res<PlayerInventory>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[0].value = format!("Candies: {}", inventory.candies);
    }
}

fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut Particle)>,
) {
    for (entity, mut transform, mut particle) in particles.iter_mut() {
        particle.lifetime.tick(time.delta());
        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
        } else {
            transform.translation.x += particle.velocity.x * time.delta_seconds();
            transform.translation.y += particle.velocity.y * time.delta_seconds();
        }
    }
}

fn setup_menu(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        },
        MenuUI,
    ));
}

fn menu_system(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    menu_ui: Query<Entity, With<MenuUI>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        // Remove menu UI
        for entity in menu_ui.iter() {
            commands.entity(entity).despawn_recursive();
        }
        game_state.set(GameState::Playing);
    }
}

fn pause_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        match current_state.get() {
            GameState::Playing => game_state.set(GameState::Paused),
            GameState::Paused => game_state.set(GameState::Playing),
            _ => (), // Do nothing if in menu state
        }
    }
}

fn candy_deposit_system(
    mut commands: Commands,
    mut ghost_query: Query<(&Transform, &mut CandySack)>,
    pumpkin_query: Query<&Transform, With<Pumpkin>>,
    mut progress_bar_query: Query<(&mut Style, &mut BackgroundColor), With<ProgressBar>>,
    message_query: Query<Entity, With<FullSackMessage>>,
) {
    let deposit_range = 100.0;

    if let (Ok((ghost_transform, mut candy_sack)), Ok(pumpkin_transform)) = 
        (ghost_query.get_single_mut(), pumpkin_query.get_single()) {
        
        let distance = ghost_transform.translation.distance(pumpkin_transform.translation);
        
        if distance < deposit_range && candy_sack.current > 0 {
            // Update progress bar (25% per full sack)
            if let Ok((mut style, mut background_color)) = progress_bar_query.get_single_mut() {
                let current_width = if let Val::Percent(width) = style.width {
                    width
                } else {
                    0.0
                };
                
                // Calculate progress increase (25% per full sack)
                let progress_increase = (candy_sack.current as f32 / candy_sack.capacity as f32) * 25.0;
                let new_width = (current_width + progress_increase).min(100.0);
                style.width = Val::Percent(new_width);
                
                // Change color when full
                if new_width >= 100.0 {
                    *background_color = Color::srgb(1.0, 0.5, 0.0).into();
                }
            }
            
            // Spawn deposit effect
            spawn_floating_text(
                &mut commands,
                pumpkin_transform.translation,
                &format!("Deposited {} candies!", candy_sack.current)
            );
            
            // Reset candy sack
            candy_sack.current = 0;
            
            // Remove full sack message if it exists
            for message_entity in message_query.iter() {
                commands.entity(message_entity).despawn_recursive();
            }
        }
    }
}

#[derive(Default)]
struct BurstConfig {
    count: i32,
    min_speed: f32,
    max_speed: f32,
    min_scale: f32,
    lifetime: f32,
    color: Color,
}

fn spawn_money_burst(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    config: BurstConfig,
) {
    for i in 0..config.count {
        let angle = (i as f32 / config.count as f32) * std::f32::consts::TAU;
        let speed = rand::random::<f32>() * (config.max_speed - config.min_speed) + config.min_speed;
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
        
        let spread = rand::random::<f32>() * 0.2 - 0.1;
        let particle_angle = angle + spread;
        
        let scale_variation = rand::random::<f32>() * 0.1;
        let scale = config.min_scale + scale_variation;
        
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("sprites/money_shot.png"),
                transform: Transform::from_xyz(position.x, position.y, 2.0)
                    .with_scale(Vec3::splat(scale))
                    .with_rotation(Quat::from_rotation_z(particle_angle)),
                sprite: Sprite {
                    color: config.color,
                    ..default()
                },
                ..default()
            },
            Particle {
                velocity,
                lifetime: Timer::from_seconds(config.lifetime, TimerMode::Once),
            },
        ));
    }
}

fn bullet_system(
    mut commands: Commands,
    mut bullets_and_balloons: ParamSet<(BulletQuery, BalloonQuery)>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
) {
    let balloon_pos = bullets_and_balloons.p1()
        .get_single()
        .ok()
        .map(|(entity, transform)| (entity, transform.translation));
    
    for (bullet_entity, mut transform, bullet) in bullets_and_balloons.p0().iter_mut() {
        // Move bullet
        transform.translation.x += bullet.direction.x * bullet.speed * time.delta_seconds();
        transform.translation.y += bullet.direction.y * bullet.speed * time.delta_seconds();

        // Check collision with balloon
        if let Some((balloon_entity, balloon_pos)) = balloon_pos {
            let distance = transform.translation.distance(balloon_pos);
            if distance < 50.0 {
                // Inner burst
                spawn_money_burst(&mut commands, &asset_server, balloon_pos, BurstConfig {
                    count: 12,
                    min_speed: 200.0,
                    max_speed: 300.0,
                    min_scale: 0.1,
                    lifetime: 0.5,
                    color: Color::srgb(1.0, 0.9, 0.3),
                });
                
                // Middle burst
                spawn_money_burst(&mut commands, &asset_server, balloon_pos, BurstConfig {
                    count: 8,
                    min_speed: 150.0,
                    max_speed: 250.0,
                    min_scale: 0.15,
                    lifetime: 0.7,
                    color: Color::srgb(1.0, 0.8, 0.0),
                });
                
                // Outer burst
                spawn_money_burst(&mut commands, &asset_server, balloon_pos, BurstConfig {
                    count: 6,
                    min_speed: 100.0,
                    max_speed: 200.0,
                    min_scale: 0.2,
                    lifetime: 1.0,
                    color: Color::srgb(0.9, 0.7, 0.0),
                });

                // Trailing particles
                for _ in 0..4 {
                    let angle = rand::random::<f32>() * std::f32::consts::TAU;
                    let speed = rand::random::<f32>() * 50.0 + 25.0;
                    let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
                    
                    commands.spawn((
                        SpriteBundle {
                            texture: asset_server.load("sprites/money_shot.png"),
                            transform: Transform::from_xyz(balloon_pos.x, balloon_pos.y, 2.0)
                                .with_scale(Vec3::splat(0.25))
                                .with_rotation(Quat::from_rotation_z(angle)),
                            sprite: Sprite {
                                color: Color::srgb(1.0, 0.6, 0.0),
                                ..default()
                            },
                            ..default()
                        },
                        Particle {
                            velocity,
                            lifetime: Timer::from_seconds(1.5, TimerMode::Once),
                        },
                    ));
                }

                // Spawn hit text with sparkle emoji
                spawn_floating_text(
                    &mut commands,
                    balloon_pos,
                    "JACKPOT! 💰✨"
                );

                commands.entity(bullet_entity).despawn();
                commands.entity(balloon_entity).despawn();
            }
        }

        // Despawn bullets that go off screen
        if transform.translation.length() > 1000.0 {
            commands.entity(bullet_entity).despawn();
        }
    }
}

fn animate_progress_particles(
    mut commands: Commands,
    _time: Res<Time>,
    progress_bar: Query<&Style, With<ProgressBar>>,
    asset_server: Res<AssetServer>,
) {
    if let Ok(style) = progress_bar.get_single() {
        if let Val::Percent(progress) = style.width {
            if progress >= 100.0 && rand::random::<f32>() < 0.1 {
                let x = rand::random::<f32>() * 800.0 - 400.0;
                let y = rand::random::<f32>() * 600.0 - 300.0;
                
                commands.spawn((
                    SpriteBundle {
                        texture: asset_server.load("sprites/sparkle.png"),
                        transform: Transform::from_xyz(x, y, 5.0)
                            .with_scale(Vec3::splat(0.2)),
                        sprite: Sprite {
                            color: Color::srgb(1.0, 0.9, 0.3),
                            ..default()
                        },
                        ..default()
                    },
                    Particle {
                        velocity: Vec2::new(
                            rand::random::<f32>() * 50.0 - 25.0,
                            rand::random::<f32>() * 50.0 - 25.0
                        ),
                        lifetime: Timer::from_seconds(1.0, TimerMode::Once),
                    },
                ));
            }
        }
    }
}

fn shoot_balloon(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorPosition>,
    ghost_query: Query<&Transform, With<Ghost>>,
    progress_bar_query: Query<&Style, With<ProgressBar>>,
) {
    // Check if progress bar is at 100%
    let can_shoot = progress_bar_query
        .get_single()
        .map(|style| {
            if let Val::Percent(progress) = style.width {
                progress >= 100.0
            } else {
                false
            }
        })
        .unwrap_or(false);

    // Only allow shooting if progress bar is full
    if can_shoot && (mouse_button.just_pressed(MouseButton::Left) || mouse_button.just_pressed(MouseButton::Right)) {
        if let Ok(ghost_transform) = ghost_query.get_single() {
            let direction = (cursor_pos.position - ghost_transform.translation.truncate()).normalize();
            
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: if mouse_button.just_pressed(MouseButton::Left) {
                            Color::srgb(1.0, 0.5, 0.5) // Red bullet
                        } else {
                            Color::srgb(0.5, 0.5, 1.0) // Blue bullet
                        },
                        custom_size: Some(Vec2::new(10.0, 10.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        ghost_transform.translation.x,
                        ghost_transform.translation.y,
                        1.0
                    ),
                    ..default()
                },
                Bullet {
                    speed: 500.0,
                    direction,
                },
            ));
        }
    }
}
