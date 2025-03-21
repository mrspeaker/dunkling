use bevy::math::Vec3;

pub const SHOW_DBG: bool = false;

pub const STONE_RADIUS: f32 = 10.0; // in metres
pub const STONE_DAMPENING: f32 = 0.04; // default: 0.0
pub const STONE_ANGULAR_DAMPENING: f32 = 0.04; // default: 0.0
pub const STONE_ANGULAR_DAMPENING_INC_START_AT: f32 = 20.0;
pub const STONE_ANGULAR_DAMPENING_INC_MAX_Y: f32 = 20.0;
pub const STONE_ANGULAR_DAMPENING_INC_AMOUNT: f32 = 0.015; // * dt
pub const STONE_MAX_VEL: f32 = 500.0;
pub const STONE_STOP_VEL: f32 = 0.5;

pub const CHUNK_SIZE: f32 = 400.0;
pub const NUM_CHUNKS: i32 = 15;
pub const CELL_SIZE: usize = 140;

pub const SHEET_TOTAL: f32 = CHUNK_SIZE * NUM_CHUNKS as f32;
pub const SHEET_PRE_AREA: f32 = 50.0;

pub const STONE_X: f32 = 0.0;
pub const STONE_Y: f32 = 200.0;
pub const STONE_Z: f32 = -CHUNK_SIZE + SHEET_PRE_AREA;

pub const TARGET_CENTRE: Vec3 = Vec3::new(0.0, 0.0, SHEET_TOTAL - CHUNK_SIZE);

pub const MAX_TERRAIN_HEIGHT: f32 = 20.0;

pub const MIN_SCULT_DIST_FROM_STONE: f32 = 18.0;

pub const STONE_HURL_POWERUP_TIME: f32 = 3.0; // seconds
pub const STONE_HURL_TIME_TO_POWER_MULTIPLIER: f32 = 150.0;
pub const STONE_HURL_AIM_ANGLE_MULTIPLIER: f32 = 200.0;

pub const SCULPT_RAISE_POWER: f32 = 0.5;
pub const SCULPT_LOWER_POWER: f32 = 0.5;
