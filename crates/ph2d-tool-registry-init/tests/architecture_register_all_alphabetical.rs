//! Wave 9 Eixo A.2 — gate alphabetical order of tool registrations.
//!
//! The `register_all()` body and the `[dependencies]` block in this
//! crate's `Cargo.toml` are the two shared edit points that every new
//! tool touches. Forcing alphabetical order means concurrent inserts
//! from parallel agents land in distinct positions of the file with
//! deterministic merge resolution: `git merge` produces a conflict if
//! and only if the SAME letter is inserted twice, which is fast to
//! resolve by hand. Random-order inserts produce semantic conflicts
//! the merge driver can't even diagnose.
//!
//! This test runs on both surfaces. Failures point at the line range
//! to re-sort.

use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn register_all_body_is_alphabetical() {
    let src = std::fs::read_to_string(crate_root().join("src/lib.rs"))
        .expect("src/lib.rs must be readable");
    let tools: Vec<&str> = src
        .lines()
        .map(str::trim)
        .filter_map(|l| {
            // Match `ph2d_tool_<slug>::register(reg);` only (skip
            // commented placeholder lines starting with `//`).
            let stripped = l.strip_prefix("ph2d_tool_")?;
            let slug_end = stripped.find("::register(reg);")?;
            Some(&stripped[..slug_end])
        })
        .collect();
    assert!(
        !tools.is_empty(),
        "no `ph2d_tool_*::register(reg);` lines found in src/lib.rs — \
         parser may be stale relative to register_all() shape"
    );
    let mut sorted = tools.clone();
    sorted.sort_unstable();
    assert_eq!(
        tools, sorted,
        "register_all() tool calls must be in alphabetical order. \
         actual={tools:?} sorted={sorted:?}"
    );
}

#[test]
fn cargo_toml_dependencies_are_alphabetical() {
    let toml = std::fs::read_to_string(crate_root().join("Cargo.toml"))
        .expect("Cargo.toml must be readable");
    let mut in_deps = false;
    let mut tool_deps: Vec<String> = Vec::new();
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_deps = trimmed == "[dependencies]";
            continue;
        }
        if !in_deps {
            continue;
        }
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        // Match `ph2d-tool-<slug> = ...` only. `ph2d-tool-registry`
        // is the runtime types crate (not a tool) and is exempt.
        if let Some(rest) = trimmed.strip_prefix("ph2d-tool-")
            && !rest.starts_with("registry ")
            && let Some(eq) = rest.find(' ')
        {
            tool_deps.push(rest[..eq].to_string());
        }
    }
    assert!(
        !tool_deps.is_empty(),
        "no `ph2d-tool-*` deps found under [dependencies] — parser may \
         be stale relative to Cargo.toml shape"
    );
    let mut sorted = tool_deps.clone();
    sorted.sort_unstable();
    assert_eq!(
        tool_deps, sorted,
        "ph2d-tool-* dependencies in Cargo.toml must be alphabetical. \
         actual={tool_deps:?} sorted={sorted:?}"
    );
}
