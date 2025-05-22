use log::debug;

use crate::{figma::{NodeMetadata, RemoteMetadata}, Error, Result};

pub fn find_node_by_name(args: FindNodeByNameArgs) -> Result<&NodeMetadata> {
    debug!("seeking node: '{}'", args.name);
    let node = args
        .remote
        .name_to_node
        .get(args.name)
        .ok_or_else(|| Error::FindNode {
            node_name: args.name.to_string(),
        })?;
    Ok(node)
}

pub struct FindNodeByNameArgs<'a> {
    pub name: &'a str,
    pub remote: &'a RemoteMetadata,
}
