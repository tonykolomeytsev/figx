use crossterm::style::Stylize;
use lib_label::LabelPattern;
use phase_loading::{Profile, Workspace};

mod error;
pub use error::*;

pub struct FeatureQueryOptions {
    pub pattern: Vec<String>,
    pub output: QueryOutputType,
}

pub enum QueryOutputType {
    Label,
    Profile,
    Package,
    Tree,
}

pub fn query(opts: FeatureQueryOptions) -> Result<()> {
    let pattern = LabelPattern::try_from(opts.pattern)?;
    let ws = phase_loading::load_workspace(pattern, true)?;
    use QueryOutputType::*;
    match &opts.output {
        Label => print_labels(ws)?,
        Profile => print_profiles(ws)?,
        Package => print_packages(ws)?,
        Tree => print_trees(ws)?,
    }
    Ok(())
}

fn print_labels(ws: Workspace) -> Result<()> {
    ws.packages
        .iter()
        .flat_map(|it| &it.resources)
        .for_each(|res| println!("{}", res.attrs.label));
    Ok(())
}

fn print_profiles(ws: Workspace) -> Result<()> {
    ws.packages
        .iter()
        .flat_map(|it| &it.resources)
        .for_each(|res| {
            let label = &res.attrs.label;
            let profile = match res.profile.as_ref() {
                Profile::Png(_) => "png",
                Profile::Svg(_) => "svg",
                Profile::Pdf(_) => "pdf",
                Profile::Webp(_) => "webp",
                Profile::Compose(_) => "compose",
                Profile::AndroidWebp(_) => "android-webp",
            };
            println!("{} {label}", profile.bold())
        });
    Ok(())
}

fn print_packages(ws: Workspace) -> Result<()> {
    for file in &ws.context.fig_files {
        println!("{}", file.package)
    }
    Ok(())
}

fn print_trees(ws: Workspace) -> Result<()> {
    for pkg in ws.packages {
        println!("{}", pkg.label);
        let res_count = pkg.resources.len();
        for (idx, res) in pkg.resources.iter().enumerate() {
            let tab = if idx == res_count - 1 {
                "╰── ".dark_grey()
            } else {
                "├── ".dark_grey()
            };
            let profile = match res.profile.as_ref() {
                Profile::Png(_) => "png",
                Profile::Svg(_) => "svg",
                Profile::Pdf(_) => "pdf",
                Profile::Webp(_) => "webp",
                Profile::Compose(_) => "compose",
                Profile::AndroidWebp(_) => "android-webp",
            };
            println!("{tab}{} {}", profile.bold(), res.attrs.label.name);
        }
        println!()
    }
    Ok(())
}
