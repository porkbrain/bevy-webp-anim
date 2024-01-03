//! Exports systems that are necessary to run for the plugin to function
//! correctly.
//!
//! The plugin does not automatically register the systems for you in case you
//! wanted to customize run conditions for them.

use bevy::prelude::*;
use tokio::sync::mpsc::error::TryRecvError;

use crate::{RemoteControl, WebpAnimator, WebpVideo};

/// Run this on update to load the next frame of the webp video.
/// It respects the FPS settings of the animation.
pub fn load_next_frame(
    mut remotes: Query<(Entity, &mut RemoteControl)>,
    mut handles: Query<&mut Handle<Image>, With<RemoteControl>>,
    mut images: ResMut<Assets<Image>>,
    mut animator: ResMut<WebpAnimator>,
    time: Res<Time>,
) {
    for (entity, mut remote_control) in remotes.iter_mut() {
        remote_control.fps_tracker.tick(time.delta());

        if !remote_control.fps_tracker.finished() {
            continue;
        }

        match animator.try_next_frame_for(&remote_control) {
            None => {
                // video not loaded yet
            }
            Some(Ok(next_frame)) => {
                if let Ok(mut handle) = handles.get_mut(entity) {
                    *handle = images.add(next_frame);
                } else {
                    debug!("{}: image handle not found", remote_control.id);
                }
            }
            Some(Err(TryRecvError::Empty)) => {
                trace!("{}: frame skipped", remote_control.id);
            }
            Some(Err(TryRecvError::Disconnected)) => {
                error!("{}: video channel disconnected", remote_control.id);
            }
        }
    }
}

/// Register this over `T` along with the [`WebpAnimator<T>`] resource.
/// You can then use [`WebpAnimator:add`] to spawn new videos.
///
/// Since videos are loaded asynchronously, you need to call this and keep
/// checking each video.
/// As soon as the video is loaded, we spawn a decoder task for it.
pub fn start_loaded_videos<T: 'static + Send + Sync>(
    mut animator: ResMut<WebpAnimator<T>>,
    videos: Res<Assets<WebpVideo>>,
) {
    let WebpAnimator {
        runtime,
        videos_to_queue,
        remote_control_receivers,
        prepared_frames_count,
        ..
    } = &mut *animator;

    videos_to_queue.retain(|(for_remote_control_id, video_handle)| {
        if let Some(video) = videos.get(video_handle) {
            let (animation_frames, next_frame_receiver) =
                tokio::sync::mpsc::channel(*prepared_frames_count);

            remote_control_receivers
                .insert(*for_remote_control_id, next_frame_receiver);

            let task = video.clone().produce(animation_frames);
            runtime.spawn(task);

            false
        } else {
            // not loaded yet, retain
            true
        }
    });
}
