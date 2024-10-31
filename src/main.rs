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
struct Ghost;

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
