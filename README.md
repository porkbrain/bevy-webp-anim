# `bevy_webp_anim`

Plugin for loading and playing animated `.webp` images in bevy.
We make the assumption that any webp file loaded with the provided `WebpLoader` is an animation.
The actual decoding of the frames is done by the [`image`] crate.

The decoding is ran in a `tokio::runtime::Runtime` threadpool.
The resource `WebpAnimator` holds a mapping from video uuids to channel receivers.
The runtime sends `bevy::Image` frames to the channel receivers.

Register the system `load_next_frame` to automatically continue the animation.
This system works with a core component `RemoteControl`.
This component contains the uuid of the video and FPS settings.
By running the `load_next_frame` system e.g. on `Update` or on a fixed schedule with period matching that of the FPS of the video, each `RemoteControl` component will load the next frame of the video into the entity's `Handle<Image>`.
If the entity does not have a `Handle<Image>` component, the frame is dropped.

## Issue: Support large videos

The current implementation of `bevy_webp_anim` loads each frame into memory.
Then, it keeps sending the frames from the that frames vector to the channel receiver in a loop.
This works for my use case as I have small videos which I want to play in a loop.

However, an alternative approach would facilitate larger videos.
We could store the decoder (from the `image` crate) and decode frames only as needed.
The trouble right now with implementing this is that spawning an async task requires `Send` futures.
However, the `Frames<'a>` iterator of the `image` crate has an inner `Box<dyn Iterator<Item = Frame<'a>> + 'a>` type.
We cannot keep the iterator across `.await` points of our channel sender `async fn send` calls.

## Feature: Reuse decoded frames

We can have several entities listening to the same video frames.
This would be useful if you play the same video in several places on your screen.
Right now, you'd need separate decoders and separate `Handle<Image>`.

I don't need this feature in my project at the moment, but happy to add it if there's a demand.
