use bevy::prelude::*;
use avian3d::prelude::*;

use crate::game::{GameState, OnGameScreen, Spotty, CollisionLayer};

use crate::constants::{
    CHUNK_SIZE,
    STONE_ANGULAR_DAMPENING,
    STONE_DAMPENING,
    STONE_MAX_VEL,
    STONE_RADIUS,
    STONE_X,
    STONE_Y,
    STONE_Z,
};

#[derive(Component)]
pub struct Stone;

pub fn stone_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::InGame), setup);
    app.add_systems(Update, stone_update);
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
        //RigidBody::Dynamic, // Gets added when you fire
        Collider::sphere(STONE_RADIUS),
        ColliderDensity(10.0),
        CollisionLayers::new(
            [CollisionLayer::Stone],
            [CollisionLayer::Terrain, CollisionLayer::Sensors, CollisionLayer::Townsfolk]
        ),
        LinearDamping(STONE_DAMPENING),
        AngularDamping(STONE_ANGULAR_DAMPENING),
        MaxLinearSpeed(STONE_MAX_VEL),
        Friction::new(1.0),
        //CollisionMargin(0.1),
        //Mass(weight),
        LinearVelocity(Vec3::new(0.0, 0.0, 160.0)),
        AngularVelocity(Vec3::new( 10.0, 0.0, 0.0)),
        Mesh3d(meshes.add(Sphere::new(STONE_RADIUS))),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(STONE_X, STONE_Y, STONE_Z),
        TransformInterpolation, // smooths the movement
    ));
}

fn stone_update (
    mut stone: Query<(&mut Transform, &mut LinearVelocity), With<Stone>>,
    mut spotty: Query<&mut Transform, (With<Spotty>, Without<Stone>)>,
){
    let Ok((mut stone_pos, mut vel_vec)) = stone.get_single_mut() else { return; };
    let Ok(mut spot_pos) = spotty.get_single_mut() else { return; };

    spot_pos.translation = stone_pos.translation + Vec3::new(1.0, STONE_RADIUS * 2.0, 1.0);

    let x_dist = stone_pos.translation.x.abs();
    let y_dist = stone_pos.translation.y;
    if x_dist > CHUNK_SIZE || y_dist < -STONE_RADIUS * 12.0 {
        // TODO: this should transition to phase?
        // don't think this is needed anymore?
        // is it reset after fall off edge?
        stone_pos.translation = Vec3::new(STONE_X, STONE_Y, STONE_Z);
        vel_vec.x = 0.0;
        vel_vec.y = 0.0;
        vel_vec.z = 0.0;
    }
}
