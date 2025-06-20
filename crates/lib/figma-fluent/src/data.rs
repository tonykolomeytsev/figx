use crate::{
    node_stream::{NodeStream, NodeStreamError}, Node, Result
};
use bytes::Bytes;
use log::debug;
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc, time::Duration};

#[derive(Clone)]
pub struct FigmaApi {
    client: Arc<ureq::Agent>,
}

impl Default for FigmaApi {
    fn default() -> Self {
        Self {
            client: Arc::new(
                ureq::Agent::config_builder()
                    .timeout_connect(Some(Duration::from_secs(15)))
                    .max_idle_connections(10)
                    .max_idle_connections_per_host(3)
                    .build()
                    .into(),
            ),
        }
    }
}

macro_rules! set_query_if_needed {
    (arr: $request:path, $q:literal => $x:expr) => {
        if let Some(arr) = $x {
            $request = $request.query($q, &arr.join(","));
        }
    };
    (num: $request:path, $q:literal => $x:expr) => {
        if let Some(num) = $x {
            $request = $request.query($q, &num.to_string());
        }
    };
    (bln: $request:path, $q:literal => $x:expr) => {
        if let Some(bln) = $x {
            $request = $request.query($q, if *bln { "true" } else { "false" });
        }
    };
    (txt: $request:path, $q:literal => $x:expr) => {
        if let Some(txt) = $x {
            $request = $request.query($q, &txt);
        }
    };
}

const fn mb(size_in_mb: u64) -> u64 {
    size_in_mb * 1024 * 1024
}

impl FigmaApi {
    const X_FIGMA_TOKEN: &str = "X-FIGMA-TOKEN";
    const BASE_URL: &str = "https://api.figma.com";

    pub fn get_file_nodes(
        &self,
        access_token: &str,
        file_key: &str,
        query: GetFileNodesQueryParameters,
    ) -> Result<impl Iterator<Item = std::result::Result<Node, NodeStreamError>>>
    {
        debug!(target: "Figma API", "get_file_nodes called for: {file_key}");
        let mut request = self
            .client
            .get(format!(
                "{base_url}/v1/files/{file_key}/nodes",
                base_url = Self::BASE_URL,
            ))
            .header(Self::X_FIGMA_TOKEN, access_token);
        // region: queries
        set_query_if_needed!(arr: request, "ids" => &query.ids);
        set_query_if_needed!(num: request, "depth" => &query.depth);
        set_query_if_needed!(txt: request, "geometry" => &query.geometry);
        set_query_if_needed!(txt: request, "version" => &query.version);
        // endregion: queries
        let reader = request.call()?.into_body().into_reader();
        debug!(target: "Figma API", "get_file_nodes done for: {file_key}");
        Ok(NodeStream::from(reader))
    }

    pub fn get_image(
        &self,
        access_token: &str,
        file_key: &str,
        query: GetImageQueryParameters,
    ) -> Result<GetImageResponse> {
        debug!(target: "Figma API", "get_image called for: {file_key}/{:?}", query.ids);
        let mut request = self
            .client
            .get(format!(
                "{base_url}/v1/images/{file_key}",
                base_url = Self::BASE_URL,
            ))
            .header(Self::X_FIGMA_TOKEN, access_token);
        // region: queries
        set_query_if_needed!(arr: request, "ids" => &query.ids);
        set_query_if_needed!(num: request, "scale" => &query.scale);
        set_query_if_needed!(txt: request, "format" => &query.format);
        set_query_if_needed!(bln: request, "svg_outline_text" => &query.svg_outline_text);
        set_query_if_needed!(bln: request, "svg_include_id" => &query.svg_include_id);
        set_query_if_needed!(bln: request, "svg_simplify_stroke" => &query.svg_simplify_stroke);
        set_query_if_needed!(bln: request, "contents_only" => &query.contents_only);
        set_query_if_needed!(bln: request, "use_absolute_bounds" => &query.use_absolute_bounds);
        set_query_if_needed!(txt: request, "version" => &query.version);
        // endregion: queries
        let response = request
            .call()?
            .body_mut()
            .with_config()
            .limit(mb(50))
            .read_json::<GetImageResponse>()?;
        debug!(target: "Figma API", "get_image done for: {file_key}/{:?}", query.ids);
        Ok(response)
    }

    pub fn download_resource(&self, access_token: &str, url: &str) -> Result<Bytes> {
        debug!(target: "Figma API", "download_resource called for: {url}");
        let request = self
            .client
            .get(url)
            .header(Self::X_FIGMA_TOKEN, access_token);
        let buf = request
            .call()?
            .body_mut()
            .with_config()
            .limit(mb(50))
            .read_to_vec()?;
        debug!(target: "Figma API", "download_resource done for: {url}");
        Ok(bytes::Bytes::from(buf))
    }
}

// region: GET file nodes

#[derive(Default)]
pub struct GetFileNodesQueryParameters<'a> {
    pub ids: Option<&'a [String]>,
    pub depth: Option<i32>,
    pub geometry: Option<&'a str>,
    pub version: Option<&'a str>,
}

// endregion: GET file nodes

// region: GET image

#[derive(Default)]
pub struct GetImageQueryParameters<'a> {
    pub ids: Option<&'a [String]>,
    pub scale: Option<f32>,
    pub format: Option<&'a str>,
    pub svg_outline_text: Option<bool>,
    pub svg_include_id: Option<bool>,
    pub svg_simplify_stroke: Option<bool>,
    pub contents_only: Option<bool>,
    pub use_absolute_bounds: Option<bool>,
    pub version: Option<&'a str>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetImageResponse {
    pub images: HashMap<String, Option<String>>,
}

// endregion: GET image
