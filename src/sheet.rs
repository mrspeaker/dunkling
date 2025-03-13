use avian3d::prelude::*;
use bevy::{
    image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    math::Affine2,
    prelude::*,
    pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    render::mesh::VertexAttributeValues
};

use rand::prelude::*;

use crate::constants::{
    CELL_SIZE,
    SHEET_PRE_AREA,
    CHUNK_SIZE,
    SHEET_TOTAL,
    NUM_CHUNKS,
    SCULPT_RAISE_POWER,
    SCULPT_LOWER_POWER
};

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

struct SpawnTerrain {
    pos: IVec2,
    bumpiness: f32,
}

impl Command for SpawnTerrain {
    fn apply(self, world: &mut World) {

        //let mut hm = HeightMap::new(CHUNK_SIZE, CHUNK_SIZE, CELL_SIZE, CELL_SIZE);
        let xo = self.pos.x * CELL_SIZE as i32;
        let yo = self.pos.y * CELL_SIZE as i32;

        //meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))

        let mut plane = Plane3d::default()
            .mesh()
            .size(CHUNK_SIZE, CHUNK_SIZE)
            .subdivisions(CELL_SIZE as u32 - 2)
            .build();

        let hm = world
            .get_resource_mut::<HeightMap>()
            .expect("Height map should exist");

        sync_plane_with_heightmap(&mut plane, &hm, xo, yo);

        let mesh = world
            .get_resource_mut::<Assets<Mesh>>()
            .expect("Mesh Assets should exist")
            .add(plane);

        let mat = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .expect("StandardMaterial Assets to exist")
            .add(Color::WHITE);

        // Mesh chunk
        let mut ent = world.spawn((
            OnGameScreen,
            Mesh3d(mesh),
            RigidBody::Static,
            Friction::new(1.0),
            ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES),
            CollisionMargin(0.05),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(
                self.pos.x as f32 * CHUNK_SIZE,
                0.,
                self.pos.y as f32 * CHUNK_SIZE,
            ),
            //Wireframe,
        ));

        // Don't make the final target chunk a "Sheet".
        if self.pos.y != NUM_CHUNKS - 1 {
            ent.insert(Sheet);
        }

        let mesh_underground = world
            .get_resource_mut::<Assets<Mesh>>()
            .expect("Mesh Assets should exist")
            .add(Cuboid::new(CHUNK_SIZE, 50.0, CHUNK_SIZE));

        let mat_ground = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .expect("StandardMaterial Assets to exist")
            .add(Color::BLACK);

        // Stop from falling through ground
        world.spawn((
            OnGameScreen,
            RigidBody::Static,
            Friction::new(10.0),
            Mesh3d(mesh_underground),
            MeshMaterial3d(mat_ground),
            ColliderConstructor::Cuboid {
                x_length: CHUNK_SIZE,
                y_length: 50.0,
                z_length: CHUNK_SIZE
            },
            Transform::from_xyz(
                self.pos.x as f32 * CHUNK_SIZE,
                -25.05,
                self.pos.y as f32 * CHUNK_SIZE,
            ),
        ));

    }
}


pub struct SheetPlugin;

impl Plugin for SheetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WireframePlugin);
        app.insert_resource(WireframeConfig {
            global: false,
            default_color: Color::linear_rgb(0.1,0.1, 0.),
        });
        app.add_systems(OnEnter(GameState::InGame), setup);
        app.add_observer(terrain_sculpt);
    }
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

    // Create the height map then spawn the chunks
    let height_map = HeightMap::new(
        CHUNK_SIZE,
        CHUNK_SIZE * NUM_CHUNKS as f32,
        CELL_SIZE,
        CELL_SIZE * NUM_CHUNKS as usize);

    commands.insert_resource(height_map);
    commands.trigger(TerrainCreated);

    for i in 0..NUM_CHUNKS -1{
        commands.queue(SpawnTerrain{
            pos: IVec2::new(0, i),
            bumpiness: if i == NUM_CHUNKS - 1 { 0.0 } else { i as f32 / NUM_CHUNKS as f32 }
        });
    }

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

}

pub fn vert_height_to_color(cols: &Vec<[f32; 3]>) -> Vec<[f32; 4]> {
    cols
        .iter()
        .map(|[_, h, _]| {
            let h = *h;// / terrain_height;
            let col;
            if h > 40.0 {
                col = Color::srgb(1.0, 0.9, 1.0);
            } else if h > 18.0 {
                col = Color::srgb(0.6, 0.5, 0.0);
            } else if h > 7.0 {
                col = Color::srgb(0.4, 0.9, 0.1);
            } else if h > 1.0{
                col = Color::srgb(0.4, 0.8, 0.1);
            } else {
                col = Color::srgb(0.26,0.7, 0.119);
            }
            col.to_linear().to_f32_array()
        })
        .collect()
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
    //let uv_attribute = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();

    //let VertexAttributeValues::Float32x3(vert_pos) = uv_attribute else {
    //    panic!("Unexpected vertex format, expected Float32x3.");
    //};

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
    sync_plane_with_heightmap(&mut mesh, &height_map, 0, (chunk_idx * CELL_SIZE) as i32);

    // Re-add collider to match new terrain
    commands.entity(e).remove::<Collider>();
    commands.entity(e).insert(ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES));

}

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

fn sync_plane_with_heightmap(mesh: &mut Mesh, map: &HeightMap, xo: i32, yo: i32) {
    let vert = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();
    let VertexAttributeValues::Float32x3(vert_pos) = vert else {
        panic!("Unexpected vertex format, expected Float32x3.");
    };

    // Copy the heightmap values to the plane
    for y in 0..CELL_SIZE {
        for x in 0..CELL_SIZE {
            vert_pos[y * CELL_SIZE + x][1] = map.map[y + yo as usize][x + xo as usize] as f32;
        }
    }

    // Set the colors
    let cols = vert_height_to_color(vert_pos);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        cols,
    );

    mesh.compute_normals();
}
