use bevy::prelude::*;
use avian3d::prelude::*;

use crate::game::{GameState, OnGameScreen};

use crate::constants::{
    STONE_DAMPENING,
    STONE_MAX_VEL,
    STONE_RADIUS,
    STONE_X,
    STONE_Y,
    STONE_Z,
};

#[derive(Component)]
pub struct Stone;

pub struct StonePlugin;
impl Plugin for StonePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let texture_handle = asset_server.load("textures/stone076.jpg");
    // this material renders the texture normally
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        perceptual_roughness: 0.8,
        ..default()
    });

    // stone
    commands.spawn((
        Stone,
        OnGameScreen,
        //RigidBody::Dynamic,
        Collider::sphere(STONE_RADIUS),
        LinearDamping(STONE_DAMPENING),
        MaxLinearSpeed(STONE_MAX_VEL),
        //Friction::new(10.0),
        //CollisionMargin(0.1),
        //Mass(weight),
        LinearVelocity(Vec3::new(0.0, 0.0, 160.0)),//160.0)),
        AngularVelocity(Vec3::new( 10.0, 0.0, 0.0)),
        Mesh3d(meshes.add(Sphere::new(STONE_RADIUS))),
        MeshMaterial3d(material_handle),//materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(STONE_X, STONE_Y, STONE_Z),
        TransformInterpolation
    ));
}
