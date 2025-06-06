use lib_label::LabelPattern;
use owo_colors::OwoColorize;
use phase_evaluation::actions::{
    import_android_webp::{cartesian_product, density_name, scale_factor},
    import_compose::{get_kotlin_package, get_output_dir_for_compose_profile},
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
                "{prefix}{} {}{}",
                "┆".bright_black(),
                param_key.blue(),
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
    let res_name = r.label.name.to_string();
    node!(
        r.label.to_string(),
        node!(
            "Write to file",
            [("file", format!("{res_name}.png"))],
            node!(
                "Download PNG",
                node!(
                    "Export PNG",
                    [
                        ("node", r.node_name.to_string()),
                        ("scale", p.scale.to_string())
                    ],
                    node!(format!("Fetch remote {}", r.remote), [])
                )
            )
        )
    )
}

fn svg_resource_tree(r: &ResourceAttrs, p: &SvgProfile) -> Node {
    let res_name = r.label.name.to_string();
    node!(
        r.label.to_string(),
        node!(
            "Write to file",
            [("file", format!("{res_name}.svg"))],
            node!(
                "Download SVG",
                node!(
                    "Export SVG",
                    [
                        ("node", r.node_name.to_string()),
                        ("scale", p.scale.to_string())
                    ],
                    node!(format!("Fetch remote {}", r.remote), [])
                )
            )
        )
    )
}

fn pdf_resource_tree(r: &ResourceAttrs, p: &PdfProfile) -> Node {
    let res_name = r.label.name.to_string();
    node!(
        r.label.to_string(),
        node!(
            "Write to file",
            [("file", format!("{res_name}.pdf"))],
            node!(
                "Download PDF",
                node!(
                    "Export PDF",
                    [
                        ("node", r.node_name.to_string()),
                        ("scale", p.scale.to_string())
                    ],
                    node!(format!("Fetch remote {}", r.remote), [])
                )
            )
        )
    )
}

fn webp_resource_tree(r: &ResourceAttrs, p: &WebpProfile) -> Node {
    let res_name = r.label.name.to_string();
    node!(
        r.label.to_string(),
        node!(
            "Write to file",
            [("file", format!("{res_name}.webp"))],
            node!(
                "Transform PNG to WEBP",
                [("quality", p.quality.to_string())],
                node!(
                    "Download PNG",
                    node!(
                        "Export PNG",
                        [
                            ("node", r.node_name.to_string()),
                            ("scale", p.scale.to_string())
                        ],
                        node!(format!("Fetch remote {}", r.remote), [])
                    )
                )
            )
        )
    )
}

fn compose_resource_tree(r: &ResourceAttrs, p: &ComposeProfile) -> Node {
    let res_name = r.label.name.to_string();
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

    fn child_tree(
        r: &ResourceAttrs,
        p: &ComposeProfile,
        res_name: &str,
        node_name: &str,
        package: &str,
    ) -> Node {
        node!(
            "Write to file",
            [("file", format!("{res_name}.kt"))],
            node!(
                "Transform SVG to Compose",
                [("package", package.to_string())],
                node!(
                    "Download PNG",
                    node!(
                        "Export PNG",
                        [
                            ("node", node_name.to_string()),
                            ("scale", p.scale.to_string())
                        ],
                        node!(format!("Fetch remote {}", r.remote), [])
                    )
                )
            )
        )
    }

    if let Some(variants) = &p.variants {
        for variant in variants {
            let naming = &p.variant_naming;
            let res_name = naming
                .local_name
                .replace("{base}", &res_name)
                .replace("{variant}", &variant);
            let node_name = naming
                .figma_name
                .replace("{base}", &r.node_name)
                .replace("{variant}", &variant);
            root_node
                .children
                .push(child_tree(r, p, &res_name, &node_name, &package));
        }
    } else {
        root_node
            .children
            .push(child_tree(r, p, &res_name, &r.node_name, &package));
    }
    root_node
}

fn android_webp_resource_tree(r: &ResourceAttrs, p: &AndroidWebpProfile) -> Node {
    // region: generating all android variants
    let scales = &p.scales;
    let themes: &[_] = if let Some(_) = &p.night {
        &[(false), (true)]
    } else {
        &[(false)]
    };
    let all_variants = cartesian_product(scales, themes);
    // endregion: generating all android variants

    let res_name = r.label.name.to_string();
    Node {
        name: r.label.to_string(),
        children: all_variants
            .iter()
            .map(|(d, is_night)| {
                let density_name = density_name(d);
                let scale_factor = scale_factor(d);
                let variant_name = if !*is_night {
                    format!("{density_name}")
                } else {
                    format!("night-{density_name}")
                };
                node!(
                    "Write to file",
                    [("output", format!("drawable-{variant_name}/{res_name}.webp"))],
                    node!(
                        "Transform PNG to WEBP",
                        [("quality", p.quality.to_string())],
                        node!(
                            "Download PNG",
                            node!(
                                "Export PNG",
                                [
                                    ("node", r.node_name.clone()),
                                    ("scale", scale_factor.to_string())
                                ],
                                node!(format!("Fetch remote {}", r.remote), [])
                            )
                        )
                    )
                )
            })
            .collect(),
        ..Default::default()
    }
}
