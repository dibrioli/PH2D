//! Wave 8 Phase 1.E — architecture invariants for the panel
//! ecosystem dependency graph.
//!
//! The Wave-7 audit's P3 finding flagged that nothing guarded
//! against a future regression where:
//!
//! 1. `ph2d-editor-core` accidentally gains a dep on a `ph2d-panel-*`
//!    crate (or on `ph2d-editor`), inverting the foundation layer.
//! 2. A panel crate keeps depending on `ph2d-editor` post Wave 8
//!    Phase 2 (Stage 4 migration), defeating the panel-as-crate
//!    isolation promise.
//!
//! This test reads each crate's `Cargo.toml` and asserts the
//! invariants. Dep-free — parses TOML as text since `serde` /
//! `toml` aren't workspace dev-deps and the `[dependencies]` shape
//! we care about is single-line-per-entry.
//!
//! See `docs/architecture/decisions/0028-wave-2-codegen-design-canonical.md`
//! §Wave 8 for the rationale + the two-phase rollout of these
//! invariants.

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points at this crate (`crates/ph2d-editor-core`,
    // post-ADR-0029 Phase B.2 move); workspace root is two parents up.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

/// Parse the `[dependencies]` block of a Cargo.toml and return the
/// dep names (left of `=`). Strips comments, blank lines, and
/// other section headers.
fn parse_deps(toml_path: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(toml_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", toml_path.display()));
    let mut out = Vec::new();
    let mut in_deps = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            in_deps = line == "[dependencies]";
            continue;
        }
        if !in_deps {
            continue;
        }
        if let Some(eq) = line.find('=') {
            let name = line[..eq].trim();
            if !name.is_empty() {
                out.push(name.to_string());
            }
        }
    }
    out
}

/// All `ph2d-panel-*` crate paths under `crates/`, discovered by walking
/// `crates/` for directories whose name starts with the prefix. Skips
/// `ph2d-panel-registry-init` — that crate is the codegen aggregator
/// whose entire job is to depend on every panel sibling at once
/// (mirror of `ph2d-tool-registry-init`); the cross-panel-dep ban does
/// not apply to it. Self-extending: a new panel crate joins the gate
/// the moment it lands on disk, with no edit here required (ADR-0040
/// TG-E follow-up — the hardcoded list silently missed
/// `ph2d-panel-bgremoval` and `ph2d-panel-padding` after TG-B/TG-C
/// added their `ph2d-tool-*` deps).
fn panel_crate_tomls() -> Vec<PathBuf> {
    let crates_dir = workspace_root().join("crates");
    let mut out: Vec<PathBuf> = std::fs::read_dir(&crates_dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", crates_dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("ph2d-panel-") && n != "ph2d-panel-registry-init")
        })
        .map(|p| p.join("Cargo.toml"))
        .filter(|p| p.exists())
        .collect();
    out.sort();
    out
}

#[test]
fn editor_core_has_no_panel_or_editor_deps() {
    let toml = workspace_root()
        .join("crates")
        .join("ph2d-editor-core")
        .join("Cargo.toml");
    let deps = parse_deps(&toml);
    for dep in &deps {
        assert!(
            !dep.starts_with("ph2d-panel-"),
            "INVARIANT VIOLATED — `ph2d-editor-core` depends on `{dep}`. \
             editor-core is the foundation layer; panel crates must \
             depend on it, not the other way around. Move the shared \
             code DOWN into editor-core (or another shared crate) \
             instead of pulling a panel UP."
        );
        assert_ne!(
            dep, "ph2d-editor",
            "INVARIANT VIOLATED — `ph2d-editor-core` depends on `ph2d-editor`. \
             That's a cycle: editor-core is the foundation, ph2d-editor \
             builds on it."
        );
    }
}

