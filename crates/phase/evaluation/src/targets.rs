use phase_loading::{Profile, Resource, ResourceAttrs, ResourceVariants};

pub struct Target<'a> {
    pub id: Option<String>,
    pub attrs: &'a ResourceAttrs,
    pub profile: &'a Profile,
    pub figma_name: Option<String>,
    pub output_name: Option<String>,
    pub scale: Option<f32>,
}

impl<'a> Target<'a> {
    pub fn figma_name(&self) -> &str {
        self.figma_name
            .as_ref()
            .unwrap_or_else(|| &self.attrs.node_name)
    }

    pub fn output_name(&self) -> &str {
        self.output_name
            .as_deref()
            .unwrap_or_else(|| self.attrs.label.name.as_ref())
    }
}

pub fn targets_from_resource(res: &Resource) -> Vec<Target> {
    use phase_loading::Profile::*;
    let variants = match res.profile.as_ref() {
        Png(p) => p.variants.as_ref(),
        Svg(p) => p.variants.as_ref(),
        Pdf(p) => p.variants.as_ref(),
        Webp(p) => p.variants.as_ref(),
        Compose(p) => p.variants.as_ref(),
        AndroidWebp(_) => None,
    };

    match variants {
        None => vec![Target {
            id: None,
            attrs: &res.attrs,
            profile: &res.profile,
            figma_name: None,
            output_name: None,
            scale: None,
        }],
        Some(ResourceVariants {
            all_variants,
            use_variants,
        }) => all_variants
            .iter()
            .filter(|(k, _)| match use_variants {
                None => true,
                Some(only) => only.contains(*k),
            })
            .map(|(k, v)| {
                let output_name = v
                    .output_name
                    .as_ref()
                    .replace("{base}", &res.attrs.label.name.as_ref());
                let figma_name = v
                    .figma_name
                    .as_ref()
                    .replace("{base}", &res.attrs.node_name);
                let scale = v.scale.as_deref().cloned();

                Target {
                    id: Some(k.to_owned()),
                    attrs: &res.attrs,
                    profile: &res.profile,
                    figma_name: Some(figma_name),
                    output_name: Some(output_name),
                    scale: if res.profile.vector() {
                        Some(1.0)
                    } else {
                        scale
                    },
                }
            })
            .collect(),
    }
}
