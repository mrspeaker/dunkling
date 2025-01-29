use avian3d::prelude::*;
use bevy::prelude::*;

use rand::prelude::*;

use crate::constants::{
    STONE_RADIUS,
    SHEET_LENGTH
};

pub struct TownsfolkPlugin;

impl Plugin for TownsfolkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, update);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Lil people
    let mut rng = rand::thread_rng();
    let w = STONE_RADIUS * 10.0;
    for _ in 0..200 {
        let pos = Vec3::new(
            rng.gen_range(-w..w),
            0.1,
            -rng.gen_range(0.0..SHEET_LENGTH));
        commands
            .spawn((
                Name::new("Person1"),
                SceneRoot(
                    asset_server
                        .load(GltfAssetLabel::Scene(0).from_asset("person.glb"))),
                ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
                RigidBody::Dynamic,
//                Collider::cuboid(1.0, 1.0, 1.0),
                Transform::from_xyz(pos.x, pos.y, pos.z)));
                //.with_rotation(Quat::from_rotation_z(PI / 2.))
                 //   .with_scale(Vec3::splat(0.2))));

    }
}

fn update() {

}
