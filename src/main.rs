use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::f32::consts::PI;
use rand::Rng;
use std::time::Duration;

#[derive(Component)]
struct Wizard {
    speed: f32,
}

#[derive(Component)]
struct Goblin;

#[derive(Resource)]
struct SpawnGoblinsConfig {
    timer: Timer,
}

#[derive(Component)]
struct ThemeMusic;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, keyboard_input_system)
        .add_systems(Update, mouse_look)
        .add_systems(Update, goblin_move)
        .add_systems(Update, goblin_spawner)
        .run();
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
        .insert(Wizard { speed: 6.0 })
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
                .insert(Goblin)
                .insert(RigidBody::KinematicPositionBased)
                .insert(Collider::ball(0.5))
                .insert(KinematicCharacterController::default());
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
    // move in the direction the wizard is facing
    // if keyboard_input.pressed(KeyCode::KeyW) {
    //     direction += transform.rotation * Vec3::X;
    // }
    // if keyboard_input.pressed(KeyCode::KeyS) {
    //     direction -= transform.rotation * Vec3::X;
    // }
    // if keyboard_input.pressed(KeyCode::KeyA) {
    //     direction -= transform.rotation * Vec3::Z;
    // }
    // if keyboard_input.pressed(KeyCode::KeyD) {
    //     direction += transform.rotation * Vec3::Z;
    // }
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
    for (mut goblin_controller, mut transform) in set.p1().iter_mut() {
        // creepy goblins, always looking at the wizard
        let direction3d = wizard_position - transform.translation; 
        let direction = Vec2::new(direction3d.x,direction3d.y);
        let rotation = Quat::from_rotation_z(-direction.x.atan2(direction.y));
        transform.rotation = rotation;
        // // and maybe moving towards the wizard
        goblin_controller.translation  = Some(direction.normalize() * 1.5 * time.delta_seconds());
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
