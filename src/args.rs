use std::collections::HashSet;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

use clap::{Args, Parser, ValueEnum};

use crate::args::TargetMode::{Ignore, Target};

#[derive(ValueEnum, Clone)]
enum TargetMode {
    Ignore,
    Target,
}

#[derive(Args, Clone)]
struct OptionalHandling {
    target: TargetMode,
    values: Vec<String>,
}

fn parse_special_options(s: &str) -> Result<OptionalHandling, &'static str> {
    let result = s.split("\\s+").collect::<Vec<&str>>();
    if result.len() < 2 {
        return Err("Not enough arguments supplied");
    }
    let mode = match result[0] {
        "Ignore" => { Ignore }
        "Target" => { Target }
        _ => { return Err("Invalid target mode specified"); }
    };
    /*
        let extensions = result.iter().skip(1)
            .filter(|ext| !ext.contains("."))
            .map(|ext| String::from(".").push_str(*ext))
            .collect::<Vec<String>>();
     */
    let extensions = result.into_iter().skip(1)
        .map(|ext| ext.to_string())
        .collect();
    Ok(OptionalHandling {
        target: mode,
        values: extensions,
    })
}

#[derive(Parser, Clone)]
#[clap(author = "Shahondin1624", about = "A simple cli-application for fast scraping of data from a system")]
pub struct CliArgs {
    ///The root folder from which all data should be scraped recursively
    pub source_root_file_path: String,
    ///The target root folder to which all data should be copied to
    pub target_root_file_path: String,
    // ///File extensions that should be either ignored or copied specifically
    #[arg(long, value_parser = parse_special_options)]
    pub file_extensions: Option<OptionalHandling>,
    // ///Folders that should be either ignored or copied specifically
    #[arg(long, value_parser = parse_special_options)]
    pub folders: Option<OptionalHandling>,
    ///Whether links should be followed or ignored
    #[arg(short, long, default_value = "false")]
    pub follow_links: bool,
    ///Whether the logging should be verbose or not
    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}

impl CliArgs {
    pub fn convert(&self) -> Arguments {
        let file_extensions = match &self.file_extensions {
            None => { FileExtensionFilterMode::Ignored(HashSet::new()) }
            Some(inner) => {
                let extensions: HashSet<String> = inner.clone().values.iter()
                    .map(|s| if s.starts_with('.') { s.clone() } else { format!(".{}", s) })
                    .collect();
                match inner.target {
                    Ignore => { FileExtensionFilterMode::Ignored(extensions) }
                    Target => { FileExtensionFilterMode::Targeted(extensions) }
                }
            }
        };
        let folders = match &self.folders {
            None => { FolderFilterMode::Ignored(HashSet::new()) }
            Some(inner) => {
                match inner.target {
                    Ignore => { FolderFilterMode::Ignored(as_hash_set(inner.values.clone())) }
                    Target => { FolderFilterMode::Targeted(as_hash_set(inner.values.clone())) }
                }
            }
        };
        Arguments {
            source_root_file_path: self.source_root_file_path.clone(),
            target_root_file_path: self.target_root_file_path.clone(),
            file_extensions,
            folders,
            follow_links: self.follow_links,
            verbose: self.verbose.clone(),
        }
    }
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

fn as_hash_set(vec: Vec<String>) -> HashSet<String> {
    vec.into_iter().collect()
}


enum FileExtensionFilterMode {
    Ignored(HashSet<String>),
    Targeted(HashSet<String>),
}

trait FileExtensionFilter {
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

trait FolderFilter {
    fn should_copy(&self, path: &Path) -> bool;
}


enum FolderFilterMode {
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