/// ADR-0040 T1.5 — editor-core must NOT depend on any concrete tool
/// crate (`ph2d-tool-<slug>`). Tools are satellite crates that depend
/// on editor-core (the `Tool`/`RasterEditTool` contract — renamed from
/// `ImageEditTool` in ADR-0041), never the reverse — the same
/// foundation-layer invariant the panel crates already obey.
/// `ph2d-tool-registry` (the data-contract leaf:
/// `ToolManifest`/`Registry`/`Zone`) is exempt — it's the contract,
/// not a tool, analogous to `ph2d-tokens`.
///
/// Note: this checks `[dependencies]` only (via `parse_deps`); a
/// `[dev-dependencies]` edge to a tool crate (e.g. the make_square
/// round-trip test) is allowed — it builds no runtime cycle.
#[test]
fn editor_core_has_no_concrete_tool_deps() {
    let toml = workspace_root()
        .join("crates")
        .join("ph2d-editor-core")
        .join("Cargo.toml");
    let deps = parse_deps(&toml);
    for dep in &deps {
        if dep == "ph2d-tool-registry" {
            continue; // the data contract, not a tool — allowed.
        }
        assert!(
            !dep.starts_with("ph2d-tool-"),
            "INVARIANT VIOLATED — `ph2d-editor-core` depends on `{dep}`. \
             editor-core is the foundation; tool crates depend on it via \
             the `Tool`/`RasterEditTool` contract, never the reverse \
             (ADR-0040). Move the code into the tool crate, or — if a \
             test needs the tool's algorithm — make it a [dev-dependency]."
        );
    }
}

/// Wave 8 Phase 2 invariant — every `ph2d-panel-*` crate depends on
/// `ph2d-editor-core` but NOT on `ph2d-editor` (the legacy shim).
/// ADR-0040 TG-B/TG-C added a second allowed direction: a panel that
/// paints a specific tool's snapshot MAY depend on that tool's crate
/// (`ph2d-tool-*`) — the foundation `Tool` contract makes the edge
/// non-cyclic (tool → editor-core → ..., panel → editor-core AND
/// optionally panel → tool). What panels MUST NOT do: depend on
/// `ph2d-editor` (would cycle the foundation) or on another
/// `ph2d-panel-*` (would entangle panel siblings).
///
/// ADR-0029 Phase D lifted the `#[ignore]` — every in-tree panel
/// lives as a typed `Panel<State>` impl in its own crate with
/// `ph2d-editor-core` as its foundation dep.
#[test]
fn panel_crates_depend_only_on_editor_core() {
    let mut violations = Vec::new();
    for toml in panel_crate_tomls() {
        let panel = toml
            .parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| toml.display().to_string());
        let deps = parse_deps(&toml);
        let has_core = deps.iter().any(|d| d == "ph2d-editor-core");
        let has_editor = deps.iter().any(|d| d == "ph2d-editor");
        if !has_core {
            violations.push(format!(
                "{panel} must declare `ph2d-editor-core` in [dependencies]"
            ));
        }
        if has_editor {
            violations.push(format!(
                "{panel} depends on `ph2d-editor` — Stage 4 invariant requires \
                 panel crates to consume `ph2d-editor-core` only"
            ));
        }
        // ADR-0040 TG-E: panel ↛ panel — siblings stay isolated. A
        // panel-tool edge (`ph2d-tool-*`) is allowed (see docstring).
        for dep in &deps {
            if dep.starts_with("ph2d-panel-") && dep.as_str() != panel.as_str() {
                violations.push(format!(
                    "{panel} depends on `{dep}` — panel crates must not cross-depend; \
                     share code through `ph2d-editor-core` (or a leaf widget crate) \
                     instead"
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "Panel-crate dependency invariant violated:\n  - {}",
        violations.join("\n  - ")
    );
}

#[test]
fn parse_deps_smoke() {
    // Self-check: editor-core has the deps we expect (sanity that
    // `parse_deps` is reading the file, not silently returning
    // empty due to a TOML shape we didn't anticipate).
    let toml = workspace_root()
        .join("crates")
        .join("ph2d-editor-core")
        .join("Cargo.toml");
    let deps = parse_deps(&toml);
    assert!(
        deps.iter().any(|d| d == "ph2d-tokens"),
        "expected `ph2d-tokens` in editor-core deps, got: {deps:?}"
    );
}
