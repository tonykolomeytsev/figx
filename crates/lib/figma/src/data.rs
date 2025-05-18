use crate::Result;
use bytes::Bytes;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::{collections::HashMap, hash::Hasher, sync::Arc, time::Duration};

#[derive(Clone)]
pub struct FigmaApi {
    client: Arc<ureq::Agent>,
}

impl Default for FigmaApi {
    fn default() -> Self {
        Self {
            client: Arc::new(
                ureq::Agent::config_builder()
                    .timeout_connect(Some(Duration::from_secs(5)))
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

    pub fn get_file(
        &self,
        access_token: &str,
        file_key: &str,
        query: GetFileQueryParameters,
    ) -> Result<GetFileResponse> {
        let mut request = self
            .client
            .get(format!(
                "{base_url}/v1/files/{file_key}",
                base_url = Self::BASE_URL,
            ))
            .header(Self::X_FIGMA_TOKEN, access_token);
        // region: queries
        set_query_if_needed!(arr: request, "ids" => &query.ids);
        set_query_if_needed!(num: request, "depth" => &query.depth);
        set_query_if_needed!(txt: request, "geometry" => &query.geometry);
        set_query_if_needed!(txt: request, "version" => &query.version);
        // endregion: queries
        Ok(request
            .call()?
            .body_mut()
            .with_config()
            .limit(mb(100))
            .read_json::<GetFileResponse>()?)
    }

    pub fn get_file_nodes(
        &self,
        access_token: &str,
        file_key: &str,
        query: GetFileNodesQueryParameters,
    ) -> Result<GetFileNodesResponse> {
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
        Ok(request
            .call()?
            .body_mut()
            .with_config()
            .limit(mb(100))
            .read_json::<GetFileNodesResponse>()?)
    }

    pub fn get_image(
        &self,
        access_token: &str,
        file_key: &str,
        query: GetImageQueryParameters,
    ) -> Result<GetImageResponse> {
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
        Ok(request
            .call()?
            .body_mut()
            .with_config()
            .limit(mb(50))
            .read_json::<GetImageResponse>()?)
    }

    pub fn download_resource(&self, access_token: &str, url: &str) -> Result<Bytes> {
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
        Ok(bytes::Bytes::from(buf))
    }
}

#[derive(Debug)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub children: Vec<Node>,
    pub hash: u64,
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut json_value = Value::deserialize(deserializer)?;
        let hash = {
            let mut hasher = xxhash_rust::xxh64::Xxh64::new(42);
            hasher.write(json_value.to_string().as_bytes());
            hasher.digest()
        };

        let obj = json_value
            .as_object_mut()
            .ok_or_else(|| serde::de::Error::custom("Expected JSON object"))?;

        let id = obj
            .get("id")
            .and_then(|value| value.as_str())
            .map(String::from)
            .ok_or_else(|| serde::de::Error::missing_field("id"))?;

        let name = obj
            .get("name")
            .and_then(|value| value.as_str())
            .map(String::from)
            .ok_or_else(|| serde::de::Error::missing_field("name"))?;

        let visible = obj
            .remove("visible")
            .and_then(|value| Value::as_bool(&value))
            .unwrap_or(true);

        let children = obj
            .remove("children")
            .map(|v| serde_json::from_value(v).map_err(serde::de::Error::custom))
            .unwrap_or(Ok(Vec::new()))?;

        Ok(Node {
            id,
            name,
            visible,
            children,
            hash,
        })
    }
}

// region: GET file

#[derive(Default)]
pub struct GetFileQueryParameters {
    pub ids: Option<Vec<String>>,
    pub depth: Option<i32>,
    pub geometry: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetFileResponse {
    pub document: Node,
}

// endregion: GET file

// region: GET file nodes

#[derive(Default)]
pub struct GetFileNodesQueryParameters {
    pub ids: Option<Vec<String>>,
    pub depth: Option<i32>,
    pub geometry: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetFileNodesResponse {
    pub nodes: HashMap<String, IdentifiedNodeDto>,
}

#[derive(Debug, Deserialize)]
pub struct IdentifiedNodeDto {
    pub document: Node,
}

// endregion: GET file nodes

// region: GET image

#[derive(Default)]
pub struct GetImageQueryParameters {
    pub ids: Option<Vec<String>>,
    pub scale: Option<f32>,
    pub format: Option<String>,
    pub svg_outline_text: Option<bool>,
    pub svg_include_id: Option<bool>,
    pub svg_simplify_stroke: Option<bool>,
    pub contents_only: Option<bool>,
    pub use_absolute_bounds: Option<bool>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetImageResponse {
    pub err: Option<String>,
    pub images: HashMap<String, Option<String>>,
}

// endregion: GET image
