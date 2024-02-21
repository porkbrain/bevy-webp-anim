//! If you want to run this, you need bevy default-features which are not
//! enabled by the lib.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy_webp_anim::Plugin)
        .init_resource::<bevy_webp_anim::WebpAnimator>()
        .add_systems(Startup, (spawn_camera, spawn_video))
        .add_systems(
            Update,
            (
                // the generic allows you to have multiple `WebpAnimator<T>`
                bevy_webp_anim::systems::start_loaded_videos::<()>,
                bevy_webp_anim::systems::load_next_frame,
            ),
        )
        .run()
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_video(
    mut webp: ResMut<bevy_webp_anim::WebpAnimator>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let fps = 30.0;

    commands.spawn(bevy_webp_anim::WebpBundle {
        remote_control: webp
            .add_and_wait_for_asset_load(asset_server.load("bunny.webp"), fps),
        sprite: Sprite {
            // Because the handle is empty when created and the rendering
            // pipeline doesn't update the size when the actual video
            // frames are being loaded into the handle, we inform the
            // pipeline about the size.
            // Otherwise, if the center of the video goes off screen,
            // it won't be rendered at all.
            custom_size: Some(Vec2::splat(32.0)),
            ..default()
        },
        ..default()
    });
}
