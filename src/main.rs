mod game;
pub mod camera;
pub mod chunk;
pub mod constants;
pub mod height_map;
pub mod player;
pub mod powerups;
pub mod sheet;
pub mod splash;
pub mod stone;
pub mod timey;
pub mod townsfolk;

#[cfg(test)]
mod tests;

use bevy::{
    prelude::*,
    render::{
        render_resource::WgpuFeatures,
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    }
};

use game::GamePlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.1)))
        .add_plugins(
            DefaultPlugins
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        features: WgpuFeatures::POLYGON_MODE_LINE,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
        )
        .add_plugins(GamePlugin)
        .run();
}
