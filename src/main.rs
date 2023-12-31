use std::env::consts::OS;
use std::path::Path;
use std::time::{Duration, Instant};

use atomic_counter::{AtomicCounter, RelaxedCounter};
use colorful::{Color, Colorful};
use colorful::core::color_string::CString;
use env_logger::Builder;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use log::{debug, info, warn};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walkdir::{DirEntry, WalkDir};

use crate::args::Arguments;

mod args;

fn main() -> anyhow::Result<()> {
    let args = Arguments::default();
    Builder::new().filter_level(args.verbose.log_level_filter()).init();
    let files: Vec<DirEntry> = WalkDir::new(Path::new(&args.source_root_file_path))
        .follow_links(args.follow_links)
        .into_iter().filter(|e| {
        match e {
            Ok(_) => { true }
            Err(err) => {
                debug!("Could not access {}", err);
                false
            }
        }
    })
        .filter_map(|e| e.ok())
        .filter(|e| {
            if !args.should_copy(e.path()) {
                debug!("Skipped copying for {}", e.path().to_str().unwrap_or_else(|| "<could not read path>"));
                return false;
            }
            true
        })
        .collect();
    info!("Found {} files and directories eligible for copying", files.len());
    let duration = copy(args, files);
    let message = format!("Whole operation took {:?}", duration);
    let message = message.as_str();
    print_colorful_when_supported(message, |msg| msg.gradient(Color::Green));
    Ok(())
}

fn copy(args: Arguments, files: Vec<DirEntry>) -> Duration {
    let start_time = Instant::now();
    info!("Beginning copy-process...");
    let counter = RelaxedCounter::new(0);
    let bar = create_progress_bar(files.len() as u64);
    files.par_iter().progress_with(bar).for_each(|entry| {
        let source_path = entry.path();
        let source_path_string = source_path.to_string_lossy().to_string();
        let source_path_parent = source_path.parent().unwrap();
        std::fs::create_dir_all(source_path_parent).unwrap();
        let target_path = args.transform_source_to_target_path(source_path);
        match std::fs::copy(source_path, target_path) {
            Ok(_) => { debug!("Successfully copied {}", source_path_string) }
            Err(err) => { warn!("Failed to copy {} due to {}", source_path_string, err) }
        }
        counter.inc();
    });
    start_time.elapsed()
}

pub fn is_colorful_supported() -> bool {
    "linux".eq(OS)
}

fn print_colorful_when_supported(message: &str, function: fn(&str) -> CString) {
    if is_colorful_supported() {
        let message = function(message);
        println!("{}", message);
    } else {
        println!("{}", message);
    }
}

fn create_progress_bar(items: u64) -> ProgressBar {
    let bar = ProgressBar::new(items);
    bar.enable_steady_tick(Duration::from_secs(1));
    let style: Option<ProgressStyle> = match ProgressStyle::with_template
        ("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}") {
        Ok(_style) => { Some(_style) }
        Err(_) => {
            debug!("Could not retrieve progress bar style!");
            None
        }
    };
    if style.is_some() {
        let mut style = style.unwrap();
        style = style.progress_chars("##-");
        bar.set_style(style);
    }
    bar
}