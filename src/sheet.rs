use avian3d::prelude::*;
use bevy::{
    color::palettes::css::{SILVER, ORANGE},
    prelude::*,
    pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
};

use crate::{constants::{
    CELL_SIZE,
    CHUNK_SIZE,
    SHEET_TOTAL,
    NUM_CHUNKS,
    SCULPT_RAISE_POWER,
    SCULPT_LOWER_POWER, TARGET_CENTRE
}, stone::Stone};
use crate::chunk::{SpawnChunk, sync_chunk_with_heightmap};
use crate::game::{GameState, OnGameScreen};
use crate::height_map::HeightMap;

#[derive(Component)]
pub struct Sheet;

#[derive(Debug, Event)]
pub struct TerrainSculpt {
    pub up: bool,
    pub idx: usize,
    pub p1: Vec3,
}

#[derive(Debug, Event)]
pub struct TerrainCreated;

#[derive(Debug, Event)]
pub struct StoneInHole;

#[derive(Component)]
struct HoleSensor;

pub fn sheet_plugin(app: &mut App) {
    app.add_plugins(WireframePlugin);
    app.insert_resource(WireframeConfig {
        global: false,
        default_color: Color::linear_rgb(0.1,0.1, 0.),
    });
    app.add_systems(OnEnter(GameState::InGame), setup);
    app.add_systems(Update, detect_collisions);
    app.add_observer(terrain_sculpt);
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Add the initial slanty chunk mesh
    /*
    let plane = Plane3d::default()
        .mesh()
        .size(CHUNK_SIZE, CHUNK_SIZE)
        .build();

    let texture_handle = asset_server.load_with_settings(
        "textures/Ground037_2K-JPG_Color.jpg",
        |s: &mut _| {
            *s = ImageLoaderSettings {
                sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                    // rewriting mode to repeat image,
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    ..default()
                }),
                ..default()
            }
        });
    // this material renders the texture normally
    let uv_x = 2.0;
    let uv_y = uv_x;
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        perceptual_roughness: 0.8,
        uv_transform: Affine2::from_scale(Vec2::new(uv_x, uv_y)),
        ..default()
    });

    // slanty chunk
    commands.spawn((
        OnGameScreen,
        Mesh3d(meshes.add(plane)),
        RigidBody::Static,
        Friction::new(10.0),
        ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES),
        CollisionMargin(0.05),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(
            0.0,
            -2.0,
            -CHUNK_SIZE / 2.0 + (SHEET_PRE_AREA / 2.0) )
            .with_rotation(Quat::from_rotation_x(0.2)),
        //Wireframe,
        Sheet
));
    */

    // Create the height map then spawn the chunk meshes
    let height_map = HeightMap::new(
        CHUNK_SIZE,
        CHUNK_SIZE * NUM_CHUNKS as f32,
        CELL_SIZE,
        CELL_SIZE * NUM_CHUNKS as usize);

    commands.insert_resource(height_map);
    commands.trigger(TerrainCreated);

    for i in 0..NUM_CHUNKS -1{
        commands.queue(SpawnChunk {
            pos: IVec2::new(0, i)
        });
    }

    // Endzone hole
    commands
        .spawn((
            Name::new("Hole"),
            OnGameScreen,
            SceneRoot(
                asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("models/hole.glb"))),
            RigidBody::Static,
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
            Transform::from_xyz(0.0, 0.0, SHEET_TOTAL - CHUNK_SIZE)
        ));

    // Trigger inside hole
    let hole = commands.spawn((
        Mesh3d(meshes.add(Cylinder::default())),
        MeshMaterial3d(materials.add(Color::from(ORANGE))),
        Transform::from_translation(TARGET_CENTRE - (Vec3::Y * 40.0))
            .with_scale(Vec3::new(50.0, 5.0, 50.0)),
        OnGameScreen,
        RigidBody::Static,
        Collider::cylinder(0.5, 1.0),
        Sensor,
        HoleSensor
    ));
    dbg!(hole.id());

    // Endzone flag pole
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::default())),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
        Transform::from_translation(TARGET_CENTRE)
            .with_scale(Vec3::new(5.0, 200.0, 5.0)),
        OnGameScreen
    ));

}

