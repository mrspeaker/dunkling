use avian3d::prelude::*;
use bevy::{
    image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    math::Affine2,
    prelude::*,
    pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    render::{
        mesh::{Indices, VertexAttributeValues, PlaneMeshBuilder},
        render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    }
};
use perlin_noise::PerlinNoise;
// use std::f32::consts::*;

use rand::prelude::*;

use crate::constants::{
    CELL_SIZE,
    SHEET_PRE_AREA,
    CHUNK_SIZE,
    STONE_RADIUS,
    SHEET_TOTAL,
    NUM_CHUNKS
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

struct SpawnTerrain {
    pos: IVec2,
    bumpiness: f32,
}

#[derive(Resource)]
struct PerlinInst(Box<PerlinNoise>);
impl PerlinInst {
    pub fn new() -> Self {
        let perlin_noise = Box::new(PerlinNoise::new());
        Self(perlin_noise)
    }
}

impl Command for SpawnTerrain {
    fn apply(self, world: &mut World) {

        //let mut hm = HeightMap::new(CHUNK_SIZE, CHUNK_SIZE, CELL_SIZE, CELL_SIZE);
        let xo = self.pos.x * CELL_SIZE as i32;
        let yo = self.pos.y * CELL_SIZE as i32;

        let mut plane = build_plane(Plane3d::default()
                                    .mesh()
                                    .size(CHUNK_SIZE, CHUNK_SIZE)
        );

        let hm = world
            .get_resource_mut::<HeightMap>()
            .expect("Hieght map should exist");

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

        // Stop from falling through ground
        world.spawn((
            OnGameScreen,
            RigidBody::Static,
            Friction::new(10.0),
            Mesh3d(mesh_underground),
            MeshMaterial3d(mat.clone()),
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
    let plane = build_plane(Plane3d::default()
        .mesh()
        .size(CHUNK_SIZE, CHUNK_SIZE)
    );
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

    // Create the height map then spawn the chunks
    let height_map = HeightMap::new(
        CHUNK_SIZE,
        CHUNK_SIZE * NUM_CHUNKS as f32,
        CELL_SIZE,
        CELL_SIZE * NUM_CHUNKS as usize);

    commands.insert_resource(height_map);

    for i in 0..NUM_CHUNKS {
        commands.queue(SpawnTerrain{
            pos: IVec2::new(0, i),
            bumpiness: if i == NUM_CHUNKS - 1 { 0.0 } else { i as f32 / NUM_CHUNKS as f32 }
        });
    }

    // Endzone chunk (the target)
    let endzone_mat = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("target.png")),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(CHUNK_SIZE, CHUNK_SIZE))),
        MeshMaterial3d(endzone_mat),
        Transform::from_xyz(0.0, 0.5, SHEET_TOTAL - CHUNK_SIZE)
            .with_rotation(Quat::from_euler(
                // YXZ = "yaw"/"pitch"/"roll"
                EulerRot::YXZ,
                (180.0_f32).to_radians(),
                (-90.0_f32).to_radians(),
                (0.0_f32).to_radians(),
            )),
        OnGameScreen
    ));


    // Add some trees
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let pos = Vec3::new(
            rng.gen_range((-CHUNK_SIZE / 4.0)..(CHUNK_SIZE/ 4.0)),
            0.0,
            rng.gen_range(0.0..SHEET_TOTAL - CHUNK_SIZE)
        );

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

fn set_height(hm_x: usize, hm_y: usize, value: f32, height_map: &mut HeightMap, verts: &mut Vec<[f32; 3]>) {
    if hm_x >= height_map.cell_w ||
        hm_y >= height_map.cell_h {
           return;
        }
    let map = &mut height_map.map;
    (*map)[hm_y][hm_x] = value;
    verts[hm_y * CELL_SIZE + hm_x][1] = value;
}

