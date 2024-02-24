use std::{io::ErrorKind, path::Path};

use bevy::{
    asset::io::{AssetReader, AssetReaderError, PathStream, Reader, VecReader},
    prelude::*,
    utils::BoxedFuture,
};
use reqwest::StatusCode;

use super::response::GetMediaFileSuccessResponse;
use crate::path::PathExtras;

pub struct RemoteAssetReader;

impl RemoteAssetReader {
    const API_BASE_URL: &'static str = "https://api.fakeyou.com/v1/media_files/file";
    const BUCKET_BASE_URL: &'static str = "https://storage.googleapis.com/vocodes-public";
}

impl AssetReader for RemoteAssetReader {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(async move {
            let token = path
                .ext_file_prefix()
                .ok_or_else(|| {
                    AssetReaderError::Io(std::io::Error::new(
                        ErrorKind::Other,
                        format!("Failed to parse token from '{path:?}'"),
                    ))
                })?
                .to_str()
                .unwrap();

            info!("Loading asset from media token: '{token}'");

            let response = reqwest::get(format!("{}/{token}", &Self::API_BASE_URL))
                .await
                .map_err(IntoError::<AssetReaderError>::into)?;

            let response = match response.status() {
                StatusCode::OK => response
                    .json::<GetMediaFileSuccessResponse>()
                    .await
                    .map_err(IntoError::<AssetReaderError>::into),
                other => Err(AssetReaderError::Io(std::io::Error::new(
                    ErrorKind::Other,
                    format!("Status Code from endpoint: {other}"),
                ))),
            }?;

            let url = format!(
                "{}{}",
                &Self::BUCKET_BASE_URL,
                &response.media_file.public_bucket_path
            );

            let response = reqwest::get(url)
                .await
                .map_err(IntoError::<AssetReaderError>::into)?;

            let bytes = match response.status() {
                // TODO: This is not thread-safe, but some variation of this
                //       would be better than the current solution once Bevy
                //       WASM becomes multi-threaded:
                // StatusCode::OK => Ok(response.bytes_stream().map(|result| {
                //     result.map_err(|err| std::io::Error::new(ErrorKind::Other, format!("{err}")))
                // })),
                StatusCode::OK => Ok(response
                    .bytes()
                    .await
                    .map_err(IntoError::<AssetReaderError>::into)?
                    .into_iter()
                    .collect::<Vec<_>>()),
                other => Err(AssetReaderError::Io(std::io::Error::new(
                    ErrorKind::Other,
                    format!("Status Code from bucket URL: {other}"),
                ))),
            }?;

            let reader: Box<Reader> = Box::new(VecReader::new(bytes));
            Ok(reader)
        })
    }

    fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        // TODO: Do we want to try to read .meta files?
        Box::pin(async { Err(AssetReaderError::NotFound(path.into())) })
    }

    fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<PathStream>, AssetReaderError>> {
        // We won't have to deal with directories
        Box::pin(async { Err(AssetReaderError::NotFound(path.into())) })
    }

    fn is_directory<'a>(&'a self, _: &'a Path) -> BoxedFuture<'a, Result<bool, AssetReaderError>> {
        // We won't have to deal with directories
        Box::pin(async { Ok(false) })
    }
}

trait IntoError<Output> {
    fn into(self) -> Output;
}

impl IntoError<AssetReaderError> for reqwest::Error {
    fn into(self) -> AssetReaderError {
        AssetReaderError::Io(std::io::Error::new(ErrorKind::Other, format!("{self}")))
    }
}
