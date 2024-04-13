use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::f32::consts::PI;
use rand::Rng;
use std::time::Duration;

const MAX_MANA:f32 = 10.0;
const GOBLIN_ATTACK_TIME:f32 = 3.0;
const MANA_REPLENISH_RATE:f32 = 1.0;

#[derive(Component)]
struct Wizard {
    speed: f32,
    mana: f32,
    health: f32,
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, (setup,setup_hud))
        .add_systems(Update, (
            keyboard_input_system,
            mouse_look,
            goblin_move,
            goblin_spawner,
            demon_summoner,
            demon_lifespan,
            wizard_mana_replenish,
        ))
        .run();
}

fn setup_hud(
    mut commands: Commands,
) {
    commands.spawn((
        TextBundle::from_section(
            "Mana: X",
            TextStyle {
                font_size: 32.0,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        }),
        ManaText,
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
        .insert(Wizard { speed: 6.0, mana: 10.0, health: 100.0 })
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::ball(0.5))
        .insert(KinematicCharacterController::default());

    commands.insert_resource(SpawnGoblinsConfig {
        timer: Timer::new(Duration::from_secs(8), TimerMode::Repeating),
    });

    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/LD55-theme.ogg"),
            ..default()
        },
        ThemeMusic,
    ));
}

fn goblin_spawner(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut config: ResMut<SpawnGoblinsConfig>,
){
    config.timer.tick(time.delta());
    let level = (time.elapsed().as_secs() / 30) + 1;
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
    time: Res<Time>,
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform,&mut Wizard)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

fn wizard_mana_replenish(
    time: Res<Time>,
    mut query: Query<(&mut Wizard), With<Wizard>>,
    mut text_query: Query<&mut Text, With<ManaText>>,
) {
    let mut text = text_query.single_mut();
    for mut wizard in query.iter_mut() {
        wizard.mana += MANA_REPLENISH_RATE * time.delta_seconds();
        if wizard.mana > MAX_MANA {
            wizard.mana = MAX_MANA;
        }

    }
}

fn keyboard_input_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut KinematicCharacterController,&Transform, &Wizard)>,
) {
    let mut direction = Vec2::new(0.0,0.0);
    let (mut controller,transform,wizard) = query.single_mut();
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

fn goblin_move (
   time: Res<Time>,
   mut set: ParamSet<(
    Query<&Transform, With<Wizard>>,
    Query<(&mut KinematicCharacterController, &mut Transform), With<Goblin>>,
   )>
) {
    let wizard_position = set.p0().single().translation.clone();
    for (mut controller, mut transform) in set.p1().iter_mut() {
        // creepy goblins, always looking at the wizard
        let direction3d = wizard_position - transform.translation; 
        let direction = Vec2::new(direction3d.x,direction3d.y);
        let rotation = Quat::from_rotation_z(-direction.x.atan2(direction.y));
        transform.rotation = rotation;
        // // and maybe moving towards the wizard
        controller.translation = Some(direction.normalize() * 1.5 * time.delta_seconds());
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