fn add_height(hm_x: usize, hm_y: usize, value: f32, height_map: &mut HeightMap, verts: &mut Vec<[f32; 3]>, chunk_idx: usize) {
    if hm_x >= height_map.cell_w ||
        hm_y >= height_map.cell_h {
           return;
        }

    let map = &mut height_map.map;
    let zoff = hm_y + (chunk_idx * CELL_SIZE);
    let cur = (*map)[zoff][hm_x];
    let next = (cur + value).max(0.0);
    (*map)[zoff][hm_x] = next;
    let idx = hm_y * CELL_SIZE + hm_x;
    if idx < CELL_SIZE * CELL_SIZE {
        verts[idx][1] = next;
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

    let mesh = meshes.get_mut(mesh_handle).unwrap();
    let uv_attribute = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();

    let VertexAttributeValues::Float32x3(vert_pos) = uv_attribute else {
        panic!("Unexpected vertex format, expected Float32x3.");
    };

    let ev = trigger.event();
    let up = ev.up;
    let _vert = ev.idx;
    let point = ev.p1;

    let chunk_idx = (t.translation.z / CHUNK_SIZE).floor() as usize;

    // Get sheet position from world position
    let p1 = point - t.translation + Vec3::new(CHUNK_SIZE * 0.5, 0.0, CHUNK_SIZE * 0.5);
    let Some((c1x, c1y)) = height_map.get_cell_from_pos(p1.x, p1.z) else { return; };

    let h = STONE_RADIUS * 0.1 * if up { 0.5 } else { -1.0 };

    // change the heights of surrounding verts
    add_height(c1x, c1y, h * 0.3, &mut *height_map, &mut *vert_pos, chunk_idx);
    let ns = get_neighbours(c1x, c1y);
    for (x, y) in ns {
        add_height(x, y, h * 0.3, &mut *height_map, &mut *vert_pos, chunk_idx);
        let ns = get_neighbours(x, y);
        for (x, y) in ns {
            add_height(x, y, h * 0.3, &mut *height_map, &mut *vert_pos, chunk_idx);
            let ns = get_neighbours(x, y);
            for (x, y) in ns {
                add_height(x, y, h * 0.3, &mut *height_map, &mut *vert_pos, chunk_idx);
            }
        }
    }

    // Re-colorize the chunk verts
    let cols: Vec<[f32; 4]> = vert_pos
        .iter()
        .map(|[_, h, _]| {
            let h = *h;// / terrain_height;
            if h > 7.0 {
                Color::WHITE.to_linear().to_f32_array()
            } else if h > 1.0{
                Color::srgb(0.4, 0.3, 0.1)
                    .to_linear()
                    .to_f32_array()
            } else {
                Color::linear_rgb(0.26,0.9, 0.119)
                //Color::srgb(0.1, 0.5, 0.0)
                    .to_linear()
                    .to_f32_array()
            }
        })
        .collect();

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        cols,
    );

    mesh.compute_normals();
    commands.entity(e).remove::<Collider>();
    commands.entity(e).insert(ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES));

}

fn get_neighbours(x: usize, z: usize) -> Vec<(usize, usize)> {
    let mut ns: Vec<(usize,usize)> = vec![];

    if x > 0 {
        ns.push((x - 1, z)); // left
    }
    if x < CELL_SIZE {
        ns.push((x + 1, z)); // right
    }
    if z > 0 {
        ns.push((x, z - 1)); // back
    }
    if z < CELL_SIZE {
        ns.push((x, z + 1))
    }
    return ns;
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
    let cols: Vec<[f32; 4]> = vert_pos
        .iter()
        .map(|[_, h, _]| {
            let h = *h;// / terrain_height;
            if h > 7.0 {
                Color::WHITE.to_linear().to_f32_array()
            } else if h > 1.0{
                Color::srgb(0.4, 0.3, 0.1)
                    .to_linear()
                    .to_f32_array()
            } else {
                Color::linear_rgb(0.26,0.9, 0.119)
                //Color::srgb(0.1, 0.5, 0.0)
                    .to_linear()
                    .to_f32_array()
            }
        })
        .collect();

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        cols,
    );

    mesh.compute_normals();

}

fn build_plane(mb: PlaneMeshBuilder) -> Mesh {
    let size = mb.plane.half_size * 2.0;
    let z_vertex_count = CELL_SIZE as u32;
    let x_vertex_count = CELL_SIZE as u32;
    let num_vertices = (z_vertex_count * x_vertex_count) as usize;
    let num_indices = ((z_vertex_count - 1) * (x_vertex_count - 1) * 6) as usize;

    let mut positions: Vec<Vec3> = Vec::with_capacity(num_vertices);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_vertices);
    let mut indices: Vec<u32> = Vec::with_capacity(num_indices);

    let rotation = Quat::from_rotation_arc(Vec3::Y, *mb.plane.normal);

    for z in 0..z_vertex_count {
        for x in 0..x_vertex_count {
            let tx = x as f32 / (x_vertex_count - 1) as f32;
            let tz = z as f32 / (z_vertex_count - 1) as f32;
            let pos = rotation * Vec3::new((-0.5 + tx) * size.x, 0.0, (-0.5 + tz) * size.y);
            positions.push(pos);
            normals.push(mb.plane.normal.to_array());
            uvs.push([tx, tz]);
        }
    }

    for z in 0..z_vertex_count - 1 {
        for x in 0..x_vertex_count - 1 {
            let quad = z * x_vertex_count + x;
            indices.push(quad + x_vertex_count + 1);
            indices.push(quad + 1);
            indices.push(quad + x_vertex_count);
            indices.push(quad);
            indices.push(quad + x_vertex_count);
            indices.push(quad + 1);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
        .with_inserted_indices(Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}
