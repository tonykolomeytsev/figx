use bincode::{Decode, Encode};
use std::collections::HashMap;

#[derive(Debug, Encode, Decode)]
pub struct RemoteMetadata {
    pub name_to_node: HashMap<String, NodeMetadata>,
}

#[derive(Debug, Encode, Decode)]
pub struct NodeMetadata {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub hash: u64,
}
