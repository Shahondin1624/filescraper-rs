use std::collections::HashSet;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

use clap::{Args, Parser, ValueEnum};

#[derive(Parser, Clone)]
#[command(author = "Shahondin1624", about = "A simple cli-application for fast scraping of data from a system")]
pub struct CliArgs {
    ///The root folder from which all data should be scraped recursively
    pub source_root_file_path: String,
    ///The target root folder to which all data should be copied to
    pub target_root_file_path: String,
    // ///File extensions that should be either ignored or copied specifically
    #[arg(short, long)]
    pub file_extensions: Option<FileExtensions>,
    // ///Folders that should be either ignored or copied specifically
    #[arg(short, long)]
    pub folders: Option<Folders>,
    ///Whether links should be followed or ignored
    pub follow_links: bool,
    ///Whether the logging should be verbose or not
    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}

#[derive(ValueEnum, Clone)]
pub enum TargetMode {
    Ignore,
    Target,
}

#[derive(Args, Clone)]
pub struct FileExtensions {
    target: TargetMode,
    values: StringArgs,
}

#[derive(Args, Clone)]
pub struct Folders {
    target: TargetMode,
    values: StringArgs,
}

#[derive(Args, Clone)]
pub struct StringArgs {
    values: Vec<String>,
}

impl CliArgs {
    // pub fn convert(&self) -> Arguments {}
}

pub struct Arguments {
    pub source_root_file_path: String,
    pub target_root_file_path: String,
    pub file_extensions: FileExtensionFilterMode,
    pub folders: FolderFilterMode,
    pub follow_links: bool,
    pub verbose: clap_verbosity_flag::Verbosity,
}

impl Arguments {
    pub fn should_copy(&self, path: &Path) -> bool {
        return if path.is_dir() {
            self.folders.should_copy(path)
        } else {
            path.file_name()
                .and_then(|file_name| file_name.to_str())
                .and_then(|file_name| Some(self.file_extensions.should_copy(file_name)))
                .or_else(|| Some(false))
                .unwrap()
        };
    }

    pub fn transform_source_to_target_path(&self, source_path: &Path) -> PathBuf {
        let source_root = Path::new(&self.source_root_file_path);
        match source_path.strip_prefix(source_root) {
            Ok(stripped) => {
                let target_root = Path::new(&self.target_root_file_path);
                target_root.join(stripped)
            }
            Err(err) => {
                panic!("{}", err)
            }
        }
    }
}

impl Default for Arguments {
    fn default() -> Self {
        todo!()
    }
}

fn as_hash_set(vec: Vec<String>) -> HashSet<String> {
    vec.into_iter().collect()
}


pub enum FileExtensionFilterMode {
    Ignored(HashSet<String>),
    Targeted(HashSet<String>),
}

pub trait FileExtensionFilter {
    fn should_copy(&self, file_name: &str) -> bool;
}

impl FileExtensionFilter for FileExtensionFilterMode {
    fn should_copy(&self, file_name: &str) -> bool {
        match self {
            FileExtensionFilterMode::Ignored(ignored) => {
                ignored.contains(file_name)
            }
            FileExtensionFilterMode::Targeted(targeted) => {
                targeted.contains(file_name)
            }
        }
    }
}

pub trait FolderFilter {
    fn should_copy(&self, path: &Path) -> bool;
}


pub enum FolderFilterMode {
    Ignored(HashSet<String>),
    Targeted(HashSet<String>),
}

impl FolderFilter for FolderFilterMode {
    fn should_copy(&self, path: &Path) -> bool {
        match self {
            FolderFilterMode::Ignored(ignored) => {
                ignored.into_iter().any(|ign| path_contains_folder(path, ign))
            }
            FolderFilterMode::Targeted(targeted) => {
                targeted.into_iter().any(|tar| path_contains_folder(path, tar))
            }
        }
    }
}

fn path_contains_folder(path: &Path, folder: &str) -> bool {
    return match path.to_str() {
        None => {
            false
        }
        Some(path_str) => {
            let folders: HashSet<&str> = path_str.split(MAIN_SEPARATOR).collect();
            folders.contains(folder)
        }
    };
}
