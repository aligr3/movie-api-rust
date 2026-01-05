use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
struct Snowflake {
    speed: f32,
    drift: f32,
    drift_speed: f32,
}

#[derive(Component)]
struct VirusChar {
    velocity: Vec2,
    rotation_speed: f32,
}

#[derive(Component)]
struct StartText;

#[derive(Component)]
struct CountdownText;

#[derive(Component)]
struct Background;

#[derive(Resource)]
struct GameState {
    started: bool,
    countdown: f32,
    virus_active: bool,
    virus_spawn_timer: f32,
    chaos_level: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "🎄 Das beste Spiel des Jahres 🎄".to_string(),
                resolution: (1200.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.15)))
        .insert_resource(GameState {
            started: false,
            countdown: 10.0,
            virus_active: false,
            virus_spawn_timer: 0.0,
            chaos_level: 1.0,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (
            check_start_system,
            countdown_system,
            snowflake_system,
            virus_system,
            virus_movement_system,
        ))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    // Hintergrund Gradient Effekt
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.3),
                custom_size: Some(Vec2::new(1200.0, 800.0)),
                ..default()
            },
            ..default()
        },
        Background,
    ));

    // Start Text mit Glow-Effekt
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "DRÜCKE LEERTASTE",
                TextStyle {
                    font_size: 60.0,
                    color: Color::srgb(0.0, 1.0, 0.5),
                    ..default()
                },
            )
            .with_justify(JustifyText::Center),
            transform: Transform::from_xyz(0.0, -50.0, 3.0),
            ..default()
        },
        StartText,
    ));

    // Untertitel
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "um zu starten...",
                TextStyle {
                    font_size: 30.0,
                    color: Color::srgba(0.7, 0.7, 0.7, 0.8),
                    ..default()
                },
            )
            .with_justify(JustifyText::Center),
            transform: Transform::from_xyz(0.0, -110.0, 3.0),
            ..default()
        },
        StartText,
    ));

    // Countdown / Titel Text
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "🎄 Das beste Spiel des Jahres 🎄",
                TextStyle {
                    font_size: 70.0,
                    color: Color::srgb(1.0, 0.843, 0.0),
                    ..default()
                },
            )
            .with_justify(JustifyText::Center),
            transform: Transform::from_xyz(0.0, 250.0, 3.0),
            ..default()
        },
        CountdownText,
    ));

    // Verbesserte Schneeflocken
    let mut rng = rand::thread_rng();
    for _ in 0..150 {
        let x = rng.gen_range(-600.0..600.0);
        let y = rng.gen_range(-400.0..400.0);
        let speed = rng.gen_range(30.0..80.0);
        let size = rng.gen_range(20.0..40.0);
        let drift_speed = rng.gen_range(0.5..2.0);
        
        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    "❄",
                    TextStyle {
                        font_size: size,
                        color: Color::srgba(1.0, 1.0, 1.0, rng.gen_range(0.6..1.0)),
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(x, y, 1.0),
                ..default()
            },
            Snowflake { 
                speed,
                drift: rng.gen_range(0.0..6.28),
                drift_speed,
            },
        ));
    }
}

fn check_start_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<GameState>,
    mut start_query: Query<&mut Text, With<StartText>>,
    mut countdown_query: Query<&mut Text, (With<CountdownText>, Without<StartText>)>,
    mut bg_query: Query<&mut Sprite, With<Background>>,
    virus_query: Query<Entity, With<VirusChar>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if !state.started {
            // Spiel starten
            state.started = true;
            for mut text in start_query.iter_mut() {
                text.sections[0].value = "".to_string();
            }
            // Hintergrund dunkler machen
            for mut sprite in bg_query.iter_mut() {
                sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.6);
            }
        } else if state.virus_active {
            // Spiel zurücksetzen
            state.started = false;
            state.countdown = 10.0;
            state.virus_active = false;
            state.virus_spawn_timer = 0.0;
            state.chaos_level = 1.0;
            
            // Start-Text wieder anzeigen
            for mut text in start_query.iter_mut() {
                if text.sections[0].value == "" {
                    text.sections[0].value = "DRÜCKE LEERTASTE".to_string();
                } else {
                    text.sections[0].value = "um zu starten...".to_string();
                }
            }
            
            // Titel zurücksetzen
            for mut text in countdown_query.iter_mut() {
                text.sections[0].value = "🎄 Das beste Spiel des Jahres 🎄".to_string();
                text.sections[0].style.color = Color::srgb(1.0, 0.843, 0.0);
                text.sections[0].style.font_size = 70.0;
            }
            
            // Hintergrund aufhellen
            for mut sprite in bg_query.iter_mut() {
                sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.3);
            }
            
            // Alle Virus-Zeichen entfernen
            for entity in virus_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn countdown_system(
    time: Res<Time>,
    mut state: ResMut<GameState>,
    mut query: Query<&mut Text, With<CountdownText>>,
) {
    if state.started && !state.virus_active {
        state.countdown -= time.delta_seconds();

        if state.countdown > 0.0 {
            for mut text in query.iter_mut() {
                let countdown_num = state.countdown.ceil() as u32;
                let color = if countdown_num <= 3 {
                    Color::srgb(1.0, 0.2, 0.2) // Rot für letzte 3 Sekunden
                } else if countdown_num <= 5 {
                    Color::srgb(1.0, 0.6, 0.0) // Orange
                } else {
                    Color::srgb(0.0, 1.0, 0.5) // Grün
                };
                text.sections[0].value = format!("{}", countdown_num);
                text.sections[0].style.color = color;
                text.sections[0].style.font_size = 120.0 + (10.0 - state.countdown) * 10.0;
            }
        } else {
            for mut text in query.iter_mut() {
                text.sections[0].value = "🎄 Frohe Weihnachten! 🎄".to_string();
                text.sections[0].style.color = Color::srgb(1.0, 0.0, 0.0);
                text.sections[0].style.font_size = 80.0;
            }
            state.virus_active = true;
        }
    }
}

