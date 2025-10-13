mod error;
use std::{
    fs::File,
    io::{BufWriter, Write},
    str::FromStr,
};

pub use error::*;
use lib_figma_fluent::{FigmaApi, GetFileNodesQueryParameters};
use lib_label::LabelPattern;
use log::{info, warn};
use phase_loading::load_workspace;

pub struct FeatureScanOptions {
    pub remotes: Vec<String>,
    pub checksum: bool,
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
        let mut stream = api.get_file_nodes_stream(
            &remote.access_token,
            &remote.file_key,
            GetFileNodesQueryParameters {
                ids: Some(&remote.container_node_ids),
                geometry: if opts.checksum { Some("paths") } else { None },
                ..Default::default()
            },
        )?;

        stream.try_for_each(|item| match item {
            Ok(node) => {
                if !node.visible || node.r#type != "COMPONENT" {
                    return Ok(());
                }

                writer.write(b"[[node]]\n")?;
                writer.write_fmt(format_args!("id = \"{}\"\n", node.id))?;
                writer.write_fmt(format_args!("name = \"{}\"\n", node.name))?;
                if opts.checksum {
                    writer.write_fmt(format_args!("checksum = {}\n", node.hash))?;
                }
                writer.write(b"\n")?;
                Ok(())
            }
            Err(e) => Err(Error::IndexingRemote(e.to_string())),
        })?;

        writer.flush()?;
        info!(target: "Scan", "scan saved to: {}", output_file.display());
    }
    Ok(())
}
