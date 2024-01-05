use std::collections::HashSet;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

use clap::{Args, Parser, ValueEnum};
use regex::Regex;

use crate::args::TargetMode::{Ignore, Target};

#[derive(ValueEnum, Clone, PartialOrd, PartialEq, Debug)]
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
    let splitter = Regex::new("\\s+").unwrap();
    let result = splitter.split(s).collect::<Vec<&str>>();
    if result.len() < 2 {
        return Err("Not enough arguments supplied");
    }
    let mode = match result[0] {
        "Ignore" => { Ignore }
        "Target" => { Target }
        _ => { return Err("Invalid target mode specified"); }
    };
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
    source_root_file_path: String,
    ///The target root folder to which all data should be copied to
    target_root_file_path: String,
    // ///File extensions that should be either ignored or copied specifically
    #[arg(long, value_parser = parse_special_options)]
    file_extensions: Option<OptionalHandling>,
    // ///Folders that should be either ignored or copied specifically
    #[arg(long, value_parser = parse_special_options)]
    folders: Option<OptionalHandling>,
    ///Whether links should be followed or ignored
    #[arg(short, long, default_value = "false")]
    follow_links: bool,
    ///Whether the logging should be verbose or not
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
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
        transform_source_to_target_path(&self.source_root_file_path, &self.target_root_file_path, source_path)
    }
}

fn transform_source_to_target_path(source_root_file_path: &str, target_root_file_path: &str, source_path: &Path) -> PathBuf {
    let source_root = Path::new(source_root_file_path);
    match source_path.strip_prefix(source_root) {
        Ok(stripped) => {
            let target_root = Path::new(target_root_file_path);
            target_root.join(stripped)
        }
        Err(err) => {
            panic!("{}", err)
        }
    }
}

fn as_hash_set(vec: Vec<String>) -> HashSet<String> {
    vec.into_iter().collect()
}


#[derive(PartialEq, Debug)]
enum FileExtensionFilterMode {
    Ignored(HashSet<String>),
    Targeted(HashSet<String>),
}

trait FileExtensionFilter {
    fn should_copy(&self, file_name: &str) -> bool;
}

impl FileExtensionFilter for FileExtensionFilterMode {
    fn should_copy(&self, file_name: &str) -> bool {
        let file_extension = file_extension(file_name);
        match self {
            FileExtensionFilterMode::Ignored(ignored) => {
                !ignored.contains(&file_extension)
            }
            FileExtensionFilterMode::Targeted(targeted) => {
                targeted.contains(&file_extension)
            }
        }
    }
}

fn file_extension(file_name: &str) -> String {
    Path::new(file_name)
        .extension()
        .map(|extension| format!(".{}", extension.to_string_lossy()))
        .unwrap_or_else(|| "".to_string())
}


trait FolderFilter {
    fn should_copy(&self, path: &Path) -> bool;
}


#[derive(PartialEq, Debug)]
enum FolderFilterMode {
    Ignored(HashSet<String>),
    Targeted(HashSet<String>),
}

impl FolderFilter for FolderFilterMode {
    fn should_copy(&self, path: &Path) -> bool {
        match self {
            FolderFilterMode::Ignored(ignored) => {
                !ignored.into_iter().any(|ign| path_contains_folder(path, ign))
            }
            FolderFilterMode::Targeted(targeted) => {
                targeted.into_iter().any(|tar| path_contains_folder(path, tar))
            }
        }
    }
}

