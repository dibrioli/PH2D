#![forbid(unsafe_code)]
//! `ph2d-bindgen` — generates the Luau type definitions and the MCP JSON Schema.
//!
//! Two outputs, two sources:
//! - `runtime/luau/ph2d.d.luau` — Luau definitions for luau-lsp, from
//!   `ph2d_script::api_table` (the API the script VM really has; ADR-0188 §3).
//! - `runtime/mcp/schema.json` — MCP tool catalog in the JSON Schema
//!   format MCP clients consume, from [`ph2d_mcp::CATALOG`] (HR-10).
//!
//! ### Modes
//!
//! - `ph2d-bindgen --write` — regenerate and write to disk. Run this
//!   locally after editing `crates/ph2d-script/src/api_table.rs` or
//!   `crates/ph2d-mcp/src/catalog.rs`.
//! - `ph2d-bindgen --check` — regenerate in memory, diff against the
//!   committed files, exit non-zero on drift. CI uses this to enforce
//!   parity (HR-10).
//! - `ph2d-bindgen` (no args) — same as `--check`.

use ph2d_mcp::CATALOG;
use std::path::{Path, PathBuf};

fn main() -> std::process::ExitCode {
    let mode = parse_mode();
    let root = workspace_root();
    let luau_path = root.join("runtime/luau/ph2d.d.luau");
    let schema_path = root.join("runtime/mcp/schema.json");

    let luau = generate_luau();
    let schema = generate_schema();

    match mode {
        Mode::Write => {
            ensure_parent(&luau_path);
            ensure_parent(&schema_path);
            std::fs::write(&luau_path, &luau).expect("write luau");
            std::fs::write(&schema_path, &schema).expect("write schema");
            println!("✓ wrote {}", luau_path.display());
            println!("✓ wrote {}", schema_path.display());
            std::process::ExitCode::SUCCESS
        }
        Mode::Check => {
            let mut drift = false;
            if drift_detected("luau", &luau_path, &luau) {
                drift = true;
            }
            if drift_detected("schema", &schema_path, &schema) {
                drift = true;
            }
            if drift {
                eprintln!(
                    "ph2d-bindgen --check: drift detected. Run `cargo run -p ph2d-bindgen -- --write` and commit."
                );
                std::process::ExitCode::FAILURE
            } else {
                println!("✓ generated artifacts match committed files (HR-10 parity OK)");
                std::process::ExitCode::SUCCESS
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Mode {
    Check,
    Write,
}

fn parse_mode() -> Mode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--write") {
        Mode::Write
    } else {
        Mode::Check
    }
}

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points at tools/ph2d-bindgen — go up two
    // levels to reach the workspace root regardless of where `cargo
    // run` was invoked from.
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    here.ancestors()
        .nth(2)
        .expect("workspace root above tools/ph2d-bindgen")
        .to_path_buf()
}

fn ensure_parent(p: &Path) {
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).expect("create parent dir");
    }
}

fn drift_detected(label: &str, path: &Path, fresh: &str) -> bool {
    match std::fs::read_to_string(path) {
        Ok(committed) => {
            // Normalize line endings before comparison: Windows git
            // checkouts may rewrite LF → CRLF (the default without an
            // `.gitattributes` directive), while bindgen emits raw LF.
            // The bytes-on-disk diverge from what bindgen would write
            // even though the semantic content is identical; HR-10
            // parity cares about content, not byte-level equality.
            let committed_norm = committed.replace("\r\n", "\n");
            let fresh_norm = fresh.replace("\r\n", "\n");
            if committed_norm == fresh_norm {
                false
            } else {
                eprintln!(
                    "✗ {label} drift: committed != generated\n  committed: {}",
                    path.display()
                );
                true
            }
        }
        Err(_) => {
            eprintln!("✗ {label} missing: {} (run --write)", path.display());
            true
        }
    }
}

/// The Luau half comes from the API table of `ph2d-script` (ADR-0188 §3), which a test checks
/// against the real VM wiring; the catalog no longer feeds it.
fn generate_luau() -> String {
    ph2d_script::api_table::luau_definitions()
}

fn generate_schema() -> String {
    let tools: Vec<serde_json::Value> = CATALOG
        .iter()
        .map(|spec| {
            let input: serde_json::Value =
                serde_json::from_str(spec.input_schema_json).expect("input_schema valid");
            let output: serde_json::Value =
                serde_json::from_str(spec.output_schema_json).expect("output_schema valid");
            let mut obj = serde_json::json!({
                "name": spec.name,
                "description": spec.description,
                "input_schema": input,
                "output_schema": output,
            });
            if spec.destructive {
                obj["destructive"] = serde_json::Value::Bool(true);
            }
            obj
        })
        .collect();
    let doc = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$generator": "tools/ph2d-bindgen",
        "$source_of_truth": "crates/ph2d-mcp/src/catalog.rs",
        "tools": tools,
    });
    let mut out = serde_json::to_string_pretty(&doc).expect("serialize schema");
    out.push('\n');
    out
}
