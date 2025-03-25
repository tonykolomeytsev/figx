use crate::{
    ConvertToWebpAction, DownloadImgAction, Error, EvalState, ExportImgAction, FetchRemoteAction,
    FindNodeAction, GetKotlinPackageAction, MaterializeImgAction, NoOpAction, Result,
    ScalePngAction, SvgToComposeAction,
};
use lib_cache::CacheKey;
use lib_graph_exec::{
    action::{ActionGraph, ActionGraphBuilder, ActionId},
    graph_deps,
};
use lib_label::Label;
use phase_loading::{AndroidDensity, ComposeProfile, Profile, Resource, Workspace};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

pub struct EvalBuilder<'a> {
    affected_resources: Vec<&'a Resource>,
    inner: ActionGraphBuilder<CacheKey, Error, EvalState>,
    remotes_to_fetch: HashMap<String, ActionId>,
    involved_actions: HashMap<Label, InvolvedAction>,
}

enum InvolvedAction {
    Single(ActionId),
    Multiple(Vec<ActionId>),
}

impl<'a> EvalBuilder<'a> {
    const DEFAULT_ANDROID_WEBP_SCALE: f32 = 4.0;

    pub fn from_workspace(workspace: &'a Workspace) -> Self {
        Self {
            affected_resources: workspace
                .packages
                .iter()
                .flat_map(|it| it.resources.iter())
                .collect(),
            inner: ActionGraph::builder(),
            remotes_to_fetch: Default::default(),
            involved_actions: Default::default(),
        }
    }

    /// Fetch only those remotes, which resources are affected by user
    pub fn fetch_remotes(mut self, force: bool) -> Self {
        let affected_remotes = self
            .affected_resources
            .iter()
            .map(|res| &res.attrs.remote)
            .collect::<HashSet<_>>();

        for remote in affected_remotes {
            let fetch_remote = self.inner.add_action(FetchRemoteAction {
                remote: remote.clone(),
                force_refetch: force,
            });

            // remember for next stages
            self.remotes_to_fetch
                .insert(remote.id.clone(), fetch_remote);
        }
        self
    }

    /// Import resources from remotes, but do not transform nor materialize
    pub fn fetch_resources(mut self) -> Self {
        for res in self.affected_resources.iter() {
            let fetch_remote = *self
                .remotes_to_fetch
                .get(&res.attrs.remote.id)
                .expect("affected remotes must be configured before this call");
            let find_node_id = self.inner.add_action(FindNodeAction {
                label: res.attrs.label.clone(),
                node_name: res.attrs.node_name.clone(),
            });
            let export_img = self.inner.add_action(ExportImgAction {
                label: res.attrs.label.clone(),
                remote: res.attrs.remote.clone(),
                format: match res.profile.as_ref() {
                    Profile::Png(_) => "png",
                    Profile::Svg(_) => "svg",
                    Profile::Pdf(_) => "pdf",
                    Profile::Webp(_) => "png",
                    Profile::Compose(_) => "svg",
                    Profile::AndroidWebp(_) => "png",
                }
                .to_string(),
                scale: match res.profile.as_ref() {
                    Profile::Png(p) => p.scale,
                    Profile::Svg(p) => p.scale,
                    Profile::Pdf(p) => p.scale,
                    Profile::Webp(p) => p.scale,
                    Profile::Compose(p) => p.scale,
                    Profile::AndroidWebp(_) => Self::DEFAULT_ANDROID_WEBP_SCALE,
                },
            });
            let download_img = self.inner.add_action(DownloadImgAction {
                label: res.attrs.label.clone(),
                remote: res.attrs.remote.clone(),
            });
            graph_deps! { self.inner, download_img => export_img => find_node_id => fetch_remote };

            // remember for next stages
            self.involved_actions.insert(
                res.attrs.label.clone(),
                InvolvedAction::Single(download_img),
            );
        }
        self
    }

