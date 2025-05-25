use image_vector::ImageVector;
use kotlin::FileSpec;
use vec2compose::BackingFieldComposableSpec;

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
    pub extension_target_fq_name: Option<String>,
}

pub fn transform_svg_to_compose(svg: &[u8], options: SvgToComposeOptions) -> Result<Vec<u8>> {
    let tree = usvg::Tree::from_data(svg, &Default::default())?;
    let mut image_vector: ImageVector = tree.try_into()?;
    image_vector.name = options.image_name.to_owned();
    let output = backing_field_template(image_vector, options);
    Ok(output.into_bytes())
}

fn backing_field_template(image_vector: ImageVector, options: SvgToComposeOptions) -> String {
    let cb: FileSpec = BackingFieldComposableSpec {
        options,
        image_vector,
    }
    .into();
    cb.to_string()
}
