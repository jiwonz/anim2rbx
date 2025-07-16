//! Command-line tool for converting animation files to Roblox KeyframeSequence format

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use anyhow::Result;
use clap::Parser;
use log::{info, debug};

use anim2rbx::AnimationConverter;

/// Convert animation files to Roblox KeyframeSequence format
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input animation file (FBX, COLLADA, etc.)
    input: String,

    /// Output .rbxm file
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output: Option<String>,

    /// Don't filter out bones with identical poses
    #[arg(long = "no-filter")]
    no_filter: bool,

    /// Epsilon value for floating-point comparisons
    #[arg(long = "epsilon", default_value = "0.00001")]
    epsilon: f32,

    /// Enable verbose logging
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logger based on verbose flag
    let log_level = if args.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    let output_file = args.output
        .as_deref()
        .unwrap_or_else(|| {
            // Generate output filename from input
            let path = Path::new(&args.input);
            let stem = path.file_stem().unwrap().to_str().unwrap();
            Box::leak(format!("{}.rbxm", stem).into_boxed_str())
        });

    info!("Converting {} to {}", args.input, output_file);
    debug!("Filter identical bones: {}", !args.no_filter);
    debug!("Epsilon value: {}", args.epsilon);

    // Configure the converter using the new API
    let converter = AnimationConverter::new(!args.no_filter, args.epsilon);

    // Convert the file
    let kfs = converter.convert_file_to_weakdom(&args.input)?;

    // Write to output file
    let output = BufWriter::new(File::create(output_file)?);
    rbx_binary::to_writer(output, &kfs, &[kfs.root_ref()])?;

    info!("Successfully converted animation to {}", output_file);

    Ok(())
}
