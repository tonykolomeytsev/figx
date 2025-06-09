use crate::Result;
use crate::image_vector::{Node, PooledColor};
use crate::{ColorMapping, image_vector::ImageVector};
use colorsys::Rgb;
use log::debug;

pub fn map_colors(
    image_vector: &mut ImageVector,
    color_mappings: &[ColorMapping],
) -> Result<Vec<String>> {
    let mut used_imports = Vec::new();
    for node in image_vector.nodes.iter_mut() {
        replace_in_node(color_mappings, &mut used_imports, node)?;
    }
    Ok(used_imports)
}

fn replace_in_node(
    color_mappings: &[ColorMapping],
    used_imports: &mut Vec<String>,
    node: &mut Node,
) -> Result<()> {
    match node {
        Node::Group(group) => {
            for node in group.nodes.iter_mut() {
                replace_in_node(color_mappings, used_imports, node)?;
            }
        }
        Node::Path(path) => {
            if let Some(color) = path.fill_color.as_mut() {
                replace_color_if_needed(color, color_mappings, used_imports)?;
            }
            if let Some(color) = path.stroke.color.as_mut() {
                replace_color_if_needed(color, color_mappings, used_imports)?;
            }
        }
    }
    Ok(())
}

fn replace_color_if_needed(
    color: &mut PooledColor,
    color_mappings: &[ColorMapping],
    used_imports: &mut Vec<String>,
) -> Result<()> {
    for mapping in color_mappings {
        let rgb = match color {
            PooledColor::Source(rgb) => rgb,
            _ => continue,
        };
        if mapping.from == "*" || rgb == &Rgb::from_hex_str(&mapping.from)? {
            debug!(target: "Svg2Compose", "Found color mapping match: {} -> {}", mapping.from, mapping.to);
            *color = PooledColor::Mapped(mapping.to.to_owned());
            used_imports.append(&mut mapping.imports.to_owned());
            return Ok(()); // color was replaced, no more to do
        }
    }
    Ok(())
}
