use crossterm::style::Stylize;
use lib_label::LabelPattern;
use phase_evaluation::{
    actions::{get_kotlin_package, get_output_dir_for_compose_profile},
    targets_from_resource,
};
use phase_loading::{
    AndroidWebpProfile, ComposeProfile, PdfProfile, PngProfile, Profile, Resource, SvgProfile,
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
    let ws = phase_loading::load_workspace(pattern, true)?;

    let mut nodes = Vec::with_capacity(1024);
    for res in ws.packages.iter().flat_map(|pkg| &pkg.resources) {
        let node = match res.profile.as_ref() {
            Profile::Png(p) => png_resource_tree(res, p),
            Profile::Svg(p) => svg_resource_tree(res, p),
            Profile::Pdf(p) => pdf_resource_tree(res, p),
            Profile::Webp(p) => webp_resource_tree(res, p),
            Profile::Compose(p) => compose_resource_tree(res, p),
            Profile::AndroidWebp(p) => android_webp_resource_tree(res, p),
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
        // Print current node
        writeln!(f, "{}", self.name.clone().bold())?;
        for (param_key, param_value) in &self.params {
            let param_key = format!("{param_key}: ");
            writeln!(
                f,
                "{prefix}   {} {}{}",
                "┆".dark_grey(),
                param_key.green(),
                param_value
            )?;
        }

        // Обрабатываем всех детей кроме последнего
        let middle_children = self.children.len().saturating_sub(1);
        for child in self.children.iter().take(middle_children) {
            // Префикс для текущего узла
            write!(f, "{prefix}{corner} ", corner = "├──".dark_grey())?;
            // Префикс для детей текущего узла
            let new_prefix = format!("{prefix}{border}   ", border = "│".dark_grey());
            child.fmt_tree(f, &new_prefix)?;
        }

        // Обрабатываем последнего ребенка (если есть)
        if let Some(last_child) = self.children.last() {
            // Префикс для последнего узла
            write!(f, "{prefix}{corner} ", corner = "╰──".dark_grey())?;
            // Префикс для детей последнего узла (пробелы вместо │)
            let new_prefix = format!("{prefix}    ");
            last_child.fmt_tree(f, &new_prefix)?;
        }

        Ok(())
    }
}

fn png_resource_tree(res: &Resource, p: &PngProfile) -> Node {
    let attrs = &res.attrs;
    let targets = targets_from_resource(res);

    let mut root_node = Node {
        name: attrs.label.to_string(),
        children: Vec::new(),
        params: Vec::new(),
    };
    for t in targets {
        let mut child_nodes = Vec::with_capacity(4);
        let scale = t.scale.unwrap_or(*p.scale);
        if p.legacy_loader {
            child_nodes.push(node!(
                format!("📤 Export PNG from remote {}", attrs.remote),
                [
                    ("node", t.figma_name().to_string()),
                    ("scale", scale.to_string())
                ]
            ));
        } else {
            child_nodes.push(node!(
                format!("📤 Export SVG from remote {}", attrs.remote),
                [("node", t.figma_name().to_string())]
            ));
            child_nodes.push(node!(
                "🎨 Render PNG locally",
                [("scale", scale.to_string())]
            ));
        }
        child_nodes.push(node!(
            "💾 Write to file",
            [("output", format!("{}.png", t.output_name()))]
        ));

        if let Some(variant_id) = t.id {
            let variant_node = Node {
                name: format!("Variant '{}'", variant_id),
                children: child_nodes,
                params: Vec::new(),
            };
            root_node.children.push(variant_node);
        } else {
            root_node.children.append(&mut child_nodes);
        }
    }
    root_node
}

fn svg_resource_tree(res: &Resource, _p: &SvgProfile) -> Node {
    let attrs = &res.attrs;
    let targets = targets_from_resource(res);

    let mut root_node = Node {
        name: attrs.label.to_string(),
        children: Vec::new(),
        params: Vec::new(),
    };
    for t in targets {
        let mut child_nodes = vec![
            node!(
                format!("📤 Export SVG from remote {}", attrs.remote),
                [("node", t.figma_name().to_string())]
            ),
            node!(
                "💾 Write to file",
                [("output", format!("{}.svg", t.output_name()))]
            ),
        ];

        if let Some(variant_id) = t.id {
            let variant_node = Node {
                name: format!("Variant '{}'", variant_id),
                children: child_nodes,
                params: Vec::new(),
            };
            root_node.children.push(variant_node);
        } else {
            root_node.children.append(&mut child_nodes);
        }
    }
    root_node
}

fn pdf_resource_tree(res: &Resource, _p: &PdfProfile) -> Node {
    let attrs = &res.attrs;
    let targets = targets_from_resource(res);

    let mut root_node = Node {
        name: attrs.label.to_string(),
        children: Vec::new(),
        params: Vec::new(),
    };

    for t in targets {
        let mut child_nodes = vec![
            node!(
                format!("📤 Export PDF from remote {}", attrs.remote),
                [("node", t.figma_name().to_string())]
            ),
            node!(
                "💾 Write to file",
                [("output", format!("{}.pdf", t.output_name()))]
            ),
        ];

        if let Some(variant_id) = t.id {
            let variant_node = Node {
                name: format!("Variant '{}'", variant_id),
                children: child_nodes,
                params: Vec::new(),
            };
            root_node.children.push(variant_node);
        } else {
            root_node.children.append(&mut child_nodes);
        }
    }
    root_node
}

fn webp_resource_tree(res: &Resource, p: &WebpProfile) -> Node {
    let attrs = &res.attrs;
    let targets = targets_from_resource(res);

    let mut root_node = Node {
        name: attrs.label.to_string(),
        children: Vec::new(),
        params: Vec::new(),
    };
    for t in targets {
        let mut child_nodes = Vec::with_capacity(4);
        let scale = t.scale.unwrap_or(*p.scale);
        if p.legacy_loader {
            child_nodes.push(node!(
                format!("📤 Export PNG from remote {}", attrs.remote),
                [
                    ("node", t.figma_name().to_string()),
                    ("scale", scale.to_string())
                ]
            ));
        } else {
            child_nodes.push(node!(
                format!("📤 Export SVG from remote {}", attrs.remote),
                [("node", t.figma_name().to_string())]
            ));
            child_nodes.push(node!(
                "🎨 Render PNG locally",
                [("scale", scale.to_string())]
            ));
        }
        child_nodes.push(node!(
            "✨ Transform PNG to WEBP",
            [("quality", p.quality.to_string())]
        ));
        child_nodes.push(node!(
            "💾 Write to file",
            [("output", format!("{}.webp", t.output_name()))]
        ));

        if let Some(variant_id) = t.id {
            let variant_node = Node {
                name: format!("Variant '{}'", variant_id),
                children: child_nodes,
                params: Vec::new(),
            };
            root_node.children.push(variant_node);
        } else {
            root_node.children.append(&mut child_nodes);
        }
    }
    root_node
}

fn compose_resource_tree(res: &Resource, p: &ComposeProfile) -> Node {
    let attrs = &res.attrs;
    let targets = targets_from_resource(res);

    let output_dir = get_output_dir_for_compose_profile(p, &attrs.package_dir);
    let package = match &p.package {
        Some(pkg) if pkg.is_empty() => "Explicitly empty".to_owned(),
        Some(pkg) => pkg.to_owned(),
        None => match get_kotlin_package(&output_dir) {
            Some(pkg) => pkg,
            None => "Undetermined".to_owned(),
        },
    };

    let mut root_node = Node {
        name: attrs.label.to_string(),
        children: Vec::new(),
        params: Vec::new(),
    };
    for t in targets {
        let mut child_nodes = vec![
            node!(
                format!("📤 Export SVG from remote {}", attrs.remote),
                [("node", t.figma_name().to_string())]
            ),
            node!(
                "✨ Transform SVG to Compose",
                [("package", package.to_string())]
            ),
            node!(
                "💾 Write to file",
                [("output", format!("{}.kt", t.output_name()))]
            ),
        ];

        if let Some(variant_id) = t.id {
            let variant_node = Node {
                name: format!("Variant '{}'", variant_id),
                children: child_nodes,
                params: Vec::new(),
            };
            root_node.children.push(variant_node);
        } else {
            root_node.children.append(&mut child_nodes);
        }
    }
    root_node
}

fn android_webp_resource_tree(res: &Resource, p: &AndroidWebpProfile) -> Node {
    let attrs = &res.attrs;
    let targets = targets_from_resource(res);

    let res_name = attrs.label.name.to_string();
    Node {
        name: attrs.label.to_string(),
        children: targets
            .into_iter()
            .map(|target| {
                let variant_name = target.id.as_ref().expect("always present");
                let scale = target.scale.expect("always present");
                let mut child_nodes = Vec::with_capacity(4);
                if p.legacy_loader {
                    child_nodes.push(node!(
                        format!("📤 Export PNG from remote {}", attrs.remote),
                        [
                            ("node", target.figma_name().to_string()),
                            ("scale", scale.to_string())
                        ]
                    ));
                } else {
                    child_nodes.push(node!(
                        format!("📤 Export SVG from remote {}", attrs.remote),
                        [("node", target.figma_name().to_string())]
                    ));
                    child_nodes.push(node!(
                        "🎨 Render PNG locally",
                        [("scale", scale.to_string())]
                    ));
                }
                child_nodes.push(node!(
                    "✨ Transform PNG to WEBP",
                    [("quality", p.quality.to_string())]
                ));
                child_nodes.push(node!(
                    "💾 Write to file",
                    [("output", format!("drawable-{variant_name}/{res_name}.webp"))]
                ));
                Node {
                    name: format!("Variant '{variant_name}'"),
                    children: child_nodes,
                    params: Default::default(),
                }
            })
            .collect(),
        ..Default::default()
    }
}
