use crate::{Error, EvalState, Result};
use lib_cache::CacheKey;
use lib_graph_exec::action::{Action, ActionDiagnostics, ExecutionContext};
use lib_label::Label;
use log::{debug, warn};
use std::path::{Path, PathBuf};

use super::stable_action;

pub struct GetKotlinPackageAction {
    pub label: Label,
    pub output_dir: PathBuf,
    /// Default package
    pub default: String,
}

impl Action<CacheKey, Error, EvalState> for GetKotlinPackageAction {
    fn execute(&self, ctx: ExecutionContext<CacheKey, EvalState>) -> Result<CacheKey> {
        let stable_cache_key = CacheKey::builder()
            .set_tag(Self::TAG)
            .write_str(&self.label.to_string())
            .write_str(&self.output_dir.to_string_lossy())
            .write_str(&self.default)
            .build();

        stable_action(&ctx.state.cache, stable_cache_key, false, |cache_key| {
            self.get_kotlin_package_impl(cache_key, &ctx.state)
        })
    }

    fn diagnostics_info(&self) -> ActionDiagnostics {
        ActionDiagnostics {
            name: "Get Kotlin package".to_string(),
            params: Vec::new(),
        }
    }
}

impl GetKotlinPackageAction {
    pub const TAG: u8 = 0x06;

    fn get_kotlin_package_impl(&self, stable_cache_key: CacheKey, state: &EvalState) -> Result<()> {
        if !self.default.is_empty() {
            state
                .cache
                .put::<String>(&stable_cache_key, &self.default)?;
            return Ok(());
        }

        debug!("Seeking kotlin package for {}", self.output_dir.display());
        let mut current_dir = self.output_dir.clone();

        // Step 2: Traverse upwards to find source root
        while current_dir.pop() {
            // Moves to parent directory
            if is_source_root(&current_dir) {
                debug!("Found package from sources root: {}", current_dir.display());
                // Reconstruct original path relative to source root
                let rel_path = self
                    .output_dir
                    .strip_prefix(&current_dir)
                    .expect("current_dir is always subpath of output_dir");
                let package = dir_to_package(rel_path);
                state.cache.put::<String>(&stable_cache_key, &package)?;
                return Ok(());
            }
        }

        state
            .cache
            .put::<String>(&stable_cache_key, &self.default)?;
        warn!(
            "Kotlin package for {} was not found",
            self.output_dir.display()
        );
        Ok(())
    }
}

/// Check if a directory is a known Kotlin source root
fn is_source_root(dir: &Path) -> bool {
    dir.ends_with("src/main/kotlin") ||         // Standard layout
        dir.ends_with("src/main/java") ||       // Android mixed sources
        dir.ends_with("src/commonMain/kotlin") // KMP
}

/// Convert directory path to package name (e.g., "com/example" -> "com.example")
fn dir_to_package(dir: &Path) -> String {
    dir.to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, ".")
}
