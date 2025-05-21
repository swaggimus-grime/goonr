use std::path::PathBuf;
use clap::Parser;
use ml::Scene;

/// CLI for loading a COLMAP scene and exporting Gaussian splats.
#[derive(Parser)]
struct Args {
    /// Input directory containing COLMAP files
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory to save processed data
    #[arg(short, long)]
    output: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let scene = Scene::new(args.input.as_path()).await?;

    Ok(())
}