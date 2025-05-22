use crate::EvalContext;
use crate::Result;
use lib_svg2compose::SvgToComposeOptions;
use log::debug;

pub fn convert_svg_to_compose(
    _ctx: &EvalContext,
    args: ConvertSvgToComposeArgs,
) -> Result<Vec<u8>> {
    debug!("transforming: svg to compose (name={})", args.name);
    let compose = lib_svg2compose::transform_svg_to_compose(
        args.svg,
        SvgToComposeOptions {
            image_name: args.name.to_owned(),
            package: args.package.to_owned(),
            kotlin_explicit_api: args.kotlin_explicit_api,
        },
    )?;
    Ok(compose)
}

pub struct ConvertSvgToComposeArgs<'a> {
    pub name: &'a str,
    pub package: &'a str,
    pub kotlin_explicit_api: bool,
    pub svg: &'a [u8],
}