fn detect_collisions(
    mut collision_event_reader: EventReader<CollisionStarted>,
    stone: Query<Entity, With<Stone>>,
    hole: Query<Entity, (With<HoleSensor>, Without<Stone>)>,
    mut commands: Commands
) {
    let Ok(stone) = stone.get_single() else { return; };
    let Ok(hole) = hole.get_single() else { return; };

    for CollisionStarted(e1, e2) in collision_event_reader.read() {
        if *e1 == stone || *e2 == stone {
            let in_hole = *e1 == hole || *e2 == hole;
            if in_hole {
                commands.trigger(StoneInHole);
            }
            println!(
                "Entities {} and {} are colliding- {} {}",
                e1,
                e2,
                hole,
                in_hole
            );
        }
    }
}

pub fn terrain_sculpt(
    trigger: Trigger<TerrainSculpt>,
    mesh_query: Query<(Entity, &Mesh3d, &Transform), With<Sheet>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut height_map: ResMut<HeightMap>,
    mut commands: Commands,
) {
    let Ok((e, mesh_handle, t)) = mesh_query.get(trigger.entity()) else {
        return;
    };
    let mut mesh = meshes.get_mut(mesh_handle).unwrap();

    let ev = trigger.event();
    let up = ev.up;
    let _vert = ev.idx;
    let point = ev.p1;

    let chunk_idx = (t.translation.z / CHUNK_SIZE).floor() as usize;

    // Get sheet position from world position
    let p1 = point - t.translation + Vec3::new(CHUNK_SIZE * 0.5, 0.0, CHUNK_SIZE * 0.5);
    let Some((c1x, c1y)) = height_map.get_cell_from_pos(p1.x, p1.z) else { return; };

    let h = if up { SCULPT_RAISE_POWER } else { -SCULPT_LOWER_POWER };

    // change the heights of surrounding verts
    let amount = 0.8;
    for n in get_neighbours_radius(c1x, c1y, 4) {
        let dist = 1.0 - (1.0 - n.2).powi(3);// n.2 * n.2; // 0 - 1
        height_map.add_height(n.0, n.1, h * amount * dist, chunk_idx);
    }
    sync_chunk_with_heightmap(&mut mesh, &height_map, 0, (chunk_idx * CELL_SIZE) as i32);

    // Re-add collider to match new terrain
    commands.entity(e).remove::<Collider>();
    commands.entity(e).insert(ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES));

}


/// Calculates the neighbours within the given radius around the point `(x, y)`
///
/// This function takes three arguments: two indices representing a point in a 2D grid,
/// and the radius of the circular area of interest. It returns a vector 
/// of tuples of the cell's x and y indicie,s and a normalized distance
/// from the reference point to the cell.
///
/// # Arguments
///
/// * `x` - The x-coordinate of the point.
/// * `y` - The y-coordinate of the point.
/// * `r` - The radius around the point.
///
/// # Returns
///
/// A Vector of tuples where each element is a tuple containing the x, y indices of a cell and its normalized distance from the reference point.
///
pub fn get_neighbours_radius(x: usize, y: usize, r: usize) -> Vec<(usize, usize, f32)> {
    let mut ns: Vec<(usize,usize,f32)> = vec![];
    let max_dist = ((r as f32 * r as f32) + (r as f32 * r as f32)).sqrt();
    for j in y.saturating_sub(r)..=y.saturating_add(r) {
        for i in x.saturating_sub(r)..=x.saturating_add(r) {
            let dist = ((i as f32 - x as f32).powi(2) + (j as f32 - y as f32).powi(2)).sqrt();
            ns.push((i, j, 1.0 - (dist / max_dist)));
        }
    }
    ns
}
