use avian3d::prelude::*;
use bevy::prelude::*;

use rand::prelude::*;

use crate::constants::{
    STONE_RADIUS,
    SHEET_LENGTH
};

use crate::game::{GameState, OnGameScreen};

pub struct TownsfolkPlugin;

impl Plugin for TownsfolkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup);
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
            20.0,
            -rng.gen_range(0.0..SHEET_LENGTH));
        commands
            .spawn((
                Name::new("Person1"),
                OnGameScreen,
                SceneRoot(
                    asset_server
                        .load(GltfAssetLabel::Scene(0).from_asset("models/person.glb"))),
                //ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
                RigidBody::Dynamic,
                MaxLinearSpeed(20.0),

                Collider::cuboid(1.0, 1.0, 1.7),
                Transform::from_xyz(pos.x, pos.y, pos.z)));
    }
}