    pub fn transform_resources(mut self) -> Self {
        for res in self.affected_resources.iter() {
            let download_img = match self.involved_actions.remove(&res.attrs.label) {
                Some(InvolvedAction::Single(id)) => id,
                _ => panic!("resources to fetch must be configured before this call"),
            };
            let involved_action = match res.profile.as_ref() {
                // No transformation
                Profile::Png(_) | Profile::Svg(_) | Profile::Pdf(_) => {
                    InvolvedAction::Single(download_img)
                }

                // PNG to WEBP transformation
                Profile::Webp(p) => {
                    let png_to_webp = self
                        .inner
                        .add_action(ConvertToWebpAction { quality: p.quality });
                    graph_deps! { self.inner, png_to_webp => download_img };
                    InvolvedAction::Single(png_to_webp)
                }

                // SVG to Compose transformation
                Profile::Compose(p) => {
                    let svg_to_compose = self.inner.add_action(SvgToComposeAction {
                        image_name: res.attrs.label.name.to_string(),
                        kotlin_explicit_api: p.kotlin_explicit_api,
                    });
                    let get_kotlin_package = self.inner.add_action(GetKotlinPackageAction {
                        label: res.attrs.label.clone(),
                        output_dir: {
                            get_output_dir_for_compose_profile(p, &res.attrs.package_dir)
                        },
                        default: p.package.clone(),
                    });
                    graph_deps! { self.inner, svg_to_compose => download_img };
                    graph_deps! { self.inner, svg_to_compose => get_kotlin_package }
                    InvolvedAction::Single(svg_to_compose)
                }

                // PNG to WEBP multiple times
                Profile::AndroidWebp(p) => {
                    let mut involved_actions = Vec::with_capacity(p.scales.len());
                    for scale in p.scales.iter() {
                        let scale_png = self.inner.add_action(ScalePngAction {
                            factor: scale_factor(scale),
                            density: density_name(scale).to_string(),
                        });
                        let png_to_webp = self
                            .inner
                            .add_action(ConvertToWebpAction { quality: p.quality });
                        graph_deps! { self.inner, png_to_webp => scale_png => download_img };
                        involved_actions.push(png_to_webp); // only latest
                    }
                    InvolvedAction::Multiple(involved_actions)
                }
            };

            // remember for next stages
            self.involved_actions
                .insert(res.attrs.label.clone(), involved_action);
        }
        self
    }

