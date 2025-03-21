use bevy::prelude::*;
use avian3d::prelude::{RigidBody, Collider, CollisionLayers, MaxLinearSpeed, MaxAngularSpeed, CollisionStarted};
use crate::{sheet::TerrainCreated, constants::{CHUNK_SIZE, SHEET_TOTAL}, height_map::HeightMap, game::{OnGameScreen, CollisionLayer, GameState}, stone::Stone};
use rand::prelude::*;


#[derive(Component)]
struct PowerupSensor;

pub fn powerups_plugin(app: &mut App) {
    app.add_systems(Update, detect_collisions.run_if(in_state(GameState::InGame)));
    app.add_observer(spawn_powerups);
}

pub fn spawn_powerups(
    _trigger: Trigger<TerrainCreated>,
    mut commands: Commands,
    height_map: Res<HeightMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::thread_rng();
    let w = CHUNK_SIZE;

    for _ in 0..30 {

    let x = rng.gen_range(0.0..w); // right(0) to left (w)
    let z = rng.gen_range(0.0..SHEET_TOTAL - CHUNK_SIZE * 2.0);
    let y = height_map.pos_to_height(x, z).unwrap_or(0.0) + 30.0;
    let pos = Vec3::new(x - w / 2.0, y, z - CHUNK_SIZE / 2.0);

    let material_handle = materials.add(StandardMaterial {
        ..default()
    });

    let size = rng.gen_range(8.0..22.0);

    commands
        .spawn((
            Name::new("Powerup"),
            OnGameScreen,
            Transform::from_translation(pos),
            Mesh3d(meshes.add(Cuboid::new(size, size, size))),
            MeshMaterial3d(material_handle),
            PowerupSensor,
            Collider::cuboid(size, size, size),
            CollisionLayers::new(
                [CollisionLayer::Sensors],
                [
                    CollisionLayer::Stone,
                ]
            ),

        ));
    }
}


fn detect_collisions(
    mut collision_event_reader: EventReader<CollisionStarted>,
    stone: Query<Entity, With<Stone>>,
    powerups: Query<Entity, (With<PowerupSensor>, Without<Stone>)>,
    mut commands: Commands
) {
    let Ok(stone) = stone.get_single() else { return; };
//    let Ok(powerup) = powerup.get_single() else { return; };

    for CollisionStarted(e1, e2) in collision_event_reader.read() {
        if *e1 == stone || *e2 == stone {
            for powerup in powerups.iter() {
                let hit_powerup = *e1 == powerup || *e2 == powerup;
                if hit_powerup {
                    println!(
                        "Powerup hit {} and {} are colliding- {} ",
                        e1,
                        e2,
                        hit_powerup
                    );
                    commands.entity(powerup).despawn();
                }
            }
        }
    }
}
