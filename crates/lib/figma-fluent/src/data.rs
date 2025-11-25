use crate::{RateLimitError, Result};
use bytes::Bytes;
use log::debug;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc, time::Duration};
use ureq::{
    Body,
    http::{Response, StatusCode},
};

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
                    .http_status_as_error(false)
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

    /// Gets selected Figma nodes and returns their structure.
    ///
    /// The `ScannedNodeDto` returned by this method has a minimal set of fields
    /// and should be used only for file structure scanning purposes.
    pub fn get_file_nodes(
        &self,
        access_token: &str,
        file_key: &str,
        query: GetFileNodesQueryParameters,
    ) -> Result<GetFileNodesResponse> {
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

        let mut response = request.call()?;
        handle_http_errors(&response)?;
        let response = response
            .body_mut()
            .with_config()
            .limit(mb(2048))
            .read_json::<GetFileNodesResponse>()?;
        debug!(target: "Figma API", "get_file_nodes_scan done for: {file_key}");
        Ok(response)
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

        let mut response = request.call()?;
        handle_http_errors(&response)?;
        let response = response
            .body_mut()
            .with_config()
            .limit(mb(100))
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
        let mut response = request.call()?;
        handle_http_errors(&response)?;
        let buf = response
            .body_mut()
            .with_config()
            .limit(mb(50))
            .read_to_vec()?;
        debug!(target: "Figma API", "download_resource done for: {url}");
        Ok(bytes::Bytes::from(buf))
    }
}

// region: Http error handling

fn handle_http_errors(response: &Response<Body>) -> Result<()> {
    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        let retry_after_sec = response
            .headers()
            .get("Retry-After")
            .and_then(|val| val.to_str().ok())
            .and_then(|val| val.parse::<u32>().ok())
            .unwrap_or(5);
        let figma_plan_tier = response
            .headers()
            .get("X-Figma-Plan-Tier")
            .and_then(|val| val.to_str().ok())
            .unwrap_or("")
            .to_string();
        let figma_limit_type = response
            .headers()
            .get("X-Figma-Rate-Limit-Type")
            .and_then(|val| val.to_str().ok())
            .unwrap_or("")
            .to_string();

        return Err(RateLimitError {
            retry_after_sec,
            figma_plan_tier,
            figma_limit_type,
        }
        .into());
    }
    if !response.status().is_success() {
        return Err(ureq::Error::StatusCode(response.status().as_u16()).into());
    }
    Ok(())
}

// endregion: Http error handling

// region: GET file nodes scan

#[derive(Default)]
pub struct GetFileNodesQueryParameters<'a> {
    pub ids: Option<&'a [String]>,
    pub depth: Option<i32>,
    pub geometry: Option<&'a str>,
    pub version: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
pub struct GetFileNodesResponse {
    pub nodes: HashMap<String, DocumentNodeDto>,
}

#[derive(Debug, Deserialize)]
pub struct DocumentNodeDto {
    pub document: ScannedNodeDto,
}

#[derive(Debug)]
pub struct ScannedNodeDto {
    pub r#type: String,
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub children: Vec<ScannedNodeDto>,
    pub fills: Vec<PaintDto>,
    pub hash: u64,
}

#[derive(Debug, Deserialize)]
pub struct PaintDto {
    pub r#type: String,
}

impl<'de> Deserialize<'de> for ScannedNodeDto {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut json_value = Value::deserialize(deserializer)?;

        let hash = {
            let mut hasher = xxhash_rust::xxh64::Xxh64::new(42);
            hasher.update(json_value.to_string().as_bytes());
            hasher.digest()
        };

        let obj = json_value
            .as_object_mut()
            .ok_or_else(|| serde::de::Error::custom("Expected JSON object"))?;

        let r#type = obj
            .get("type")
            .and_then(|value| value.as_str())
            .map(String::from)
            .ok_or_else(|| serde::de::Error::missing_field("type"))?;

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

        let fills = obj
            .remove("fills")
            .map(|v| serde_json::from_value(v).map_err(serde::de::Error::custom))
            .unwrap_or(Ok(Vec::new()))?;

        Ok(ScannedNodeDto {
            r#type,
            id,
            name,
            visible,
            children,
            fills,
            hash,
        })
    }
}

// endregion: GET file nodes scan

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
