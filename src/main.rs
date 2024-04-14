use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::f32::consts::PI;
use rand::Rng;
use std::time::Duration;

const MAX_MANA:f32 = 10.0;
const ATTACK_MANA:f32 = 1.0;
const ATTACK_STRENGTH:f32 = 5.0;
const ATTACK_RANGE:f32 = 3.0;
const GOBLIN_ATTACK_TIME:f32 = 3.0;
const MANA_REPLENISH_RATE:f32 = 1.0;
const GOBLIN_ATTACK_DISTANCE:f32 = 2.0;
const MAX_DEMON_SCARE:f32 = 5.0;
const MAX_HEALTH:f32 = 30.0;
const WIZARD_ATTACK_TIMER:f32 = 1.0;
const DEMON_ATTACK_STRENGTH:f32 = 10.0;
const DEMON_ATTACK_RANGE:f32 = 5.0;

#[derive(Component)]
struct Wizard {
    speed: f32,
    mana: f32,
    health: f32,
    gameover: bool,
    attack_timer: f32,
}

#[derive(Component)]
struct Goblin {
    health: f32,
    attack_timer: f32,
}

#[derive(Resource)]
struct SpawnGoblinsConfig {
    timer: Timer,
}

#[derive(Component)]
struct ThemeMusic;

#[derive(Component)]
struct Demon {
    life: f32,
}

#[derive(Component)]
struct ManaText;

#[derive(Component)]
struct HealthText;

#[derive(Resource)]
struct WizardAnimations(Vec<Handle<AnimationClip>>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, (setup,setup_hud))
        .add_systems(Update, (
            wizard_move,
            wizard_attack,
            mouse_look,
            goblin_move,
            goblin_spawner,
            goblin_life,
            demon_summoner,
            demon_lifespan,
            wizard_vitals,
            display_events,
        ))
        .run();
}

fn setup_hud(
    mut commands: Commands,
) {
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Mana: ",
                TextStyle {
                    font_size: 32.0,
                    ..default()
                },
            ),
            TextSection::new(
                "",
                TextStyle {
                    font_size: 32.0,
                    ..default()   
                },
            )
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        }),
        ManaText,
    ));
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Health: ",
                TextStyle {
                    font_size: 32.0,
                    ..default()
                },
            ),
            TextSection::new(
                "",
                TextStyle {
                    font_size: 32.0,
                    ..default()   
                },
            )
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(5.0),
            ..default()
        }),
        HealthText,
    ));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(30.0)),
        material: materials.add(Color::WHITE),
        ..default()
    });

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 4.0, 8.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 20.0, 20.0).looking_at(Vec3::ZERO, Vec3::Z),
        ..default()
    });

    // Wizard
    commands.spawn(SceneBundle {
        scene: asset_server.load("wizard.glb#Scene0"),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..default()
    })
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Capsule3d::new(0.5,1.0)),
    //     material: materials.add(Color::rgb_u8(200,0,0)),
    //     transform: Transform::from_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)).with_translation(Vec3::new(0.0,0.0,0.5)),
    //     ..default()
    // })
        .insert(Wizard { speed: 6.0, mana: 10.0, health: MAX_HEALTH, gameover: false, attack_timer: WIZARD_ATTACK_TIMER})
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::ball(0.5))
        // .insert(ActiveEvents::COLLISION_EVENTS)
        // .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC)
        .insert(KinematicCharacterController::default());

    commands.insert_resource(SpawnGoblinsConfig {
        timer: Timer::new(Duration::from_secs(5), TimerMode::Repeating),
    });

    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/LD55-theme.ogg"),
            ..default()
        },
        ThemeMusic,
    ));
   
    commands.insert_resource(WizardAnimations(vec![
        asset_server.load("wizard.glb#Animation1"),
        asset_server.load("wizard.glb#Animation0"),
    ]));
}

