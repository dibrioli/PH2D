#![forbid(unsafe_code)]
//! `cooker` CLI — JSON5 source → postcard cooked binary, content-
//! addressed (HR-6) at the call site by the caller's blake3 hash.
//!
//! Usage:
//!
//! ```text
//! cooker prefab <input.json5> <output.bin>
//! cooker scene  <input.json5> <output.bin>
//! ```
//!
//! Determinism: the cooker is a pure function of (source bytes,
//! ph2d crate versions). CI runs `cook then cook again` and asserts
//! byte-equality (`tests/cooker_determinism.rs`).

use clap::{Parser, Subcommand};
use ph2d_asset_cooker::{cook_prefab_json5, cook_scene_json5};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "cooker",
    about = "PH2D asset cooker (JSON5 source → postcard cooked)"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Cook a prefab source file.
    Prefab {
        #[arg(value_name = "INPUT.json5")]
        input: PathBuf,
        #[arg(value_name = "OUTPUT.cooked")]
        output: PathBuf,
    },
    /// Cook a scene source file.
    Scene {
        #[arg(value_name = "INPUT.json5")]
        input: PathBuf,
        #[arg(value_name = "OUTPUT.cooked")]
        output: PathBuf,
    },
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    let result = match cli.cmd {
        Cmd::Prefab { input, output } => run(&input, &output, cook_prefab_json5, "prefab"),
        Cmd::Scene { input, output } => run(&input, &output, cook_scene_json5, "scene"),
    };
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("✗ {e}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run<F>(
    input: &std::path::Path,
    output: &std::path::Path,
    cook: F,
    label: &'static str,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce(&str) -> Result<Vec<u8>, ph2d_asset_cooker::CookError>,
{
    let src = std::fs::read_to_string(input)?;
    let bytes = cook(&src).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output, &bytes)?;
    let id = ph2d_asset::AssetId::from_bytes(&bytes);
    println!(
        "✓ cooked {} {} → {} ({} bytes, id {})",
        label,
        input.display(),
        output.display(),
        bytes.len(),
        id
    );
    Ok(())
}
