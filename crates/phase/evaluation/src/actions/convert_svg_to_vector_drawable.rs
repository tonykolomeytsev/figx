use lib_label::Label;

use crate::{EvalContext, Result};

pub fn convert_svg_to_vector_drawable(
    _ctx: &EvalContext,
    _args: ConvertSvgToVectorDrawableArgs,
) -> Result<Vec<u8>> {
    todo!()
}

pub struct ConvertSvgToVectorDrawableArgs<'a> {
    pub label: &'a Label,
    pub variant_name: &'a str,
    pub svg: &'a [u8],
}