    pub fn materialize_resources(mut self) -> Self {
        for res in self.affected_resources.iter() {
            let involved_action = match res.profile.as_ref() {
                Profile::Png(p) => {
                    let transform_res = match self.involved_actions.remove(&res.attrs.label) {
                        Some(InvolvedAction::Single(id)) => id,
                        _ => unreachable!(),
                    };
                    let materialize_img = self.inner.add_action(MaterializeImgAction {
                        output_dir: res.attrs.package_dir.join(&p.output_dir),
                        image_name: res.attrs.label.name.to_string(),
                        extension: "png".to_string(),
                    });
                    graph_deps! { self.inner, materialize_img => transform_res };
                    InvolvedAction::Single(materialize_img)
                }
                Profile::Svg(p) => {
                    let transform_res = match self.involved_actions.remove(&res.attrs.label) {
                        Some(InvolvedAction::Single(id)) => id,
                        _ => unreachable!(),
                    };
                    let materialize_img = self.inner.add_action(MaterializeImgAction {
                        output_dir: res.attrs.package_dir.join(&p.output_dir),
                        image_name: res.attrs.label.name.to_string(),
                        extension: "svg".to_string(),
                    });
                    graph_deps! { self.inner, materialize_img => transform_res };
                    InvolvedAction::Single(materialize_img)
                }
                Profile::Pdf(p) => {
                    let transform_res = match self.involved_actions.remove(&res.attrs.label) {
                        Some(InvolvedAction::Single(id)) => id,
                        _ => unreachable!(),
                    };
                    let materialize_img = self.inner.add_action(MaterializeImgAction {
                        output_dir: res.attrs.package_dir.join(&p.output_dir),
                        image_name: res.attrs.label.name.to_string(),
                        extension: "pdf".to_string(),
                    });
                    graph_deps! { self.inner, materialize_img => transform_res };
                    InvolvedAction::Single(materialize_img)
                }
                Profile::Webp(p) => {
                    let transform_res = match self.involved_actions.remove(&res.attrs.label) {
                        Some(InvolvedAction::Single(id)) => id,
                        _ => unreachable!(),
                    };
                    let materialize_img = self.inner.add_action(MaterializeImgAction {
                        output_dir: res.attrs.package_dir.join(&p.output_dir),
                        image_name: res.attrs.label.name.to_string(),
                        extension: "webp".to_string(),
                    });
                    graph_deps! { self.inner, materialize_img => transform_res };
                    InvolvedAction::Single(materialize_img)
                }
                Profile::Compose(p) => {
                    let transform_res = match self.involved_actions.remove(&res.attrs.label) {
                        Some(InvolvedAction::Single(id)) => id,
                        _ => unreachable!(),
                    };
                    let materialize_img = self.inner.add_action(MaterializeImgAction {
                        output_dir: get_output_dir_for_compose_profile(p, &res.attrs.package_dir),
                        image_name: res.attrs.label.name.to_string(),
                        extension: "kt".to_string(),
                    });
                    graph_deps! { self.inner, materialize_img => transform_res };
                    InvolvedAction::Single(materialize_img)
                }
                Profile::AndroidWebp(p) => {
                    let transforms = match self.involved_actions.remove(&res.attrs.label) {
                        Some(InvolvedAction::Multiple(ids)) => ids,
                        _ => unreachable!(),
                    };
                    let mut involved_actions = Vec::with_capacity(p.scales.len());
                    for (scale, tfx_action) in p.scales.iter().zip(transforms) {
                        let qualifier = density_name(scale);
                        let materialize_img = self.inner.add_action(MaterializeImgAction {
                            output_dir: res
                                .attrs
                                .package_dir
                                .join(&p.android_res_dir)
                                .join(format!("drawable-{qualifier}")),
                            image_name: res.attrs.label.name.to_string(),
                            extension: "webp".to_string(),
                        });
                        graph_deps! { self.inner, materialize_img => tfx_action };
                        involved_actions.push(materialize_img); // only latest
                    }
                    InvolvedAction::Multiple(involved_actions)
                }
            };

            // remember for next stages
            self.involved_actions
                .insert(res.attrs.label.clone(), involved_action);
        }
        self
    }

    pub fn wrap_to_diagnostics(mut self) -> Self {
        for res in self.affected_resources.iter() {
            let materialize_img = self
                .involved_actions
                .remove(&res.attrs.label)
                .expect("resources to materialize must be configured before this call");
            let no_op = self.inner.add_action(NoOpAction {
                label: res.attrs.label.clone(),
            });
            match materialize_img {
                InvolvedAction::Single(id) => graph_deps! { self.inner, no_op => id },
                InvolvedAction::Multiple(ids) => {
                    for id in ids {
                        graph_deps! { self.inner, no_op => id }
                    }
                }
            }
        }
        self
    }

    pub fn build(self) -> Result<ActionGraph<CacheKey, Error, EvalState>> {
        Ok(self.inner.build()?)
    }
}

fn get_output_dir_for_compose_profile(p: &ComposeProfile, abs_package_dir: &Path) -> PathBuf {
    let kt_src_dir = &p.src_dir;
    let kt_package = &p.package.replace('.', "/");
    // {abs_package_dir}/{kt_src_dir}/{pkg_dir}
    abs_package_dir.join(kt_src_dir).join(kt_package)
}

fn scale_factor(d: &AndroidDensity) -> f32 {
    use AndroidDensity::*;
    let density = match d {
        LDPI => 0.75,
        MDPI => 1.0,
        HDPI => 1.5,
        XHDPI => 2.0,
        XXHDPI => 3.0,
        XXXHDPI => 4.0,
    };
    density / 4.0
}

fn density_name(d: &AndroidDensity) -> &str {
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
