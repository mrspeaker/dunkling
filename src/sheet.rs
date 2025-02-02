use avian3d::prelude::*;
use bevy::{
    prelude::*,
    pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    render::{
        mesh::{Indices, VertexAttributeValues, PlaneMeshBuilder},
        render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    }
};

use std::f32::consts::*;
use rand::prelude::*;

use crate::constants::{
    STONE_RADIUS,
    CELL_WIDTH,
    CELL_LENGTH,
    SHEET_LENGTH,
    SHEET_WIDTH,
    SHEET_PRE_AREA,
};

use crate::game::{GameState, OnGameScreen};

#[derive(Component)]
pub struct Sheet;


#[derive(Debug, Event)]
pub struct TerrainSculpt {
    pub up: bool,
    pub idx: usize,
}

#[derive(Resource, Clone, Debug)]
pub struct HeightMap {
    pub map: Vec<Vec<f32>>,
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
    // start line
    commands.spawn((
        OnGameScreen,
        Mesh3d(meshes.add(Cuboid::new(STONE_RADIUS * 10.0, 0.1, STONE_RADIUS * 0.5))),
        MeshMaterial3d(materials.add(Color::BLACK)),
    ));


    let map: Vec<Vec<f32>> = vec![vec![0.0; CELL_WIDTH]; CELL_LENGTH];
    dbg!(CELL_WIDTH, CELL_LENGTH);
    let mut height_map = HeightMap{map};

    let mut plane = build_plane( Plane3d::default()
        .mesh()
        .size(SHEET_WIDTH, SHEET_LENGTH)
        );//.build();
    terraform(&mut plane, &mut height_map);

    commands.insert_resource(height_map);

    let mat = StandardMaterial {
        base_color: Color::linear_rgb(0.2,0.4, 0.0),
        perceptual_roughness: 0.1,
        ..default()
    };

    //let cube_mesh_handle: Handle<Mesh> = meshes.add(create_plane_mesh());
    commands.spawn((
        OnGameScreen,
        //Mesh3d(cube_mesh_handle),
        Mesh3d(meshes.add(plane)),
        RigidBody::Static,
        Friction::new(10.0),
        //Collider::cuboid(width, 0.3, length),
        //ColliderConstructor::ConvexHullFromMesh,
        ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES),
        CollisionMargin(0.05),
        MeshMaterial3d(materials.add(mat)),
        Transform::from_xyz(
            0.0,
            0.0,
            -SHEET_LENGTH / 2.0 + (SHEET_PRE_AREA / 2.0) ),
            //.with_rotation(Quat::from_rotation_y(PI / 4.001)),
        Wireframe,
        Sheet
    ));

    let mut rng = rand::thread_rng();
    for _ in 0..200 {
        let pos = Vec3::new(
            rng.gen_range((-SHEET_WIDTH / 4.0)..(SHEET_WIDTH/ 4.0)),
            0.0,
            rng.gen_range((-SHEET_LENGTH)..0.0)
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
    let map = &mut height_map.map;
    (*map)[hm_y][hm_x] = value;
    verts[hm_y * CELL_WIDTH + hm_x][1] = value;
}

fn add_height(hm_x: usize, hm_y: usize, value: f32, height_map: &mut HeightMap, verts: &mut Vec<[f32; 3]>) {
    let map = &mut height_map.map;
    let cur = (*map)[hm_y][hm_x];
    let next = (cur + value).max(0.0);
    (*map)[hm_y][hm_x] = next;
    verts[hm_y * CELL_WIDTH + hm_x][1] = next;
}


pub fn terrain_sculpt(
    trigger: Trigger<TerrainSculpt>,
    mesh_query: Query<(Entity, &Mesh3d), With<Sheet>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut height_map: ResMut<HeightMap>,
    mut commands: Commands,
) {
    let (e, mesh_handle) = mesh_query.get_single().expect("Query not successful");
    let mesh = meshes.get_mut(mesh_handle).unwrap();
    let uv_attribute = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();

    let VertexAttributeValues::Float32x3(vert_pos) = uv_attribute else {
        panic!("Unexpected vertex format, expected Float32x3.");
    };

    let v = trigger.event().idx;
    let up = trigger.event().up;

    let x = v % CELL_WIDTH;
    let y = (v / CELL_WIDTH) as usize;
    let h = STONE_RADIUS * 0.5 * if up { 1.0 } else { -1.0 };

    add_height(x, y, h, &mut *height_map, &mut *vert_pos);
    let ns = get_neighbours(x, y);
    for (x, y) in ns {
        add_height(x, y, h /2.0, &mut *height_map, &mut *vert_pos);
    }

    mesh.compute_normals();

    commands.entity(e).remove::<Collider>();
    commands.entity(e).insert(ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES));

}

fn get_neighbours(x: usize, z: usize) -> Vec<(usize, usize)> {
    let w = CELL_WIDTH;
    let l = CELL_LENGTH;
    let mut ns: Vec<(usize,usize)> = vec![];

    if x > 0 {
        ns.push((x - 1, z)); // left
    }
    if x < w {
        ns.push((x + 1, z)); // right
    }
    if z > 0 {
        ns.push((x, z - 1)); // back
    }
    if z < l {
        ns.push((x, z + 1))
    }
    return ns;
}

fn terraform(mesh: &mut Mesh, map: &mut HeightMap) {
    let vert = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();

    let VertexAttributeValues::Float32x3(vert_pos) = vert else {
        panic!("Unexpected vertex format, expected Float32x3.");
    };

    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        let x = rng.gen_range(0..CELL_WIDTH-1);
        let y = rng.gen_range(0..CELL_LENGTH-1);
        let h = rng.gen_range(1.0..STONE_RADIUS*0.8);
        set_height(x, y, h, map, vert_pos);

        let ns = get_neighbours(x, y);
        for (x, y) in ns {
            set_height(x, y, h /2.0, map, vert_pos);
        }
    }
    mesh.compute_normals();
}


fn build_plane(mb: PlaneMeshBuilder) -> Mesh {
    let size = mb.plane.half_size * 2.0;
    let z_vertex_count = CELL_LENGTH as u32;
    let x_vertex_count = CELL_WIDTH as u32;
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