fn goblin_spawner(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut config: ResMut<SpawnGoblinsConfig>,
){
    config.timer.tick(time.delta());
    let level = (time.elapsed().as_secs() / 30) + 2;
    if config.timer.just_finished() {
        println!("Spawning level {}",level);
        // Spawn some goblins
        let radius = 15.0;
        let goblin_count = rand::thread_rng().gen_range(1..=level);
        let phi_offset: f32 = PI * rand::random::<f32>();
        for n in 0..goblin_count {
            let phi = 2.0 * PI * (n as f32/goblin_count as f32) + phi_offset;
            let x = radius * phi.sin();
            let y = radius * phi.cos();
            // println!("Goblin {} {},{}",n,x,y);
            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(1.0,1.0,1.0)),
                material: materials.add(Color::rgb_u8(0, 200, 0)),
                transform: Transform::from_xyz(x, y, 0.5),
                ..default()
            })
                .insert(Goblin { health: 2.0, attack_timer: GOBLIN_ATTACK_TIME })
                .insert(RigidBody::KinematicPositionBased)
                .insert(GravityScale(0.0))
                .insert(Collider::ball(0.5))
                .insert(KinematicCharacterController::default());
        }
    }
}

fn demon_summoner(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform,&mut Wizard)>,
    mut goblin_query: Query<&mut Goblin, With<Goblin>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    rapier_context: Res<RapierContext>,
) {
    let (transform,mut wizard) = query.single_mut();
    if wizard.mana < MAX_MANA { 
        return; 
    }
    if keyboard_input.just_pressed(KeyCode::Space) {
        let summon_location = transform.translation + (transform.rotation * Vec3::Y).normalize() * 3.0;
        commands.spawn(PbrBundle {
            mesh: meshes.add(Sphere::new(1.0)),
            material: materials.add(Color::rgb_u8(200, 0, 0)),
            transform: Transform::from_xyz(summon_location.x, summon_location.y, 0.5),
            ..default()
        })
            .insert(Demon { life: 5.0 })
            .insert(RigidBody::KinematicPositionBased)
            .insert(Collider::ball(1.0));
        wizard.mana = 0.0; 

        // And let the demon smite the nearby goblins
        let shape = Collider::ball(DEMON_ATTACK_RANGE);
        let shape_pos = Vec2::new(summon_location.x,summon_location.y);
        let shape_rot = 0.0;
        let filter = QueryFilter::default();
        rapier_context.intersections_with_shape(
            shape_pos, shape_rot, &shape, filter, |entity| {
            if let Ok(mut goblin) = goblin_query.get_mut(entity) {
                goblin.health -= DEMON_ATTACK_STRENGTH;
            }
            return true;
        });


    }
}

fn demon_lifespan(
    time: Res<Time>,
    mut query: Query<(&mut Demon,Entity), With<Demon>>,
    mut commands: Commands,
) {
    for (mut demon, entity) in query.iter_mut() {
        demon.life -= time.delta_seconds();
        if demon.life <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn wizard_vitals(
    time: Res<Time>,
    mut wizard_query: Query<&mut Wizard, With<Wizard>>,
    mut mana_text_query: Query<&mut Text, (With<ManaText>,Without<HealthText>)>,
    mut health_text_query: Query<&mut Text, (With<HealthText>, Without<ManaText>)>,
) {
    let mut wizard = wizard_query.single_mut();
    wizard.mana += MANA_REPLENISH_RATE * time.delta_seconds();
    if wizard.mana > MAX_MANA {
        wizard.mana = MAX_MANA;
    }
    if wizard.health <= 0.0 && !wizard.gameover {
        println!("You died!");
        wizard.gameover = true;
    }
    let mana = wizard.mana;
    let health = wizard.health;
    let mut text = mana_text_query.single_mut();
    text.sections[1].value = format!("{mana:.0}");
    let mut text = health_text_query.single_mut();
    text.sections[1].value = format!("{health:.0}");
}

fn wizard_move(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut KinematicCharacterController,&Transform, &Wizard), With<Wizard>>,
) {
    let mut direction = Vec2::new(0.0,0.0);
    let (mut controller,transform,wizard) = query.single_mut();
    if wizard.gameover { return; }
    if keyboard_input.pressed(KeyCode::KeyW) {
        direction.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.x -= 1.0;
    }
    controller.translation = Some(direction.normalize() * wizard.speed * time.delta_seconds());
}

fn wizard_attack(
    time: Res<Time>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&Transform, &mut Wizard), With<Wizard>>,
    mut goblin_query: Query<&mut Goblin, With<Goblin>>,
    rapier_context: Res<RapierContext>,
    mut animation_players: Query<&mut AnimationPlayer>,
    animations: Res<WizardAnimations>,
) {
    let (transform, mut wizard) = query.single_mut();
    if wizard.attack_timer > 0.0 {
        wizard.attack_timer -= time.delta_seconds();
        return;
    }
    if wizard.mana < ATTACK_MANA {
        return;
    }
    // Get in-range goblins to hit? 
    if mouse_button_input.pressed(MouseButton::Left) {
        println!("Wizard Attack!");
        
        // Select goblins to strike
        let shape = Collider::ball(ATTACK_RANGE);
        let shape_pos = Vec2::new(transform.translation.x,transform.translation.y);
        let shape_rot = 0.0;

        // TODO move shape to front of wizard
        let filter = QueryFilter::default();
        rapier_context.intersections_with_shape(
            shape_pos, shape_rot, &shape, filter, |entity| {
            if let Ok(mut goblin) = goblin_query.get_mut(entity) {
                goblin.health -= ATTACK_STRENGTH;
            }
            return true;
        });

        wizard.mana -= ATTACK_MANA;
        wizard.attack_timer = WIZARD_ATTACK_TIMER;

        if !animation_players.is_empty() {
            println!("Playing wizard attack animation");
            let mut player = animation_players.single_mut();
            let clip = animations.0[1].clone_weak();
            player.replay();
            player.play(clip);
        } else {
            println!("No animation players!");
        }

    } else if mouse_button_input.pressed(MouseButton::Right) {
        // TODO Attack 2 (flame dartish things)
    }
}

