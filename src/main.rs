mod game;
pub mod camera;
pub mod constants;
pub mod sheet;
pub mod townsfolk;

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
        .add_plugins(
            DefaultPlugins.set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(GamePlugin)
        .run();
}
