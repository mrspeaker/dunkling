use avian3d::prelude::*;
use bevy::{
    prelude::*,
    pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    }
};
use std::f32::consts::*;
use rand::prelude::*;

use crate::constants::{
    STONE_RADIUS,
    SUBS,
    SHEET_LENGTH,
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

pub struct SheetPlugin;

impl Plugin for SheetPlugin {
    fn build(&self, app: &mut App) {
//        app.add_plugins(WireframePlugin);
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

    let mut plane = Plane3d::default().mesh().size(SHEET_LENGTH, SHEET_LENGTH).subdivisions(SUBS).build();
    rando_y(&mut plane);

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
            -SHEET_LENGTH / 2.0 + (SHEET_PRE_AREA / 2.0) )
            .with_rotation(Quat::from_rotation_y(PI / 4.001)),
        Wireframe,
        Sheet
    ));

    commands
        .spawn((
            Name::new("Tree"),
            OnGameScreen,
            SceneRoot(
                asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("models/tree.glb"))),
            RigidBody::Static,
            Collider::cuboid(1.0, 1.0, 1.7),
            Transform::from_xyz(0.0, 10.0, -SHEET_LENGTH / 2.0)));

}

pub fn terrain_sculpt(
    trigger: Trigger<TerrainSculpt>,
    mesh_query: Query<(Entity, &Mesh3d), With<Sheet>>,
    mut meshes: ResMut<Assets<Mesh>>,
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

    let amount = STONE_RADIUS * 0.5 * if up { 1.0 } else { -1.0 };

    // Modify the selected vert, plus the 4 around it (a bit less)
    let r1 = (SUBS + 2) as usize;
    vert_pos[v][1] = (vert_pos[v][1] + amount).max(0.0);
    if v > 0 {
        vert_pos[v-1][1] = (vert_pos[v-1][1] + amount / 2.0).max(0.0);
    }
    vert_pos[v+1][1] = (vert_pos[v+1][1] + amount / 2.0).max(0.0);
    if v > r1 {
        vert_pos[v-r1][1] = (vert_pos[v-r1][1]  + amount / 2.0).max(0.0);
    }
    vert_pos[v+r1][1] = (vert_pos[v+r1][1] + amount / 2.0).max(0.0);

    mesh.compute_normals();

    commands.entity(e).remove::<Collider>();
    commands.entity(e).insert(ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES));

}

fn get_neighbours(v: usize) -> Vec<usize> {
    let r1 = (SUBS + 2) as usize;
    let mut ns: Vec<usize> = vec![];

    if v > 0 {
        ns.push(v - 1);
    }
    ns.push(v + 1);
    if v > r1 {
        ns.push(v - r1);
    }
    ns.push(v + r1);
    return ns;
}

#[rustfmt::skip]
fn _create_plane_mesh() -> Mesh {
    let w = 5.0 / 2.0;
    let h = 50.0 / 2.0;

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-w, 0.0, -h],
            [w, 0.0, -h],
            [w, 0.0, h],
            [-w, 0.0, h],

            [w+w*2.0, 0.0, -h],
            [w+w*2.0, 0.0, h],
        ],
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![
            [0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0],
            [0.0, 1.0], [0.0, 0.0],
        ],
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],

            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
    )
    .with_inserted_indices(Indices::U32(vec![
        0,3,1, 1,3,2, // triangles making up the top (+y) facing side.
        1,2,4, 4,2,5
    ]))

}

fn rando_y(mesh: &mut Mesh) {
    let uv_attribute = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();

    let VertexAttributeValues::Float32x3(vert_pos) = uv_attribute else {
        panic!("Unexpected vertex format, expected Float32x3.");
    };

    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        let v = rng.gen_range(0..10000);
        let h = rng.gen_range(1.0..STONE_RADIUS*0.8);
        vert_pos[v][1] = h;
        let ns = get_neighbours(v);
        for v in ns {
            vert_pos[v][1] = h / 2.0;
        }
    }
    mesh.compute_normals();
}
