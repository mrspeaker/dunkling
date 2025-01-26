use avian3d::prelude::*;
use bevy::prelude::*;

use std::f32::consts::*;

#[derive(Component)]
struct TrackingCamera;

#[derive(Component)]
struct Stone;

#[derive(Component)]
struct BobX {
    dt: f32
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, (cam_track, stone_shoot, bob))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    // sheet
    let width = 5.0;
    let length = 50.0;
    let pre_area = 5.0;
    let post_area = 10.0;

    // start line
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(width-0.01, 0.45, 0.2))),
        MeshMaterial3d(materials.add(Color::BLACK)),
    ));

    // sheet
    commands.spawn((
        RigidBody::Static,
        Friction::new(0.05),
        Collider::cuboid(width, 0.3, length),
        Mesh3d(meshes.add(Cuboid::new(width, 0.3, length))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 0.0, -length / 2.0 + (pre_area / 2.0) ),
    ));

    // jump
    commands.spawn((
        RigidBody::Static,
        Friction::new(0.05),
        Collider::cuboid(width * 0.6, 0.3, 2.0),
        Mesh3d(meshes.add(Cuboid::new(width * 0.6, 0.3, 2.0))),
        MeshMaterial3d(materials.add(Color::BLACK)),
        Transform::from_xyz(-width/2.0, 0.0, -10.0)
            .with_rotation(Quat::from_rotation_x(PI / 8.)),
        BobX{ dt: 0.0 }
    ));


    // stone
    let radius = 0.25;
    let height = 0.2;
    let weight = 20.0;

    commands.spawn((
        Stone,
        RigidBody::Dynamic,
        Collider::cylinder(radius, height),
        Friction::new(0.05),
        Mass(weight),

        //AngularVelocity(Vec3::new(2.5, 3.5, 1.5)),
        LinearVelocity(Vec3::new(0.0, 0.0, 0.0)),
        Mesh3d(meshes.add(Cylinder::new(0.5, 0.3))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(20.0, 6.0, 10.0).looking_at(Vec3::new(0.0, 3.0, 0.0), Dir3::Y),
        TrackingCamera
    ));
}


fn cam_track(
    time: Res<Time>,
    stone: Query<&Transform, (With<Stone>, Without<TrackingCamera>)>,
    mut camera: Query<&mut Transform, With<TrackingCamera>>,
){
    let dt = time.delta_secs();
    let stone_pos = stone.single();

    let mut camera = camera.single_mut();

    let dist = stone_pos.translation.distance(camera.translation);
    if dist > 6.0 {
        let move_amount = camera.forward();
        camera.translation += move_amount * 10.0 * dt;
        camera.look_at(stone_pos.translation + Vec3::new(0.0, 1.0, 1.0), Dir3::Y);
    }

}

fn stone_shoot(
    input: Res<ButtonInput<KeyCode>>,
    mut vel: Query<&mut LinearVelocity, With<Stone>>,
){
    let mut vel_vec = vel.single_mut();
    if input.pressed(KeyCode::KeyW) {
        vel_vec.z = -8.0;
    }
}

fn bob(
    time: Res<Time>,
    mut t: Query<(&mut Transform, &mut BobX)>,
){
    let dt = time.delta_secs();

    for (mut t, mut b) in t.iter_mut() {
    let move_amount = t.right() * b.dt.sin() * 10.0;
        t.translation += move_amount * dt;
        b.dt += dt * 4.0;
    }
}
