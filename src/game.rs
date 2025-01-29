use avian3d::prelude::*;

use bevy::{
    prelude::*,
    color::palettes::css::*,
};

use std::f32::consts::*;

use crate::constants::{
    STONE_RADIUS,
    STONE_DAMPENING,
};
use crate::camera::CameraPlugin;
use crate::player::PlayerPlugin;
use crate::sheet::SheetPlugin;
use crate::townsfolk::TownsfolkPlugin;

pub struct GamePlugin;

#[derive(Component)]
pub struct Stone;

#[derive(Component)]
pub struct Spotty;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MeshPickingPlugin,
//            PhysicsDebugPlugin::default(),
            PhysicsPlugins::default()));
        app.add_plugins(CameraPlugin);
        app.add_plugins(PlayerPlugin);
        app.add_plugins(SheetPlugin);
        app.add_plugins(TownsfolkPlugin);

        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // stone
    commands.spawn((
        Stone,
        RigidBody::Dynamic,
        Collider::sphere(STONE_RADIUS),
        LinearDamping(STONE_DAMPENING),
        //Friction::new(10.0),
        //CollisionMargin(0.1),
        //Mass(weight),
        LinearVelocity(Vec3::new(0.0, 0.0, 0.0)),
        Mesh3d(meshes.add(Sphere::new(STONE_RADIUS))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, STONE_RADIUS * 2.0, 0.0),
    ));

    // Light
    commands.spawn((
        PointLight {
            intensity: 10_000_000.0,
            range: 100.0,
            radius: 100.0,
            color: BLUE.into(),
            shadows_enabled: true,
            ..default()
        },
        Spotty
    ));

    // Thor
    let texture_handle = asset_server.load("thor.png");
    let aspect = 1.0;//0.25;
    let quad_width = STONE_RADIUS * 10.0;
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        double_sided: true,
        cull_mode: None, //Some(Face::Back)
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(quad_width, quad_width * aspect))),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(STONE_RADIUS * 2.0, 1.0, STONE_RADIUS * 3.0)
            .with_rotation(Quat::from_euler(
                // YXZ = "yaw"/"pitch"/"roll"
                EulerRot::YXZ,
                (180.0_f32).to_radians(),
                (0.0_f32).to_radians(),
                (0.0_f32).to_radians(),
            ))
    ));

    // Lights
    commands.insert_resource(AmbientLight {
        color: Color::linear_rgb(1.0,1.0, 0.8),
        brightness: 100.0,
    });

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 100.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 2.0),
            ..default()
        },
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(100.0, 100.0, 50.0),
            rotation: Quat::from_rotation_x(-PI * 0.9),
            ..default()
        },
    ));

}
