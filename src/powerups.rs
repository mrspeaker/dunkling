use bevy::prelude::*;
use avian3d::prelude::{Collider, CollisionLayers, CollisionStarted, LinearVelocity};
use crate::{sheet::TerrainCreated, constants::{CHUNK_SIZE, SHEET_TOTAL}, height_map::HeightMap, game::{OnGameScreen, CollisionLayer, GameState}, stone::Stone};
use rand::prelude::*;


#[derive(Component)]
struct PowerupSensor(Vec3);

#[derive(Debug, Event)]
pub struct PowerupHit {
    speed: Vec3
}


pub fn powerups_plugin(app: &mut App) {
    app.add_systems(Update, detect_collisions.run_if(in_state(GameState::InGame)));
    app.add_observer(spawn_powerups);
    app.add_observer(on_powerup_hit);
}

pub fn spawn_powerups(
    _trigger: Trigger<TerrainCreated>,
    mut commands: Commands,
    height_map: Res<HeightMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::rng();
    let w = CHUNK_SIZE;

    let material_handle = materials.add(StandardMaterial {
        ..default()
    });

    for _ in 0..30 {

        let x = rng.random_range(0.0..w); // right(0) to left (w)
        let z = rng.random_range(0.0..SHEET_TOTAL - CHUNK_SIZE * 2.0);
        let y = height_map.pos_to_height(x, z).unwrap_or(0.0) + rng.random_range(-20.0..35.0);
        let pos = Vec3::new(x - w / 2.0, y, z - CHUNK_SIZE / 2.0);

        let size = rng.random_range(8.0..22.0);

        let powerup_x = rng.random_range(-100.0..100.0);

    commands
        .spawn((
            Name::new("Powerup"),
            OnGameScreen,
            Transform::from_translation(pos),
            Mesh3d(meshes.add(Cuboid::new(size, size, size))),
            MeshMaterial3d(material_handle.clone()),
            PowerupSensor(Vec3::new(powerup_x, 0.0, 20.0)),
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
    powerups: Query<(Entity, &PowerupSensor), Without<Stone>>,
    mut commands: Commands
) {
    let Ok(stone) = stone.get_single() else { return; };

    for CollisionStarted(e1, e2) in collision_event_reader.read() {
        if *e1 == stone || *e2 == stone {
            for (pe, powerup) in powerups.iter() {
                let hit_powerup = *e1 == pe || *e2 == pe;
                if hit_powerup {
                    println!(
                        "Powerup hit {} and {} are colliding- {} ",
                        e1,
                        e2,
                        hit_powerup
                    );
                    commands.trigger(PowerupHit { speed: powerup.0 });
                    commands.entity(pe).despawn();

                }
            }
        }
    }
}

fn on_powerup_hit(
    trigger: Trigger<PowerupHit>,
    mut stone: Query<&mut LinearVelocity, With<Stone>>,
) {
    let Ok(mut vel_vec) = stone.get_single_mut() else { return; };
    let acc = trigger.event().speed;
    vel_vec.x += acc.x;
    vel_vec.y += acc.y;
    vel_vec.z += acc.z;
}
