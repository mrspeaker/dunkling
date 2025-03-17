use avian3d::prelude::*;
use bevy::{
    prelude::*,
    render::mesh::VertexAttributeValues
};

use crate::game::{OnGameScreen, CollisionLayer};
use crate::height_map::HeightMap;
use crate::sheet::Sheet;
use crate::constants::{
    CELL_SIZE,
    CHUNK_SIZE,
    NUM_CHUNKS,
};

pub struct SpawnChunk {
    pub pos: IVec2,
}

impl Command for SpawnChunk {
    fn apply(self, world: &mut World) {
        let xo = self.pos.x * CELL_SIZE as i32;
        let yo = self.pos.y * CELL_SIZE as i32;

        let mut plane = Plane3d::default()
            .mesh()
            .size(CHUNK_SIZE, CHUNK_SIZE)
            .subdivisions(CELL_SIZE as u32 - 2)
            .build();

        let hm = world
            .get_resource_mut::<HeightMap>()
            .expect("Height map should exist");

        sync_chunk_with_heightmap(&mut plane, &hm, xo, yo);

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
            Name::new("Chunk"),
            OnGameScreen,
            Mesh3d(mesh),
            RigidBody::Static,
            Friction::new(1.0),
            ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES),
            //CollisionMargin(0.01),
            CollisionLayers::new(
                [CollisionLayer::Terrain],
                [CollisionLayer::Stone, CollisionLayer::Townsfolk]
            ),
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

pub fn sync_chunk_with_heightmap(mesh: &mut Mesh, map: &HeightMap, xo: i32, yo: i32) {
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

pub fn vert_height_to_color(cols: &Vec<[f32; 3]>) -> Vec<[f32; 4]> {
    cols
        .iter()
        .map(|[_, h, _]| {
            let h = *h; // terrain_height;
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
