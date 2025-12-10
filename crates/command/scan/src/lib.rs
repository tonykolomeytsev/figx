mod error;
use std::{
    collections::VecDeque,
    fs::File,
    io::{BufWriter, Write},
    str::FromStr,
};

pub use error::*;
use lib_figma_fluent::{FigmaApi, GetFileNodesScanQueryParameters, ScannedNodeDto};
use lib_label::LabelPattern;
use log::{info, warn};
use phase_loading::{NodeIdList, load_workspace};

pub struct FeatureScanOptions {
    pub remotes: Vec<String>,
}

pub fn scan(opts: FeatureScanOptions) -> Result<()> {
    warn!(target: "Experimental", "remote scanning is an experimental feature, api may change in the future");

    let empty_pattern = LabelPattern::from_str("").expect("always empty pattern");
    let ws = load_workspace(empty_pattern, false)?;
    let scans_dir = ws.context.out_dir.join("scans");
    std::fs::create_dir_all(&scans_dir)?;

    for name in opts.remotes {
        let Some(remote) = ws.remotes.iter().find(|it| it.id == name) else {
            return Err(Error::UserError(format!(
                "No remote with name '{name}' defined in workspace"
            )));
        };
        info!(target: "Scan", "scanning remote with name `{name}`");

        let output_file = scans_dir.join(format!("{name}.toml"));
        let mut writer = BufWriter::new(File::create(&output_file)?);
        writer.write(b"version = 1\n\n")?;

        let api = FigmaApi::default();
        let response = api.get_file_nodes_scan(
            &remote.access_token,
            &remote.file_key,
            GetFileNodesScanQueryParameters {
                ids: Some(&remote.container_node_ids.to_string_id_list()),
                ..Default::default()
            },
        )?;

        for (container_node_id, dto) in response.nodes {
            let container_node_tag = if let NodeIdList::IdToTag(table) = &remote.container_node_ids
            {
                if let Some(tag) = table.get(&container_node_id.replace(":", "-")) {
                    Some(tag.to_owned())
                } else {
                    return Err(Error::IndexingRemote(format!(
                        "tag for every node from figma must be present: node-id={container_node_id}"
                    )));
                }
            } else {
                None
            };

            let scanned_nodes = extract_metadata(&dto.document.children);
            let metadata_dict = &dto.components;

            for node in scanned_nodes {
                writer.write(b"[[node]]\n")?;
                writer.write_fmt(format_args!("id = \"{}\"\n", node.id))?;
                writer.write_fmt(format_args!("name = \"{}\"\n", node.name))?;
                if let Some(tag) = &container_node_tag {
                    writer.write_fmt(format_args!("tag = \"{tag}\"\n"))?;
                }
                if let Some(metadata) = &metadata_dict.get(&node.id) {
                    let description = &metadata.description;
                    if !metadata.description.is_empty() {
                        writer.write_fmt(format_args!("description = '''{description}'''\n"))?;
                    }
                }
                writer.write(b"\n")?;
            }
        }

        writer.flush()?;
        info!(target: "Scan", "scan saved to: {}", output_file.display());
    }
    Ok(())
}

/// Mapper from response to metadata
fn extract_metadata(values: &[ScannedNodeDto]) -> Vec<ScannedNode> {
    let mut queue = VecDeque::new();
    let mut output_nodes = Vec::with_capacity(4096);
    for value in values {
        if value.visible {
            queue.push_back(value);
        }
    }
    while let Some(current) = queue.pop_front() {
        if !current.name.is_empty() && current.r#type == "COMPONENT" {
            output_nodes.push(ScannedNode {
                id: current.id.clone(),
                name: current.name.clone(),
            });
        }
        for child in &current.children {
            if child.visible {
                queue.push_back(child);
            }
        }
    }
    output_nodes
}

struct ScannedNode {
    pub id: String,
    pub name: String,
}