fn path_contains_folder(path: &Path, folder: &str) -> bool {
    match path.to_str() {
        None => false,
        Some(path_str) => {
            let folders: HashSet<&str> = path_str.split(MAIN_SEPARATOR).filter(|s| !s.is_empty()).collect();
            folders.contains(folder)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::args::{CliArgs, FileExtensionFilterMode, FolderFilterMode, OptionalHandling, parse_special_options, transform_source_to_target_path};
    use crate::args::TargetMode::{Ignore, Target};

    #[test]
    fn test_parse_special_options() {
        let input = "Ignore .jpg .pdf .mp3";
        let result = parse_special_options(input);
        assert!(result.is_ok());
        let (mode, names) = (result.clone().ok().unwrap().target, result.ok().unwrap().values);
        assert_eq!(mode, Ignore);
        assert!(names.contains(&".jpg".to_string()));
        assert!(names.contains(&".pdf".to_string()));
        assert!(names.contains(&".mp3".to_string()));
    }

    #[test]
    fn test_parse_special_options_not_enough_options_supplied() {
        let input = "Target";
        let result = parse_special_options(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_special_options_invalid_mode() {
        let input = "Inore .pdf .wav";
        let result = parse_special_options(input);
        assert!(result.is_err())
    }

    #[test]
    fn test_convert_cli_args_to_args() {
        let cli_args = CliArgs {
            source_root_file_path: "source".to_string(),
            target_root_file_path: "target".to_string(),
            file_extensions: Some(OptionalHandling {
                target: Ignore,
                values: vec![".jpg".to_string(), ".pdf".to_string()],
            }),
            folders: None,
            follow_links: false,
            verbose: Default::default(),
        };
        let result = cli_args.convert();
        assert_eq!(result.follow_links, false);
        let target = match result.folders {
            FolderFilterMode::Ignored(_) => { Ignore }
            FolderFilterMode::Targeted(_) => { Target }
        };
        assert_eq!(target, Ignore);
        let file_extensions = match result.file_extensions {
            FileExtensionFilterMode::Ignored(ext) => { ext }
            FileExtensionFilterMode::Targeted(_) => { panic!("Wrong mode"); }
        };
        assert!(file_extensions.contains(".jpg"));
        assert!(file_extensions.contains(".pdf"));
    }

    #[test]
    fn test_should_copy_folder() {
        let cli_args = CliArgs {
            source_root_file_path: "source".to_string(),
            target_root_file_path: "target".to_string(),
            file_extensions: None,
            folders: Some(OptionalHandling {
                target: Ignore,
                values: vec!["bin".to_string(), "target".to_string()],
            }),
            follow_links: false,
            verbose: Default::default(),
        };
        let current_dir = std::env::current_dir().unwrap();
        let result = cli_args.convert();
        let path = Path::new("test/bin/");
        let binding = current_dir.clone().joined(path);
        let path = binding.as_path();
        let should_copy = result.should_copy(path);
        assert!(!should_copy);

        let path = Path::new("test/file/");
        let binding = current_dir.clone().joined(path);
        let path = binding.as_path();
        let should_copy = result.should_copy(path);
        assert!(should_copy);

        let path = Path::new("test/bin/test/");
        let binding = current_dir.clone().joined(path);
        let path = binding.as_path();
        let should_copy = result.should_copy(path);
        assert!(!should_copy);
    }

    #[test]
    fn test_should_copy_file() {
        let cli_args = CliArgs {
            source_root_file_path: "source".to_string(),
            target_root_file_path: "target".to_string(),
            file_extensions: Some(OptionalHandling {
                target: Ignore,
                values: vec![".jpg".to_string(), ".pdf".to_string()],
            }),
            folders: None,
            follow_links: false,
            verbose: Default::default(),
        };
        let result = cli_args.convert();
        let path = Path::new("test.jpg");
        let should_copy = result.should_copy(path);
        assert!(!should_copy);

        let path = Path::new("file.wav");
        let should_copy = result.should_copy(path);
        assert!(should_copy);

        let path = Path::new("/bin/test.xlsx");
        let should_copy = result.should_copy(path);
        assert!(should_copy);
    }

    #[test]
    fn test_transform_source_to_target_path() {
        let source_root_path = "test/bin";
        let target_root_path = "tar/bin2";
        let path = Path::new("test/bin/path");
        let result = transform_source_to_target_path(source_root_path, target_root_path, path);
        let path = result.to_str().unwrap();
        assert_eq!(path, "tar/bin2/path");
    }

    trait PathBufExt {
        fn joined(self, suffix: &Path) -> PathBuf;
    }

    impl PathBufExt for PathBuf {
        fn joined(mut self, suffix: &Path) -> PathBuf {
            self.push(suffix);
            self
        }
    }
}
