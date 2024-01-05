use clap::Parser;
use colorful::{Color, Colorful};
use env_logger::Builder;
use log::info;

use filescraper::{copy, gather_files_for_copying, print_colorful_when_supported};
use filescraper::args::CliArgs;


fn main() -> anyhow::Result<()> {
    let args: filescraper::args::Arguments = CliArgs::parse().convert();
    Builder::new().filter_level(args.verbose.log_level_filter()).init();
    let files = gather_files_for_copying(&args);
    info!("Found {} files and directories eligible for copying", files.len());
    let duration = copy(args, files);
    let message = format!("Whole operation took {:?}", duration);
    let message = message.as_str();
    print_colorful_when_supported(message, |msg| msg.gradient(Color::Green));
    Ok(())
}



