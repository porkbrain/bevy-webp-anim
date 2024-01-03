#![doc = include_str!("../README.md")]

pub mod loader;
pub mod systems;
pub mod types;

use bevy::{app::App, asset::AssetApp};
pub use loader::*;
pub use types::*;

/// Registers the webp asset type.
///
/// # Important
/// Does not register any system.
/// You ought to register them yourself.
/// See the README or the [`systems`] module.
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<WebpLoader>()
            .init_asset::<WebpVideo>();
    }

    fn finish(&self, _app: &mut App) {
        //
    }
}
