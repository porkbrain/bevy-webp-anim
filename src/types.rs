use std::{collections::BTreeMap, marker::PhantomData};

use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    utils::Uuid,
};
use image::{codecs::webp::WebPDecoder, AnimationDecoder};
use tokio::sync::mpsc::{Receiver, Sender};
pub use tokio::{runtime, sync::mpsc::error::TryRecvError};

/// See [`WebpAnimator::prepared_frames_count`].
pub const DEFAULT_PREPARED_FRAMES_COUNT: usize = 16;

/// Makes frames from .webp files available to the game.
///
/// `T` enables multiple independent resources should they be needed.
#[derive(Resource)]
pub struct WebpAnimator<T = ()> {
    /// Loads at most this many frames into buffer (sync channel) before
    /// awaiting consumption.
    /// In another words, the animation decoder runtime tries to keep the
    /// buffer for each video at this size.
    pub prepared_frames_count: usize,

    /// 1. Listen to the decoders that fill channels with frames.
    /// 2. Frames are read from the receivers and put into [`Handle<Image>`].
    /// 3. The handles are used to overwrite the old [`Handle<Image>`]
    ///    components of entities with [`RemoteControl`] component with
    ///    matching [`Uuid`].
    pub(crate) remote_control_receivers: BTreeMap<Uuid, Receiver<Image>>,
    /// The async runtime that performs the decoding work.
    pub(crate) runtime: runtime::Runtime,
    /// Start loading these videos as soon as possible.
    pub(crate) videos_to_queue: Vec<(Uuid, Handle<WebpVideo>)>,

    _phantom: PhantomData<T>,
}

#[derive(Bundle, Default)]
pub struct WebpBundle {
    pub remote_control: RemoteControl,

    // the rest is the same as [`SpriteBundle`]
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub texture: Handle<Image>,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

#[derive(Asset, TypePath, Clone)]
pub struct WebpVideo {
    pub bytes: Vec<u8>,
    pub label: String,
}

/// Controls the playback of a video.
/// Get this from [`WebpAnimator::add`].
///
/// # Important
/// Must be added to an entity which also contains [`Handle<Image>`].
/// Otherwise the frames will be lost.
#[derive(Component, Default)]
pub struct RemoteControl {
    /// Determines how often next frame is loaded.
    /// Also enables pausing the video.
    /// You can change this to your liking.
    /// However, if you set the mode to non-repeating, frames won't change.
    pub fps_tracker: Timer,

    /// This is how we associate the [`RemoteControl`] with the correct
    /// channel that produces frames.
    /// See [`WebpAnimator::targets`].
    pub(crate) id: Uuid,
}

impl<T> WebpAnimator<T> {
    pub fn new(prepared_frames_count: usize) -> Self {
        Self {
            prepared_frames_count,
            remote_control_receivers: BTreeMap::new(),
            runtime: runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .build()
                .unwrap(),
            videos_to_queue: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn new_with_runtime(
        prepared_frames_count: usize,
        runtime: runtime::Runtime,
    ) -> Self {
        Self {
            prepared_frames_count,
            runtime,
            remote_control_receivers: BTreeMap::new(),
            videos_to_queue: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Given handle, will wait for the asset to be loaded and then start
    /// the frame decoder.
    ///
    /// Associate the return value with an entity that contains a
    /// [`Handle<Image>`].
    pub fn add_and_wait_for_asset_load(
        &mut self,
        video: Handle<WebpVideo>,
        fps: f32,
    ) -> RemoteControl {
        let remote_control = RemoteControl::new(fps);

        self.videos_to_queue.push((remote_control.id, video));

        remote_control
    }

    /// Given the video data, will start the frame decoder immediately.
    ///
    /// Associate the return value with an entity that contains a
    /// [`Handle<Image>`].
    pub fn add_already_loaded(
        &mut self,
        video: WebpVideo,
        fps: f32,
    ) -> RemoteControl {
        let remote_control = RemoteControl::new(fps);

        let (animation_frames, next_frame_receiver) =
            tokio::sync::mpsc::channel(self.prepared_frames_count);

        self.remote_control_receivers
            .insert(remote_control.id, next_frame_receiver);

        let task = video.produce(animation_frames);
        self.runtime.spawn(task);

        remote_control
    }

    /// Returns the next frame for the given [`RemoteControl`].
    pub fn try_next_frame_for(
        &mut self,
        remote_control: &RemoteControl,
    ) -> Option<Result<Image, TryRecvError>> {
        let next_frame =
            self.remote_control_receivers.get_mut(&remote_control.id)?;

        Some(next_frame.try_recv())
    }
}

impl RemoteControl {
    pub fn set_fps(&mut self, fps: f32) {
        self.fps_tracker = Timer::from_seconds(1.0 / fps, TimerMode::Repeating);
    }

    pub fn new(fps: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            fps_tracker: Timer::from_seconds(1.0 / fps, TimerMode::Repeating),
        }
    }
}

impl<T> Default for WebpAnimator<T> {
    fn default() -> Self {
        Self::new(DEFAULT_PREPARED_FRAMES_COUNT)
    }
}

impl WebpVideo {
    pub async fn produce(self, animation_frames: Sender<Image>) {
        let WebpVideo { bytes, label } = self;

        // We decode each frame only once and then play them over and
        // over.
        // This is optimized for replaying and low CPU usage.
        //
        // TODO: Enable an alternative with lower memory usage and
        // faster startup times where we decode each frame every time
        // it is played.
        match WebPDecoder::new(bytes.as_slice())
            .and_then(|decoder| decoder.into_frames().collect_frames())
        {
            Ok(frames) => loop {
                for frame in &frames {
                    let (width, height) = frame.buffer().dimensions();
                    let image = Image::new(
                        Extent3d {
                            width,
                            height,
                            ..default()
                        },
                        TextureDimension::D2,
                        frame.clone().into_buffer().into_raw(),
                        TextureFormat::Rgba8Unorm,
                    );

                    // animation no longer required
                    if animation_frames.send(image).await.is_err() {
                        break;
                    }
                }
            },
            Err(e) => error!("Cannot load webp video {label}: {e}"),
        };
    }
}
