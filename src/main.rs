use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;

#[derive(Component)]
struct Wizard {
    speed: f32,
}

#[derive(Component)]
struct Goblin {
    speed: f32,
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, keyboard_input_system)
        .add_systems(Update, mouse_look)
        .add_systems(Update, goblin_move)
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
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // and ground collider
    commands.spawn(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)))
      .insert(Collider::cuboid(100.0, 0.1, 100.0));

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 20.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Wizard
    commands.spawn(SceneBundle {
        scene: asset_server.load("wizard.glb#Scene0"),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..default()
    })
        .insert(Wizard { speed: 6.0 })
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::capsule(Vec3::new(0.0,0.0,0.0),Vec3::new(0.0,2.0,0.0),0.5))
        .insert(KinematicCharacterController::default());
    
    // Goblins
    let radius = 15.0;
    let goblin_count = 10;
    for n in 0..goblin_count {
        let phi = 2.0 * PI * (n as f32/goblin_count as f32);
        let x = radius * phi.sin();
        let z = radius * phi.cos();
        println!("Goblin {} {},{}",n,x,z);
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::rgb_u8(0, 200, 0)),
            transform: Transform::from_xyz(x, 0.5, z),
            ..default()
        })
            .insert(Goblin { speed: 3.0 })
            .insert(RigidBody::KinematicPositionBased)
            .insert(Collider::cuboid(0.5,0.5,0.5))
            .insert(KinematicCharacterController::default());

    }

}

fn keyboard_input_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut KinematicCharacterController,&Transform, &Wizard)>,
) {
    let mut direction = Vec3::new(0.0,0.0,0.0);
    let (mut controller,transform,wizard) = query.single_mut();
    if keyboard_input.pressed(KeyCode::KeyW) {
        direction += transform.rotation * Vec3::X;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction -= transform.rotation * Vec3::X;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction -= transform.rotation * Vec3::Z;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction += transform.rotation * Vec3::Z;
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
        let direction = wizard_position - transform.translation; 
        let rotation = Quat::from_rotation_y(-direction.z.atan2(direction.x));
        transform.rotation = rotation;
        // and maybe moving towards the wizard
        goblin_controller.translation  = Some(direction.normalize() * 2.5 * time.delta_seconds());
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
    let Some(distance) = ray.intersect_plane(Vec3::ZERO, Plane3d::new(Vec3::Y))
    else {
        return;
    };
    let point = ray.get_point(distance); 

    for mut wizard in wizard_query.iter_mut() {
        let direction = (point - wizard.translation).normalize();
        let rotation = Quat::from_rotation_y(-direction.z.atan2(direction.x));
        wizard.rotation = rotation;
    }
}
