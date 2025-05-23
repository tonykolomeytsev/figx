use lib_label::LabelPattern;
use owo_colors::OwoColorize;
use phase_evaluation::actions::{
    import_android_webp::{density_name, scale_factor},
    import_compose::{get_kotlin_package, get_output_dir_for_compose_profile},
};
use phase_loading::Profile;
mod error;
pub use error::*;

pub struct FeatureAQueryOptions {
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

pub fn query(opts: FeatureAQueryOptions) -> Result<()> {
    let pattern = LabelPattern::try_from(opts.pattern)?;
    let ws = phase_loading::load_workspace(pattern)?;

    let mut nodes = Vec::with_capacity(1024);
    for res in ws.packages.iter().flat_map(|pkg| &pkg.resources) {
        let res_label = res.attrs.label.to_string();
        let res_name = res.attrs.label.name.to_string();
        let node_name = res.attrs.node_name.clone();
        let node = match res.profile.as_ref() {
            Profile::Png(p) => node!(
                res_label,
                node!(
                    "Write to file",
                    [("file", format!("{res_name}.png"))],
                    node!(
                        "Download PNG",
                        node!(
                            "Export PNG",
                            [("node", node_name), ("scale", p.scale.to_string())],
                            node!(format!("Fetch remote {}", res.attrs.remote), [])
                        )
                    )
                )
            ),
            Profile::Svg(p) => node!(
                res_label,
                node!(
                    "Write to file",
                    [("file", format!("{res_name}.svg"))],
                    node!(
                        "Download SVG",
                        node!(
                            "Export SVG",
                            [("node", node_name), ("scale", p.scale.to_string())],
                            node!(format!("Fetch remote {}", res.attrs.remote), [])
                        )
                    )
                )
            ),
            Profile::Pdf(p) => node!(
                res_label,
                node!(
                    "Write to file",
                    [("file", format!("{res_name}.pdf"))],
                    node!(
                        "Download PDF",
                        node!(
                            "Export PDF",
                            [("node", node_name), ("scale", p.scale.to_string())],
                            node!(format!("Fetch remote {}", res.attrs.remote), [])
                        )
                    )
                )
            ),
            Profile::Webp(p) => node!(
                res_label,
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
                                [("node", node_name), ("scale", p.scale.to_string())],
                                node!(format!("Fetch remote {}", res.attrs.remote), [])
                            )
                        )
                    )
                )
            ),
            Profile::Compose(p) => {
                let output_dir = get_output_dir_for_compose_profile(p, &res.attrs.package_dir);
                let package = match &p.package {
                    Some(pkg) if pkg.is_empty() => "Explicitly empty".to_owned(),
                    Some(pkg) => pkg.to_owned(),
                    None => match get_kotlin_package(&output_dir) {
                        Some(pkg) => pkg,
                        None => "Undetermined".to_owned(),
                    },
                };
                node!(
                    res_label,
                    node!(
                        "Write to file",
                        [("file", format!("{res_name}.kt"))],
                        node!(
                            "Transform SVG to Compose",
                            [("package", package)],
                            node!(
                                "Download PNG",
                                node!(
                                    "Export PNG",
                                    [("node", node_name), ("scale", p.scale.to_string())],
                                    node!(format!("Fetch remote {}", res.attrs.remote), [])
                                )
                            )
                        )
                    )
                )
            }
            Profile::AndroidWebp(p) => Node {
                name: res_label,
                children: p
                    .scales
                    .iter()
                    .map(|it| {
                        let density_name = density_name(it);
                        let scale_factor = scale_factor(it);
                        node!(
                            "Write to file",
                            [("output", format!("drawable-{density_name}/{res_name}.webp"))],
                            node!(
                                "Transform PNG to WEBP",
                                [("quality", p.quality.to_string())],
                                node!(
                                    "Download PNG",
                                    node!(
                                        "Export PNG",
                                        [
                                            ("node", node_name.clone()),
                                            ("scale", scale_factor.to_string())
                                        ],
                                        node!(format!("Fetch remote {}", res.attrs.remote), [])
                                    )
                                )
                            )
                        )
                    })
                    .collect(),
                ..Default::default()
            },
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
