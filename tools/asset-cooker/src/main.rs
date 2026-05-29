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

use clap::{Parser, Subcommand, ValueEnum};
use ph2d_asset_cooker::texture::{self, AssetClass, CookOptions, Tier};
use ph2d_asset_cooker::{cook_prefab_json5, cook_scene_json5};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "cooker",
    about = "PH2D asset cooker (JSON5 source → postcard cooked, PNG → KTX2)"
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
    /// W1.T3 (ADR-0055-v4) — cook a PNG source into a KTX2 GPU-compressed texture.
    Texture {
        #[command(subcommand)]
        sub: TextureCmd,
    },
}

#[derive(Subcommand, Debug)]
enum TextureCmd {
    /// Cook one input PNG into one KTX2 output for the given tier + asset class.
    Cook {
        #[arg(value_name = "INPUT.png")]
        input: PathBuf,
        #[arg(value_name = "OUTPUT.ktx2")]
        output: PathBuf,
        #[arg(long, value_enum, default_value_t = TierArg::Desktop)]
        tier: TierArg,
        #[arg(long, value_enum, default_value_t = AssetClassArg::SpriteColor)]
        asset_class: AssetClassArg,
    },
    /// W1.T6 — cook one input PNG into N KTX2 outputs (one per tier) using the
    /// canonical target matrix. Output files named `<stem>-<tier>.ktx2` no
    /// `--output-dir`. Stdout linha per artifact emitido.
    CookAll {
        #[arg(value_name = "INPUT.png")]
        input: PathBuf,
        #[arg(long, value_name = "OUTPUT_DIR")]
        output_dir: PathBuf,
        #[arg(long, value_enum, default_value_t = AssetClassArg::SpriteColor)]
        asset_class: AssetClassArg,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum TierArg {
    Desktop,
    Mobile,
    Web,
    LowEnd,
    Constrained,
}

impl From<TierArg> for Tier {
    fn from(v: TierArg) -> Self {
        match v {
            TierArg::Desktop => Tier::Desktop,
            TierArg::Mobile => Tier::Mobile,
            TierArg::Web => Tier::Web,
            TierArg::LowEnd => Tier::LowEnd,
            TierArg::Constrained => Tier::Constrained,
        }
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum AssetClassArg {
    SpriteColor,
    CriticalUi,
    SingleChannel,
    NormalMap,
}

impl From<AssetClassArg> for AssetClass {
    fn from(v: AssetClassArg) -> Self {
        match v {
            AssetClassArg::SpriteColor => AssetClass::SpriteColor,
            AssetClassArg::CriticalUi => AssetClass::CriticalUi,
            AssetClassArg::SingleChannel => AssetClass::SingleChannel,
            AssetClassArg::NormalMap => AssetClass::NormalMap,
        }
    }
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    let result = match cli.cmd {
        Cmd::Prefab { input, output } => run(&input, &output, cook_prefab_json5, "prefab"),
        Cmd::Scene { input, output } => run(&input, &output, cook_scene_json5, "scene"),
        Cmd::Texture {
            sub:
                TextureCmd::Cook {
                    input,
                    output,
                    tier,
                    asset_class,
                },
        } => run_texture_cook(&input, &output, tier.into(), asset_class.into()),
        Cmd::Texture {
            sub:
                TextureCmd::CookAll {
                    input,
                    output_dir,
                    asset_class,
                },
        } => run_texture_cook_all(&input, &output_dir, asset_class.into()),
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

fn run_texture_cook_all(
    input: &std::path::Path,
    output_dir: &std::path::Path,
    asset_class: AssetClass,
) -> Result<(), Box<dyn std::error::Error>> {
    let png_bytes = std::fs::read(input)?;
    // cook_all retorna TODOS os bytes em memória antes de qualquer write — se
    // o cook em si falhar, FS continua intacto.
    let artifacts = texture::cook_all(&png_bytes, asset_class)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    std::fs::create_dir_all(output_dir)?;
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("cooked");

    // Audit W1.T6 θ-H1 fix: write-with-cleanup-on-failure. Se algum write
    // intermediário falha (disk full / permission), rm os arquivos já escritos
    // pra evitar partial state em output_dir (Painter W3 Export dialog vai
    // depender disso).
    let mut written: Vec<std::path::PathBuf> = Vec::with_capacity(artifacts.len());
    for (tier, ktx2_bytes) in &artifacts {
        let filename = format!("{stem}-{}.ktx2", tier_filename(*tier));
        let out_path = output_dir.join(&filename);
        if let Err(e) = std::fs::write(&out_path, ktx2_bytes) {
            // Best-effort cleanup; if individual remove falha, ignore (já estamos
            // em erro path; original error é o que importa relatar).
            for p in &written {
                let _ = std::fs::remove_file(p);
            }
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
        let id = ph2d_asset::AssetId::from_bytes(ktx2_bytes);
        println!(
            "✓ {} (tier={tier:?}, class={asset_class:?}) → {} ({} bytes, id {id})",
            input.display(),
            out_path.display(),
            ktx2_bytes.len(),
        );
        written.push(out_path);
    }
    println!("✓ cook-all emitted {} artifacts", artifacts.len());
    Ok(())
}

fn tier_filename(tier: Tier) -> &'static str {
    match tier {
        Tier::Desktop => "desktop",
        Tier::Mobile => "mobile",
        Tier::Web => "web",
        Tier::LowEnd => "lowend",
        Tier::Constrained => "constrained",
    }
}

fn run_texture_cook(
    input: &std::path::Path,
    output: &std::path::Path,
    tier: Tier,
    asset_class: AssetClass,
) -> Result<(), Box<dyn std::error::Error>> {
    let png_bytes = std::fs::read(input)?;
    // W1.T3 γ-H2 fix: `for_asset_class` deriva `color_space` semanticamente
    // correto (Linear para NormalMap/SingleChannel, sRGB para SpriteColor/UI).
    // `..Default::default()` aplicaria sRGB universalmente → gamma bug em normal maps.
    let options = CookOptions::for_asset_class(tier, asset_class);
    let ktx2_bytes = texture::cook(&png_bytes, options)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output, &ktx2_bytes)?;
    let id = ph2d_asset::AssetId::from_bytes(&ktx2_bytes);
    println!(
        "✓ cooked texture {} (tier={tier:?}, class={asset_class:?}) → {} ({} bytes, id {id})",
        input.display(),
        output.display(),
        ktx2_bytes.len(),
    );
    Ok(())
}
