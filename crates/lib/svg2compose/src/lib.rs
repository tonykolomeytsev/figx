use color_mapping::map_colors;
use image_vector::ImageVector;
use kotlin::FileSpec;
use vec2compose::BackingFieldComposableSpec;

mod color_mapping;
mod image_vector;
mod kotlin;
mod svg2vec;
mod vec2compose;

pub type Result<T> = std::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct SvgToComposeOptions {
    pub image_name: String,
    pub package: String,
    pub kotlin_explicit_api: bool,
    pub extension_target: Option<String>,
    pub file_suppress_lint: Vec<String>,
    pub color_mappings: Vec<ColorMapping>,
    pub preview: Option<ComposePreview>,
    pub composable_get: bool,
}

pub struct ColorMapping {
    pub from: String,
    pub to: String,
    pub imports: Vec<String>,
}

pub struct ComposePreview {
    pub imports: Vec<String>,
    pub code: String,
}

pub fn transform_svg_to_compose(svg: &[u8], options: SvgToComposeOptions) -> Result<Vec<u8>> {
    let tree = usvg::Tree::from_data(svg, &Default::default())?;
    let mut image_vector: ImageVector = tree.try_into()?;
    image_vector.name = options.image_name.to_owned();
    let mut used_imports = Vec::new();
    if !options.color_mappings.is_empty() {
        used_imports.append(&mut map_colors(&mut image_vector, &options.color_mappings)?);
    }
    let output = backing_field_template(image_vector, options, used_imports);
    Ok(output.into_bytes())
}

fn backing_field_template(image_vector: ImageVector, options: SvgToComposeOptions, imports: Vec<String>) -> String {
    let cb: FileSpec = BackingFieldComposableSpec {
        options,
        image_vector,
        imports,
    }
    .into();
    cb.to_string()
}