fn goblin_life (
    time: Res<Time>,
    mut query: Query<(&mut Goblin,Entity), With<Goblin>>,
    mut commands: Commands,
) {
    for (mut goblin, entity) in query.iter_mut() {
        if goblin.health <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn goblin_move (
   time: Res<Time>,
   mut wizard_query: Query<(&Transform, &mut Wizard), (With<Wizard>, Without<Goblin>, Without<Demon>)>,
   mut goblin_query: Query<(&mut KinematicCharacterController, &mut Transform, &mut Goblin), (With<Goblin>, Without<Wizard>, Without<Demon>)>,
   demon_query: Query<&Transform, (With<Demon>, Without<Wizard>, Without<Goblin>)>,
) {
    let (wizard_transform, mut wizard) = wizard_query.single_mut();
    let wizard_position = wizard_transform.translation;
    if wizard.gameover { return; }

    let demon_position = match demon_query.is_empty() {
        true => None,
        false => Some(demon_query.single().translation),
    };

    for (mut controller, mut transform, mut goblin) in goblin_query.iter_mut() {
        // creepy goblins, always looking at the wizard
        let to_wizard = wizard_position - transform.translation; 
       
        // unless a demon is nearby! 
        let direction = match (to_wizard,demon_position) {
            (_,None) => Vec2::new(to_wizard.x,to_wizard.y),
            (to_wizard,Some(demon_position)) => {
                let to_demon = demon_position - transform.translation;
                let demon_distance = to_demon.length();
                if to_wizard.length() < demon_distance || demon_distance > MAX_DEMON_SCARE {
                    Vec2::new(to_wizard.x,to_wizard.y)
                } else {
                    // run away!
                    Vec2::new(-to_demon.x,-to_demon.y)
                }
            },
        };

        let rotation = Quat::from_rotation_z(-direction.x.atan2(direction.y));
        transform.rotation = rotation;
        // // and maybe moving towards the wizard
        controller.translation = Some(direction.normalize() * 1.5 * time.delta_seconds());

        if to_wizard.length() <= GOBLIN_ATTACK_DISTANCE && goblin.attack_timer <= 0.0 {
            println!("Attacking!");
            wizard.health -= 1.0;
            goblin.attack_timer = GOBLIN_ATTACK_TIME;
        } else if goblin.attack_timer > 0.0 {
            goblin.attack_timer -= time.delta_seconds();
        }
    }
}

fn mouse_look(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut wizard_query: Query<&mut Transform, With<Wizard>>,
) {
    let (camera, camera_transform) = camera_query.single();
    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    }; 
    let Some(distance) = ray.intersect_plane(Vec3::ZERO, Plane3d::new(Vec3::Z))
    else {
        return;
    };
    let point = ray.get_point(distance); 

    for mut wizard in wizard_query.iter_mut() {
        let direction = (point - wizard.translation).normalize();
        let rotation = Quat::from_rotation_z(-direction.x.atan2(direction.y));
        wizard.rotation = rotation;
    }
}

fn display_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
) {
    for collision_event in collision_events.read() {
        println!("Received collision event: {:?}", collision_event);
    }

    for contact_force_event in contact_force_events.read() {
        println!("Received contact force event: {:?}", contact_force_event);
    }
}