fn snowflake_system(
    time: Res<Time>,
    mut query: Query<(&mut Snowflake, &mut Transform)>,
) {
    for (mut flake, mut transform) in query.iter_mut() {
        // Fallen
        transform.translation.y -= flake.speed * time.delta_seconds();
        
        // Seitliche Drift
        flake.drift += flake.drift_speed * time.delta_seconds();
        transform.translation.x += (flake.drift.sin() * 30.0) * time.delta_seconds();
        
        // Rotation
        transform.rotation = Quat::from_rotation_z(flake.drift * 0.5);
        
        // Respawn oben
        if transform.translation.y < -450.0 {
            transform.translation.y = 450.0;
            transform.translation.x = rand::thread_rng().gen_range(-600.0..600.0);
        }
    }
}

fn virus_system(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<GameState>,
    query: Query<&VirusChar>,
) {
    if state.virus_active {
        state.virus_spawn_timer += time.delta_seconds();
        state.chaos_level += time.delta_seconds() * 0.1;

        let spawn_rate = 0.02 / state.chaos_level.min(3.0);
        
        if state.virus_spawn_timer > spawn_rate && query.iter().count() < 1000 {
            state.virus_spawn_timer = 0.0;
            let mut rng = rand::thread_rng();
            
            // Spawn von den Rändern
            let (x, y) = if rng.gen_bool(0.5) {
                (rng.gen_range(-600.0..600.0), if rng.gen_bool(0.5) { 450.0 } else { -450.0 })
            } else {
                (if rng.gen_bool(0.5) { 650.0 } else { -650.0 }, rng.gen_range(-400.0..400.0))
            };

            let target_x = rng.gen_range(-300.0..300.0);
            let target_y = rng.gen_range(-200.0..200.0);
            
            let dir = Vec2::new(target_x - x, target_y - y).normalize();
            let speed = rng.gen_range(100.0..300.0) * state.chaos_level;

            // Zufällige ASCII-Zeichen mit besserer Auswahl
            let chars = ['#', '@', '$', '%', '&', '*', '+', '=', '?', '!', '~', '^'];
            let character = chars[rng.gen_range(0..chars.len())];
            
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section(
                        character.to_string(),
                        TextStyle {
                            font_size: rng.gen_range(20.0..60.0),
                            color: Color::srgb(
                                rng.gen_range(0.5..1.0),
                                rng.gen_range(0.0..0.5),
                                rng.gen_range(0.0..1.0),
                            ),
                            ..default()
                        },
                    ),
                    transform: Transform::from_xyz(x, y, 2.0),
                    ..default()
                },
                VirusChar {
                    velocity: dir * speed,
                    rotation_speed: rng.gen_range(-5.0..5.0),
                },
            ));
        }
    }
}

fn virus_movement_system(
    time: Res<Time>,
    state: Res<GameState>,
    mut query: Query<(&mut VirusChar, &mut Transform, &mut Text)>,
) {
    if state.virus_active {
        for (mut virus, mut transform, mut text) in query.iter_mut() {
            // Bewegung
            transform.translation.x += virus.velocity.x * time.delta_seconds();
            transform.translation.y += virus.velocity.y * time.delta_seconds();
            
            // Rotation
            transform.rotation *= Quat::from_rotation_z(virus.rotation_speed * time.delta_seconds());
            
            // Beschleunigung zum Chaos
            virus.velocity *= 1.0 + (time.delta_seconds() * 0.3);
            
            // Bildschirmrand bounce mit mehr Chaos
            if transform.translation.x.abs() > 650.0 {
                virus.velocity.x *= -1.1;
                virus.rotation_speed *= -1.0;
            }
            if transform.translation.y.abs() > 450.0 {
                virus.velocity.y *= -1.1;
                virus.rotation_speed *= -1.0;
            }
            
            // Farbe pulsiert
            let pulse = (time.elapsed_seconds() * 5.0).sin() * 0.5 + 0.5;
            text.sections[0].style.color = Color::srgb(
                0.5 + pulse * 0.5,
                pulse * 0.3,
                1.0 - pulse * 0.3,
            );
        }
    }
}