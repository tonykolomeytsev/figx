use phase_loading::ResourceVariants;

pub struct ResourceVariant {
    pub default: bool,
    pub res_name: String,
    pub node_name: String,
    pub scale: f32,
}

pub fn generate_variants(
    res_name: &str,
    node_name: &str,
    scale: f32,
    variants: &Option<ResourceVariants>,
) -> Vec<ResourceVariant> {
    let base_variant = ResourceVariant {
        default: true,
        res_name: res_name.to_owned(),
        node_name: node_name.to_owned(),
        scale,
    };

    match variants {
        Some(ResourceVariants { all_variants: dict, use_variants: r#use }) => dict
            .iter()
            .filter(|(k, _)| match r#use {
                Some(variants) => variants.contains(*k),
                None => true,
            })
            .map(|(_, variant)| {
                let res_name = variant
                    .output_name
                    .replace("{base}", &base_variant.res_name);
                let node_name = variant
                    .figma_name
                    .replace("{base}", &base_variant.node_name);

                ResourceVariant {
                    default: false,
                    res_name,
                    node_name,
                    scale: variant.scale.unwrap_or(base_variant.scale),
                }
            })
            .collect::<Vec<_>>(),
        _ => vec![base_variant],
    }
}
