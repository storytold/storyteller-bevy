use std::{io::ErrorKind, path::Path, sync::Arc};

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

    async fn load<'a>(&'a self, path: &'a Path) -> Result<Box<Reader<'a>>, AssetReaderError> {
        let token = path
            .ext_file_prefix()
            .ok_or_else(|| {
                AssetReaderError::Io(Arc::new(std::io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to parse token from '{path:?}'"),
                )))
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
            other => Err(AssetReaderError::Io(Arc::new(std::io::Error::new(
                ErrorKind::Other,
                format!("Status Code from endpoint: {other}"),
            )))),
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
            other => Err(AssetReaderError::Io(Arc::new(std::io::Error::new(
                ErrorKind::Other,
                format!("Status Code from bucket URL: {other}"),
            )))),
        }?;

        let reader: Box<Reader> = Box::new(VecReader::new(bytes));
        Ok(reader)
    }

    #[cfg(all(feature = "wasm", target_family = "wasm"))]
    async fn load_local<'a>(&'a self, path: &'a Path) -> Result<Box<Reader<'a>>, AssetReaderError> {
        let token = path
            .ext_file_prefix()
            .ok_or_else(|| {
                AssetReaderError::Io(Arc::new(std::io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to parse token from '{path:?}'"),
                )))
            })?
            .to_str()
            .unwrap();

        let bytes = wasm::LoadLocalFromIdb::new(token);
        info!("Sent request to bevy");
        let bytes = bytes.await.map_err(|e| {
            AssetReaderError::Io(Arc::new(std::io::Error::new(
                ErrorKind::Other,
                format!("{e:?}"),
            )))
        })?;

        let reader: Box<Reader> = Box::new(VecReader::new(bytes));
        Ok(reader)
    }
}

impl AssetReader for RemoteAssetReader {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        #[cfg(not(all(feature = "wasm", target_family = "wasm")))]
        {
            info!("wasm mode not detected");
            // NB: Headless mode (server rendering) is running without a Tokio runtime.
            // The reqwest HTTP client requires async-compat to function at runtime.
            Box::pin(async_compat::Compat::new(
                async move { self.load(path).await },
            ))
        }

        #[cfg(all(feature = "wasm", target_family = "wasm"))]
        {
            info!("wasm mode");
            if let Some(window) = web_sys::window() {
                if let Ok(search) = window.location().search() {
                    if search.contains("mock-api") {
                        return Box::pin(async move { self.load_local(path).await });
                    }
                }
            }
            Box::pin(async move { self.load(path).await })
        }
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
        AssetReaderError::Io(Arc::new(std::io::Error::new(
            ErrorKind::Other,
            format!("{self}"),
        )))
    }
}

#[cfg(all(feature = "wasm", target_family = "wasm"))]
mod wasm {
    use std::task::Waker;

    use bevy::log::info;
    use dashmap::DashMap;
    use futures::Future;
    use js_sys::Uint8Array;
    use once_cell::sync::Lazy;
    use wasm_bindgen::{prelude::*, JsValue};
    use web_sys::{CustomEvent, CustomEventInit};

    #[wasm_bindgen(typescript_custom_section)]
    const TIMELINE_TYPES: &str = r#"
    export interface RequestFileEvent extends CustomEvent {
        type: "request-file";
        detail: string;
    }

    declare global {
        export interface GlobalEventHandlersEventMap {
            "request-file": RequestFileEvent;
        }
    }
    "#;

    // DashMap + Lazy allow us to create static, global hashmaps that work well concurrently
    static FILE_WAKERS: Lazy<DashMap<String, Waker>> = Lazy::new(DashMap::new);
    static FILE_RESPONSES: Lazy<DashMap<String, Result<Vec<u8>, String>>> = Lazy::new(DashMap::new);

    /// Notifies us that the requested file couldn't be received
    #[wasm_bindgen(js_name = requestedFileFailed)]
    pub fn requested_file_failed(token: String, error: JsValue) {
        FILE_RESPONSES.insert(token.clone(), Err(format!("Error: {error:?}")));
        let waker = FILE_WAKERS.remove(&token);
        if let Some((_, waker)) = waker {
            waker.wake();
        }
    }

    /// Provides us with the requested file's contents
    #[wasm_bindgen(js_name = requestedFileReceived)]
    pub fn requested_file_received(token: String, file: Uint8Array) {
        info!("Received File From Mock API {token}");
        FILE_RESPONSES.insert(token.clone(), Ok(file.to_vec()));
        let waker = FILE_WAKERS.remove(&token);
        if let Some((_, waker)) = waker {
            waker.wake();
        }
    }

    pub enum LoadLocalFromIdb {
        RequestingFile { token: String },
        CheckingForFile { token: String },
    }

    impl LoadLocalFromIdb {
        pub fn new(file: impl ToString) -> Self {
            Self::RequestingFile {
                token: file.to_string(),
            }
        }
    }

    // We are implementing Future ourselves here because all the existing IndexedDb rust libraries
    // rely on JsValue, which is not Send - preventing us from using it in a BoxedFuture.
    // Instead, we ask the JS front end to grab the data from our mock API, and return it to us.
    impl Future for LoadLocalFromIdb {
        type Output = Result<Vec<u8>, String>;

        fn poll(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            match self.as_ref().get_ref() {
                LoadLocalFromIdb::RequestingFile { token } => {
                    info!("Requesting file {token}");
                    let Some(window) = web_sys::window() else {
                        return std::task::Poll::Ready(Err(
                            "Couldn't find browser window".to_string()
                        ));
                    };
                    let event = CustomEvent::new_with_event_init_dict(
                        "request-file",
                        CustomEventInit::new()
                            .bubbles(true)
                            .cancelable(true)
                            .detail(&JsValue::from_str(token)),
                    )
                    .map_err(|err| format!("{err:?}"));
                    let event = match event {
                        Ok(e) => e,
                        Err(err) => {
                            return std::task::Poll::Ready(Err(err));
                        }
                    };
                    FILE_WAKERS.insert(token.clone(), cx.waker().clone());
                    if let Err(err) = window
                        .document()
                        .ok_or(JsValue::null())
                        .and_then(|document| document.dispatch_event(&event))
                    {
                        return std::task::Poll::Ready(Err(format!(
                            "Error Dispatching Message: {err:?}"
                        )));
                    }
                    self.set(LoadLocalFromIdb::CheckingForFile {
                        token: token.clone(),
                    });
                    std::task::Poll::Pending
                }
                LoadLocalFromIdb::CheckingForFile { token } => {
                    info!("Checking on file {token}");
                    match FILE_RESPONSES.remove(token) {
                        Some((_, value)) => {
                            info!("Found {token}");
                            std::task::Poll::Ready(value)
                        }
                        None => std::task::Poll::Pending,
                    }
                }
            }
        }
    }
}
