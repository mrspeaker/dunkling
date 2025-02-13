pub const STONE_RADIUS: f32 = 10.0;
pub const STONE_DAMPENING: f32 = 0.05;
pub const STONE_MAX_VEL: f32 = 400.0;
pub const STONE_STOP_VEL: f32 = 4.0;

pub const CHUNK_SIZE: f32 = 400.0;
pub const NUM_CHUNKS: i32 = 11;
pub const SHEET_TOTAL: f32 = CHUNK_SIZE * NUM_CHUNKS as f32;

pub const SHEET_RATIO: f32 = 1.0;//SHEET_TOTAL / CHUNK_SIZE;
pub const CELL_WIDTH: usize = 100;
pub const CELL_LENGTH: usize = CELL_WIDTH * (SHEET_RATIO as usize);
pub const SHEET_PRE_AREA: f32 = 50.0;

pub const STONE_X: f32 = 0.0;
pub const STONE_Y: f32 = 150.0;
pub const STONE_Z: f32 = -CHUNK_SIZE + SHEET_PRE_AREA;

pub const MAX_TERRAIN_HEIGHT: f32 = 50.0;

