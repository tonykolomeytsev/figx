use bincode::{Decode, Encode};
use std::collections::HashMap;

#[derive(Debug, Encode, Decode)]
pub struct RemoteMetadata {
    pub name_to_node: HashMap<String, NodeMetadata>,
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct NodeMetadata {
    pub id: String,
    pub name: String,
    pub hash: u64,
    pub uses_raster_paints: bool,
}
