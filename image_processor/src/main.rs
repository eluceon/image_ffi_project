use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;

use image_processor::{error::AppError, load_image, plugin_loader, save_image};

#[derive(Parser)]
#[command(name = "image-processor")]
#[command(about = "Load a PNG, apply a processing plugin, and save the result")]
struct Cli {
    /// Path to the input PNG image
    #[arg(short, long)]
    input: PathBuf,

    /// Path to save the processed PNG image
    #[arg(short, long)]
    output: PathBuf,

    /// Plugin name (without extension, e.g. "mirror_plugin" for libmirror_plugin.so)
    #[arg(short, long)]
    plugin: String,

    /// Path to the text file with plugin parameters (JSON)
    #[arg(long = "params")]
    params: PathBuf,

    /// Directory containing the plugin library (default: target/debug)
    #[arg(short = 'P', long = "plugin-path", default_value = "target/debug")]
    plugin_path: PathBuf,
}

fn run(cli: Cli) -> Result<(), AppError> {
    log::info!("Loading image from {:?}", cli.input);
    let (width, height, mut rgba_data) = load_image(&cli.input)?;
    log::info!(
        "Image loaded: {}x{}, {} bytes",
        width,
        height,
        rgba_data.len()
    );

    let params = plugin_loader::read_params(&cli.params)?;
    log::debug!("Plugin params: {:?}", params);

    let plugin = plugin_loader::Plugin::load(&cli.plugin_path, &cli.plugin)?;
    log::info!("Plugin '{}' loaded from {:?}", cli.plugin, cli.plugin_path);

    log::info!("Calling process_image...");
    // SAFETY: rgba_data is a valid Vec<u8> of size width * height * 4.
    // The pointer remains valid for the duration of the unsafe block.
    // params is a valid CString (null-terminated, no interior nulls).
    let status = unsafe { plugin.process(width, height, rgba_data.as_mut_ptr(), params.as_ptr()) };
    if status != 0 {
        return Err(AppError::PluginExecutionFailed {
            plugin: cli.plugin,
            code: status,
        });
    }

    log::info!("Saving processed image to {:?}", cli.output);
    save_image(&cli.output, width, height, rgba_data)?;
    log::info!("Done.");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    run(cli).context("Image processing failed")?;
    Ok(())
}
