use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
};
use thiserror::Error;

use crate::WebpVideo;

/// Loads `.webp` files into [`WebpVideo`] assets.
#[derive(Default, Debug)]
pub struct WebpLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("Cannot read webp video file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for WebpLoader {
    type Asset = WebpVideo;
    type Settings = ();
    type Error = LoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let label = load_context.path().display().to_string();

            Ok(Self::Asset { bytes, label })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["webp"]
    }
}
