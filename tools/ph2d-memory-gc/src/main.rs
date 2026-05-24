#![forbid(unsafe_code)]
//! `ph2d-memory-gc` — CLI driver. Defaults to scanning the canonical
//! PH2D project memory dir under `~/.claude/projects/...` and reports
//! every link that no longer resolves under the workspace root.
//!
//! Usage:
//!
//! ```text
//! cargo run -p ph2d-memory-gc
//! cargo run -p ph2d-memory-gc -- --memory-dir /custom/path --workspace /custom/wks
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut memory_dir: Option<PathBuf> = None;
    let mut workspace: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--memory-dir" => memory_dir = args.next().map(PathBuf::from),
            "--workspace" => workspace = args.next().map(PathBuf::from),
            "--help" | "-h" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("ph2d-memory-gc: unknown argument: {other}");
                print_help();
                return ExitCode::FAILURE;
            }
        }
    }

    let workspace_root = workspace.unwrap_or_else(default_workspace_root);
    let memory_dir = memory_dir.unwrap_or_else(default_memory_dir);

    if !memory_dir.is_dir() {
        eprintln!(
            "ph2d-memory-gc: memory dir not found at {}\n\
             Pass --memory-dir <path> to override.",
            memory_dir.display()
        );
        return ExitCode::FAILURE;
    }

    let broken = ph2d_memory_gc::scan_memory_dir(&memory_dir, &workspace_root);
    if broken.is_empty() {
        println!(
            "ph2d-memory-gc: scanned {} — all references resolve.",
            memory_dir.display()
        );
        return ExitCode::SUCCESS;
    }

    println!(
        "ph2d-memory-gc: {} broken reference(s) found:",
        broken.len()
    );
    for b in &broken {
        let source_rel = b
            .source
            .strip_prefix(&memory_dir)
            .unwrap_or(&b.source)
            .display();
        println!(
            "  {source_rel}:{} — `{}` (resolved: {})",
            b.line,
            b.target_text,
            b.resolved.display()
        );
    }
    println!();
    println!("Action: update or remove the stale entries. Each broken link points");
    println!("at a file/dir that has been moved or deleted since the memory was written.");
    ExitCode::FAILURE
}

fn print_help() {
    println!(
        "ph2d-memory-gc — validate that paths in MEMORY.md still exist.\n\
         \n\
         Usage:\n\
         \x20   ph2d-memory-gc [--memory-dir <path>] [--workspace <path>]\n\
         \n\
         Defaults:\n\
         \x20   --memory-dir = ~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/\n\
         \x20   --workspace  = CARGO_MANIFEST_DIR/../..\n"
    );
}

fn default_memory_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    Path::new(&home).join(".claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory")
}

fn default_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
}
