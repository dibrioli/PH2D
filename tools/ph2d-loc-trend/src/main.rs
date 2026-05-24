#![forbid(unsafe_code)]
//! CLI driver for `ph2d-loc-trend`. Three modes:
//!
//! - `record` (default): snapshot today's LOC for every critical file
//!   and append to `metrics/loc-trend.json`. Idempotent on the same day.
//! - `check`: read the history and exit non-zero if any critical file
//!   grew > 10% in the last 30 days (use in CI).
//! - `report`: print the current history for inspection.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ph2d_loc_trend::{
    CRITICAL_FILES, MAX_GROWTH_PCT, WINDOW_DAYS, append_sample, detect_growth, read_history,
    snapshot_today, today_utc, write_history,
};

fn main() -> ExitCode {
    let mut mode = "record".to_string();
    let mut workspace: Option<PathBuf> = None;
    let mut history_path: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "record" | "check" | "report" => mode = a,
            "--workspace" => workspace = args.next().map(PathBuf::from),
            "--history" => history_path = args.next().map(PathBuf::from),
            "--help" | "-h" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("ph2d-loc-trend: unknown argument: {other}");
                print_help();
                return ExitCode::FAILURE;
            }
        }
    }

    let workspace_root = workspace.unwrap_or_else(default_workspace_root);
    let history_path =
        history_path.unwrap_or_else(|| workspace_root.join("metrics/loc-trend.json"));

    let mut history = read_history(&history_path);

    match mode.as_str() {
        "record" => {
            let sample = snapshot_today(&workspace_root, &today_utc());
            let n_files = sample.files.len();
            append_sample(&mut history, sample);
            if let Err(e) = write_history(&history_path, &history) {
                eprintln!("ph2d-loc-trend: write failed: {e}");
                return ExitCode::FAILURE;
            }
            println!(
                "ph2d-loc-trend: recorded {n_files} critical file(s) to {} (total samples: {}).",
                history_path.display(),
                history.samples.len()
            );
            ExitCode::SUCCESS
        }
        "check" => {
            let flagged = detect_growth(&history);
            if flagged.is_empty() {
                println!(
                    "ph2d-loc-trend: no critical file grew > {:.0}% in the last {WINDOW_DAYS} days. ({} critical files tracked.)",
                    MAX_GROWTH_PCT * 100.0,
                    CRITICAL_FILES.len(),
                );
                return ExitCode::SUCCESS;
            }
            println!(
                "ph2d-loc-trend: {} critical file(s) grew > {:.0}% in the last {WINDOW_DAYS} days:",
                flagged.len(),
                MAX_GROWTH_PCT * 100.0,
            );
            for (file, old, new, pct) in &flagged {
                println!("  {file}: {old} → {new} LOC ({:+.1}%)", pct * 100.0);
            }
            println!();
            println!(
                "Action: split the file into sibling modules OR record an ADR in \
                 docs/architecture/decisions/ documenting why the growth is intentional."
            );
            ExitCode::FAILURE
        }
        "report" => {
            if history.samples.is_empty() {
                println!("ph2d-loc-trend: no samples yet. Run `ph2d-loc-trend record` first.");
                return ExitCode::SUCCESS;
            }
            for s in &history.samples {
                println!("=== {} ===", s.date);
                for (file, loc) in &s.files {
                    println!("  {file}: {loc} LOC");
                }
            }
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("ph2d-loc-trend: unknown mode: {other}");
            print_help();
            ExitCode::FAILURE
        }
    }
}

fn print_help() {
    println!(
        "ph2d-loc-trend — record per-file LOC over time and flag emergent god-files.\n\
         \n\
         Usage:\n\
         \x20   ph2d-loc-trend [record|check|report] [--workspace <path>] [--history <path>]\n\
         \n\
         Modes:\n\
         \x20   record (default) — snapshot today's LOC + append to history\n\
         \x20   check            — fail if any critical file grew > {}% in {} days\n\
         \x20   report           — print the full history\n",
        (MAX_GROWTH_PCT * 100.0) as i64,
        WINDOW_DAYS,
    );
}

fn default_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .to_path_buf()
        })
}
