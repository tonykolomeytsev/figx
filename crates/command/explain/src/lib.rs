use lib_label::LabelPattern;
use owo_colors::OwoColorize;
use phase_evaluation::actions::{
    import_android_webp::{cartesian_product, density_name, expand_night_variant, scale_factor},
    import_compose::{get_kotlin_package, get_output_dir_for_compose_profile},
    util_variants::generate_variants,
};
use phase_loading::{
    AndroidWebpProfile, ComposeProfile, PdfProfile, PngProfile, Profile, ResourceAttrs, SvgProfile,
    WebpProfile,
};
mod error;
pub use error::*;

pub struct FeatureExplainOptions {
    pub pattern: Vec<String>,
}

#[derive(Default)]
struct Node {
    name: String,
    children: Vec<Node>,
    params: Vec<(&'static str, String)>,
}

macro_rules! node {
    ($name:expr, [ $($par:expr),* ]) => {
        Node { name: $name.to_string(), params: vec![ $( $par ),* ], ..Default::default() }
    };
    ($name:expr, [ $($par:expr),* ], $( $ch:expr ),+) => {
        Node { name: $name.to_string(), params: vec![ $( $par ),* ], children: vec![ $( $ch ),+ ] }
    };
    ($name:expr, $( $ch:expr ),+) => {
        Node { name: $name.to_string(), children: vec![ $( $ch ),+ ], ..Default::default() }
    };
}

pub fn explain(opts: FeatureExplainOptions) -> Result<()> {
    let pattern = LabelPattern::try_from(opts.pattern)?;
    let ws = phase_loading::load_workspace(pattern)?;

    let mut nodes = Vec::with_capacity(1024);
    for res in ws.packages.iter().flat_map(|pkg| &pkg.resources) {
        let node = match res.profile.as_ref() {
            Profile::Png(p) => png_resource_tree(&res.attrs, p),
            Profile::Svg(p) => svg_resource_tree(&res.attrs, p),
            Profile::Pdf(p) => pdf_resource_tree(&res.attrs, p),
            Profile::Webp(p) => webp_resource_tree(&res.attrs, p),
            Profile::Compose(p) => compose_resource_tree(&res.attrs, p),
            Profile::AndroidWebp(p) => android_webp_resource_tree(&res.attrs, p),
        };
        nodes.push(node);
    }

    for node in nodes {
        println!("{node}");
    }

    Ok(())
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_tree(f, "")
    }
}

impl Node {
    fn fmt_tree(&self, f: &mut std::fmt::Formatter<'_>, prefix: &str) -> std::fmt::Result {
        // Выводим текущий узел
        writeln!(f, "{}", self.name.bold())?;
        for (param_key, param_value) in &self.params {
            let param_key = format!("{param_key}: ");
            writeln!(
                f,
                "{prefix}   {} {}{}",
                "┆".bright_black(),
                param_key.green(),
                param_value
            )?;
        }

        // Обрабатываем всех детей кроме последнего
        let middle_children = self.children.len().saturating_sub(1);
        for child in self.children.iter().take(middle_children) {
            // Префикс для текущего узла
            write!(f, "{prefix}{corner} ", corner = "├──".bright_black())?;
            // Префикс для детей текущего узла
            let new_prefix = format!("{prefix}{border}   ", border = "│".bright_black());
            child.fmt_tree(f, &new_prefix)?;
        }

        // Обрабатываем последнего ребенка (если есть)
        if let Some(last_child) = self.children.last() {
            // Префикс для последнего узла
            write!(f, "{prefix}{corner} ", corner = "╰──".bright_black())?;
            // Префикс для детей последнего узла (пробелы вместо │)
            let new_prefix = format!("{prefix}    ");
            last_child.fmt_tree(f, &new_prefix)?;
        }

        Ok(())
    }
}

fn png_resource_tree(r: &ResourceAttrs, p: &PngProfile) -> Node {
    let mut root_node = Node {
        name: r.label.to_string(),
        children: Vec::new(),
        params: Vec::new(),
    };
    let variants = generate_variants(
        &r.label.name.to_string(),
        &r.node_name,
        *p.scale,
        &p.variants,
    );
    for v in variants {
        let mut child_nodes = vec![
            node!(
                format!("📤 Export PNG from remote {}", r.remote),
                [
                    ("node", v.node_name.to_string()),
                    ("scale", v.scale.to_string())
                ]
            ),
            node!(
                "💾 Write to file",
                [("output", format!("{}.png", v.res_name))]
            ),
        ];

        if v.default {
            root_node.children.append(&mut child_nodes);
        } else {
            let variant_node = Node {
                name: format!("Variant '{}'", v.res_name),
                children: child_nodes,
                params: Vec::new(),
            };
            root_node.children.push(variant_node);
        }
    }
    root_node
}

fn svg_resource_tree(r: &ResourceAttrs, p: &SvgProfile) -> Node {
    let mut root_node = Node {
        name: r.label.to_string(),
        children: Vec::new(),
        params: Vec::new(),
    };
    let variants = generate_variants(&r.label.name.to_string(), &r.node_name, 1.0, &p.variants);
    for v in variants {
        let mut child_nodes = vec![
            node!(
                format!("📤 Export SVG from remote {}", r.remote),
                [("node", v.node_name.to_string())]
            ),
            node!(
                "💾 Write to file",
                [("output", format!("{}.svg", v.res_name))]
            ),
        ];

        if v.default {
            root_node.children.append(&mut child_nodes);
        } else {
            let variant_node = Node {
                name: format!("Variant '{}'", v.res_name),
                children: child_nodes,
                params: Vec::new(),
            };
            root_node.children.push(variant_node);
        }
    }
    root_node
}

