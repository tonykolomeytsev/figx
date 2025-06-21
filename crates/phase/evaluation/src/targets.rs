use phase_loading::{
    AndroidDensity, AndroidWebpProfile, Profile, Resource, ResourceAttrs, ResourceVariants,
};

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
        AndroidWebp(p) => return android_webp_targets(res, p),
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

fn android_webp_targets<'a>(res: &'a Resource, profile: &'a AndroidWebpProfile) -> Vec<Target<'a>> {
    let scales = &profile.scales;
    let themes: &[_] = if let Some(night_variant) = &profile.night {
        let light_variant = &res.attrs.node_name;
        let night_variant = night_variant.as_ref().replace("{base}", &light_variant);
        &[(light_variant.to_owned(), false), (night_variant, true)]
    } else {
        let light_variant = &res.attrs.node_name;
        &[(light_variant.to_owned(), false)]
    };
    let all_variants = cartesian_product(scales, themes);

    all_variants
        .into_iter()
        .map(|(density, (figma_name, night))| {
            let factor = scale_factor(density);
            let density_name = density_name(density);
            let variant_name = if !night {
                format!("{density_name}")
            } else {
                format!("night-{density_name}")
            };

            Target {
                id: Some(variant_name.clone()),
                attrs: &res.attrs,
                profile: &res.profile,
                figma_name: Some(figma_name.to_owned()),
                output_name: Some(res.attrs.label.name.to_string()),
                scale: Some(factor),
            }
        })
        .collect()
}

pub fn cartesian_product<'a, A, B>(list_a: &'a [A], list_b: &'a [B]) -> Vec<(&'a A, &'a B)> {
    list_a
        .iter()
        .flat_map(|a| list_b.iter().map(move |b| (a, b)))
        .collect()
}

pub fn scale_factor(d: &AndroidDensity) -> f32 {
    use AndroidDensity::*;
    match d {
        LDPI => 0.75,
        MDPI => 1.0,
        HDPI => 1.5,
        XHDPI => 2.0,
        XXHDPI => 3.0,
        XXXHDPI => 4.0,
    }
}

pub fn density_name(d: &AndroidDensity) -> &str {
    use AndroidDensity::*;
    match d {
        LDPI => "ldpi",
        MDPI => "mdpi",
        HDPI => "hdpi",
        XHDPI => "xhdpi",
        XXHDPI => "xxhdpi",
        XXXHDPI => "xxxhdpi",
    }
}
