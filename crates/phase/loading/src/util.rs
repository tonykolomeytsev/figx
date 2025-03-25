use crate::Error;
use crate::Result;
use std::path::Path;
use std::path::PathBuf;

#[allow(unused)]
pub(crate) struct FileWithParentDir {
    pub file: PathBuf,
    pub parent_dir: PathBuf,
}

/// Find a file in the current directory or one of its ancestors
/// 
/// Searches for a file with the given name starting from the specified directory 
/// and traversing up through its ancestors. If the file is found, returns both 
/// the file path and the directory it was found in.
/// 
/// # Example
/// 
/// ```ignore
/// # use std::path::Path;
/// # use phase_loading::util::find_file_in_ancestors;
/// let start = Path::new("/home/user/project/src");
/// if let Some(result) = find_file_in_ancestors("config.toml", start) {
///     println!("Found at: {}", result.file.display());
///     println!("Parent directory: {}", result.parent_dir.display());
/// } else {
///     println!("File not found");
/// }
/// ```
pub(crate) fn find_file_in_ancestors(
    file_name: &str,
    start_dir: &Path,
) -> Option<FileWithParentDir> {
    for dir in start_dir.ancestors() {
        let candidate = dir.join(file_name);
        if candidate.is_file() {
            return Some(FileWithParentDir {
                file: candidate,
                parent_dir: dir.to_path_buf(),
            });
        }
    }
    None
}

pub(crate) fn find_files_in_child_dirs(
    file_name: &str,
    start_dir: &Path,
) -> Result<Vec<FileWithParentDir>> {
    let mut builder = ignore::WalkBuilder::new(start_dir);
    builder.standard_filters(true);
    builder.hidden(false);
    builder.max_depth(Some(std::usize::MAX)); // Search all subdirectories
    
    let mut results = vec![];
    for entry in builder.build() {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            if name == file_name {
                let file = entry.into_path();
                let parent_dir = file
                    .parent()
                    .ok_or(Error::internal(format!(
                        "Cannot obtain parent dir of {:?}",
                        file
                    )))?
                    .to_path_buf();
                results.push(FileWithParentDir { file, parent_dir });
            }
        }
    }
    Ok(results)
}