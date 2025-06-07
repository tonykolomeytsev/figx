use phase_loading::ResourceVariants;

pub struct ResourceVariant {
    pub default: bool,
    pub res_name: String,
    pub node_name: String,
}

pub fn generate_variants(
    res_name: &str,
    node_name: &str,
    variants: &Option<ResourceVariants>,
) -> Vec<ResourceVariant> {
    let base_variant = ResourceVariant {
        default: true,
        res_name: res_name.to_owned(),
        node_name: node_name.to_owned(),
    };

    match variants {
        Some(ResourceVariants {
            naming,
            list: Some(list),
        }) => list
            .iter()
            .map(|variant| {
                let naming = &naming;
                let res_name = naming
                    .local_name
                    .replace("{base}", &base_variant.res_name)
                    .replace("{variant}", &variant);
                let node_name = naming
                    .figma_name
                    .replace("{base}", &base_variant.node_name)
                    .replace("{variant}", &variant);

                ResourceVariant {
                    default: false,
                    res_name,
                    node_name,
                }
            })
            .collect::<Vec<_>>(),
        _ => vec![base_variant],
    }
}
