use crate::kotlin::CodeBlock;
use codegen::iv_builder::*;
use kotlin::FileSpec;
use lib_image_vector::ImageVector;
use vec2compose::BackingFieldComposableSpec;

mod kotlin;
mod vec2compose;
mod codegen {
    pub(crate) mod iv_builder;
}

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
    let iv_code_block = codegen_iv_builder(image_vector, &options.color_mappings)?;
    let output = backing_field_template(iv_code_block, options);
    Ok(output.into_bytes())
}

fn backing_field_template(iv_code_block: CodeBlock, options: SvgToComposeOptions) -> String {
    let cb: FileSpec = BackingFieldComposableSpec {
        options,
        iv_code_block,
    }
    .into();
    cb.to_string().trim_end().to_string()
}
