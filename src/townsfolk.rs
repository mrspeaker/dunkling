use std::f32::consts::PI;

use avian3d::prelude::{RigidBody, Collider};
use bevy::prelude::*;

use rand::prelude::*;

use crate::constants::{
    SHEET_TOTAL,
    CHUNK_SIZE,
};

use crate::game::{GameState, OnGameScreen};
use crate::height_map::HeightMap;
use crate::sheet::TerrainCreated;

#[derive(Component)]
struct Peep;

#[derive(Component)]
struct Target(Option<Vec2>);

#[derive(Component)]
struct Speed(f32);


pub struct TownsfolkPlugin;

impl Plugin for TownsfolkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_peeps.run_if(in_state(GameState::InGame)));
        app.add_observer(spawn_townsfolk);
    }
}

pub fn spawn_townsfolk(
    _trigger: Trigger<TerrainCreated>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    height_map: Res<HeightMap>
) {
    // get height_map
    let mut rng = rand::thread_rng();
    let w = CHUNK_SIZE;
    for i in 0..200 {
        let x = rng.gen_range(0.0..w); // right(0) to left (w)
        let z = rng.gen_range(0.0..SHEET_TOTAL - CHUNK_SIZE * 2.0);
        let y = height_map.pos_to_height(x, z).unwrap_or(0.0);
        let pos = Vec3::new(x - w / 2.0, y, z - CHUNK_SIZE / 2.0);
        let rot = rng.gen_range(0.0..PI*2.0);

        commands
            .spawn((
                Name::new("Person1"),
                Peep,
                OnGameScreen,
                SceneRoot(
                    asset_server
                        .load(GltfAssetLabel::Scene(0).from_asset("models/person.glb"))),
                Target(None),
                Speed(0.0),
                Transform::from_xyz(pos.x, pos.y, pos.z)));

        // Some buildings. TODO: put them somehwere else
        commands
            .spawn((
                Name::new("House"),
                OnGameScreen,
                SceneRoot(
                    asset_server
                        .load(GltfAssetLabel::Scene(0).from_asset(
                            if i % 3 == 0 {
                                "models/cab.glb"
                            } else if i % 3 == 1 {
                                "models/shop.glb"
                            } else {"models/house.glb"}
                        ))),
                Transform::from_xyz(pos.x, pos.y, pos.z).with_rotation(Quat::from_rotation_y(rot))
            ));
    }

    // Add some trees
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let x = rng.gen_range(0.0..w); // right(0) to left (w)
        let z = rng.gen_range(0.0..SHEET_TOTAL - CHUNK_SIZE * 2.0);
        let y = height_map.pos_to_height(x, z).unwrap_or(0.0);
        let pos = Vec3::new(x - w / 2.0, y, z - w / 2.0);

        commands
            .spawn((
                Name::new("Tree"),
                OnGameScreen,
                SceneRoot(
                    asset_server
                        .load(GltfAssetLabel::Scene(0).from_asset("models/tree.glb"))),
                RigidBody::Static,
                Collider::cuboid(1.0, 1.0, 1.7),
                Transform::from_xyz(pos.x, pos.y, pos.z)));
    }
}

fn move_peeps(
    mut peeps: Query<(&mut Transform, &mut Target, &mut Speed), With<Peep>>,
    height_map: Res<HeightMap>,
    time: Res<Time>
) {
    let dt = time.delta_secs();

    for (mut t, mut target, mut speed) in peeps.iter_mut() {
        let pos = t.translation;
        if target.0.is_none() {
            let mut rng = rand::thread_rng();
            let x = rng.gen_range(-10.0..10.0);
            let z = rng.gen_range(-10.0..10.0);
            target.0 = Some(Vec2::new(pos.x + x, pos.z + z));
            speed.0 = rng.gen_range(1.0..5.0);
        }

        // Move towards target
        if let Some(targ) = target.0 {
            let dir = (targ - pos.xz()).normalize();
            t.translation += Vec3::new(dir.x, 0.0, dir.y) * speed.0 * dt;
            let dist = targ.distance(t.translation.xz());
            if dist <= 0.2 {
                target.0 = None;
            }
        }

        if let Some(h) = height_map.pos_to_height(pos.x + CHUNK_SIZE / 2.0, pos.z + CHUNK_SIZE / 2.0) {
            t.translation.y = h;
        } else {
            // out of bounds
            target.0 = None;
        }

    }
}