fn pdf_resource_tree(r: &ResourceAttrs, p: &PdfProfile) -> Node {
    let mut root_node = Node {
        name: r.label.to_string(),
        children: Vec::new(),
        params: Vec::new(),
    };
    let variants = generate_variants(&r.label.name.to_string(), &r.node_name, 1.0, &p.variants);
    for v in variants {
        let mut child_nodes = vec![
            node!(
                format!("📤 Export PDF from remote {}", r.remote),
                [("node", v.node_name.to_string())]
            ),
            node!(
                "💾 Write to file",
                [("output", format!("{}.pdf", v.res_name))]
            ),
        ];

        if v.default {
            root_node.children.append(&mut child_nodes);
        } else {
            let variant_node = Node {
                name: format!("Variant '{}'", v.res_name),
                children: child_nodes,
                params: Vec::new(),
            };
            root_node.children.push(variant_node);
        }
    }
    root_node
}

fn webp_resource_tree(r: &ResourceAttrs, p: &WebpProfile) -> Node {
    let mut root_node = Node {
        name: r.label.to_string(),
        children: Vec::new(),
        params: Vec::new(),
    };
    let variants = generate_variants(
        &r.label.name.to_string(),
        &r.node_name,
        *p.scale,
        &p.variants,
    );
    for v in variants {
        let mut child_nodes = vec![
            node!(
                format!("📤 Export PNG from remote {}", r.remote),
                [("node", v.node_name.to_string())]
            ),
            node!(
                "✨ Transform PNG to WEBP",
                [("quality", p.quality.to_string())]
            ),
            node!(
                "💾 Write to file",
                [("output", format!("{}.webp", v.res_name))]
            ),
        ];

        if v.default {
            root_node.children.append(&mut child_nodes);
        } else {
            let variant_node = Node {
                name: format!("Variant '{}'", v.res_name),
                children: child_nodes,
                params: Vec::new(),
            };
            root_node.children.push(variant_node);
        }
    }
    root_node
}

fn compose_resource_tree(r: &ResourceAttrs, p: &ComposeProfile) -> Node {
    let output_dir = get_output_dir_for_compose_profile(p, &r.package_dir);
    let package = match &p.package {
        Some(pkg) if pkg.is_empty() => "Explicitly empty".to_owned(),
        Some(pkg) => pkg.to_owned(),
        None => match get_kotlin_package(&output_dir) {
            Some(pkg) => pkg,
            None => "Undetermined".to_owned(),
        },
    };

    let mut root_node = Node {
        name: r.label.to_string(),
        children: Vec::new(),
        params: Vec::new(),
    };
    let variants = generate_variants(&r.label.name.to_string(), &r.node_name, 1.0, &p.variants);
    for v in variants {
        let mut child_nodes = vec![
            node!(
                format!("📤 Export SVG from remote {}", r.remote),
                [("node", v.node_name.to_string())]
            ),
            node!(
                "✨ Transform SVG to Compose",
                [("package", package.to_string())]
            ),
            node!(
                "💾 Write to file",
                [("output", format!("{}.kt", v.res_name))]
            ),
        ];

        if v.default {
            root_node.children.append(&mut child_nodes);
        } else {
            let variant_node = Node {
                name: format!("Variant '{}'", v.res_name),
                children: child_nodes,
                params: Vec::new(),
            };
            root_node.children.push(variant_node);
        }
    }
    root_node
}

fn android_webp_resource_tree(r: &ResourceAttrs, p: &AndroidWebpProfile) -> Node {
    // region: generating all android variants
    let scales = &p.scales;
    let themes: &[_] = if let Some(night_variant) = &p.night {
        let light_variant = &r.node_name;
        let night_variant = expand_night_variant(light_variant, night_variant.as_ref());
        &[(light_variant.to_owned(), false), (night_variant, true)]
    } else {
        let light_variant = &r.node_name;
        &[(light_variant.to_owned(), false)]
    };
    let all_variants = cartesian_product(scales, themes);
    // endregion: generating all android variants

    let res_name = r.label.name.to_string();
    Node {
        name: r.label.to_string(),
        children: all_variants
            .iter()
            .map(|(d, (node_name, is_night))| {
                let density_name = density_name(d);
                let scale_factor = scale_factor(d);
                let variant_name = if !*is_night {
                    format!("{density_name}")
                } else {
                    format!("night-{density_name}")
                };
                node!(
                    format!("Variant '{variant_name}'"),
                    node!(
                        format!("📤 Export PNG from remote {}", r.remote),
                        [
                            ("node", node_name.to_string()),
                            ("scale", scale_factor.to_string())
                        ]
                    ),
                    node!(
                        "✨ Transform PNG to WEBP",
                        [("quality", p.quality.to_string())]
                    ),
                    node!(
                        "💾 Write to file",
                        [("output", format!("drawable-{variant_name}/{res_name}.webp"))]
                    )
                )
            })
            .collect(),
        ..Default::default()
    }
}
