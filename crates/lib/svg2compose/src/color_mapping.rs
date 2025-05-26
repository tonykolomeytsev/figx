use crate::Result;
use crate::image_vector::{Node, PooledColor};
use crate::{ColorMapping, image_vector::ImageVector};
use colorsys::Rgb;
use log::debug;

pub fn map_colors(image_vector: &mut ImageVector, color_mappings: &[ColorMapping]) -> Result<()> {
    for node in image_vector.nodes.iter_mut() {
        match node {
            Node::Group(_) => (),
            Node::Path(path) => {
                if let Some(color) = path.fill_color.as_mut() {
                    replace_color_if_needed(color, color_mappings)?;
                }
            }
        }
    }
    Ok(())
}

fn replace_color_if_needed(color: &mut PooledColor, color_mappings: &[ColorMapping]) -> Result<()> {
    for mapping in color_mappings {
        let rgb = match color {
            PooledColor::Source(rgb) => rgb,
            _ => continue,
        };
        if mapping.from == "*" || rgb == &Rgb::from_hex_str(&mapping.from)? {
            debug!(target: "Svg2Compose", "Found color mapping match: {} -> {}", mapping.from, mapping.to);
            *color = PooledColor::Mapped(mapping.to.to_owned())
        }
    }
    Ok(())
}
