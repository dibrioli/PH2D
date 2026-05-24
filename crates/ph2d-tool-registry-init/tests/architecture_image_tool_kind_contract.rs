//! Wave 10 / Etapa 3 arch-gate: tools registered in the `"image_tools"`
//! cluster MUST EITHER:
//!
//!   (a) implement `RasterEditTool` (`impl RasterEditTool for <Tool>`
//!       in their `src/tool.rs` or `src/lib.rs`), wiring through the
//!       generic shell runtime channel (ADR-0041), OR
//!
//!   (b) be explicitly allowlisted as a "documented exception"
//!       (DIRETRIZ §3.8.3.1) — for tools whose shape genuinely cannot
//!       fit the trait (geometric-only, multi-sprite-required, etc.).
//!
//! This gate enforces the canonical pattern: every NEW image-tool
//! either adopts the generic channel from day 1, or has an explicit
//! exception with justification in DIRETRIZ.
//!
//! ## How it works
//!
//! 1. Scan every `crates/ph2d-tool-*/src/` for `cluster: "image_tools"`
//!    in the manifest.
//! 2. For each such crate, check if any `.rs` file under `src/`
//!    contains `impl RasterEditTool for`.
//! 3. If neither (a) nor exemption holds, the gate fails.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Image-tool crates that legitimately do NOT implement RasterEditTool.
/// Each entry MUST have a corresponding justification in DIRETRIZ §3.8.3.1.
///
/// **Adding to this list is a Coord-A decision** with explicit
/// architectural rationale (geometric-only / multi-sprite-required /
/// one-shot stateless / etc).
const RASTER_EDIT_EXCEPTIONS: &[&str] = &[
    // Padding: geometric-only — carries 4 ints (top/right/bottom/left)
    // + flags; bake is stateless `add_padding(rgba, w, h, spec)`. No
    // source raster cached. Forcing trait stateful would be astronaut
    // architecture (DIRETRIZ §3.8.3.1 + audit etapa-2.md §3).
    "ph2d-tool-padding",
    // EqualizeSizes: multi-sprite-required — `MaxOfSelection` mode
    // depends on the global W/H of the whole selection to compute
    // target dims. Doesn't fit `set_source(rgba, w, h)` single-buffer
    // (DIRETRIZ §3.8.3.1 + audit etapa-2.md §3).
    "ph2d-tool-equalize-sizes",
    // Make Square / Real Size / Rasterize / Trim Transparency: one-shot
    // stateless tools (no panel, no preview, just a TopBar pill that
    // fires `OneShotImageOp` on the active sprite). These are sabor (1)
    // per DIRETRIZ §3.8.3 — they have no `impl Tool`, only manifest +
    // pure `algorithm` fn. RasterEditTool is for SABOR (3) stateful
    // tools with live preview; one-shots don't qualify.
    "ph2d-tool-make-square",
    "ph2d-tool-real-size",
    "ph2d-tool-rasterize",
    "ph2d-tool-trim-transparency",
];

fn workspace_root() -> PathBuf {
    // tests/ run from CARGO_MANIFEST_DIR; this is crates/ph2d-tool-registry-init.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates/
        .unwrap()
        .parent() // workspace root
        .unwrap()
        .to_path_buf()
}

fn crate_has_image_tools_cluster(crate_dir: &Path) -> bool {
    let src = crate_dir.join("src");
    for entry in fs::read_dir(&src).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        // Look for cluster declaration in a manifest-like literal.
        // Match `cluster: "image_tools"` (the canonical ToolManifest field).
        // Skip lines that are inside `#[cfg(test)]` mod tests (just a
        // heuristic — comments are also skipped by the trim_start check).
        for line in content.lines() {
            let t = line.trim_start();
            if t.starts_with("//") {
                continue;
            }
            if t.contains("cluster: \"image_tools\"") {
                return true;
            }
        }
    }
    false
}

fn crate_implements_raster_edit_tool(crate_dir: &Path) -> bool {
    let src = crate_dir.join("src");
    for entry in fs::read_dir(&src).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        for line in content.lines() {
            let t = line.trim_start();
            if t.starts_with("//") {
                continue;
            }
            if t.contains("impl RasterEditTool for") {
                return true;
            }
        }
    }
    false
}

#[test]
fn image_tools_either_implement_raster_edit_tool_or_are_exempt() {
    let crates_dir = workspace_root().join("crates");
    let mut image_tool_crates: BTreeSet<String> = BTreeSet::new();

    for entry in fs::read_dir(&crates_dir)
        .expect("crates/ dir readable")
        .flatten()
    {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = match path.file_name().and_then(|s| s.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        if !name.starts_with("ph2d-tool-") {
            continue;
        }
        // Exclusion list — contract/infra crates that match the
        // `ph2d-tool-*` prefix but aren't concrete tools (same set as
        // `tools/ph2d-tool-sync`'s exclusion list; mention of
        // `cluster: "image_tools"` in these crates is from test fixtures
        // or doc examples, not a real registration).
        if matches!(
            name.as_str(),
            "ph2d-tool-registry" | "ph2d-tool-registry-init" | "ph2d-tool-runtime"
        ) {
            continue;
        }
        if crate_has_image_tools_cluster(&path) {
            image_tool_crates.insert(name);
        }
    }

    let mut violations: Vec<String> = Vec::new();
    for crate_name in &image_tool_crates {
        let crate_dir = crates_dir.join(crate_name);
        let implements = crate_implements_raster_edit_tool(&crate_dir);
        let is_exempt = RASTER_EDIT_EXCEPTIONS.contains(&crate_name.as_str());
        if !implements && !is_exempt {
            violations.push(format!(
                "  - {crate_name}: cluster=image_tools but no `impl RasterEditTool for` \
                 and NOT on RASTER_EDIT_EXCEPTIONS allowlist"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "Image-tool crates that violate the kind contract:\n{}\n\n\
         Either:\n\
           (a) implement RasterEditTool in src/tool.rs (preferred — joins\n\
               the generic shell-runtime channel per ADR-0041), OR\n\
           (b) add the crate name to RASTER_EDIT_EXCEPTIONS in this test\n\
               file WITH justification in DIRETRIZ §3.8.3.1.\n\
         \n\
         Don't quietly add a new image-tool that just downcasts in the\n\
         shell — that's exactly what Wave 10 retires.",
        violations.join("\n")
    );

    // Sanity: ensure the allowlist doesn't have stale entries (tools
    // that USED to be exempt but now implement RasterEditTool should
    // be removed from the allowlist to keep the gate honest).
    for exempt in RASTER_EDIT_EXCEPTIONS {
        let crate_dir = crates_dir.join(exempt);
        if !crate_dir.is_dir() {
            // Stale entry — tool was deleted/renamed.
            panic!(
                "RASTER_EDIT_EXCEPTIONS contains stale entry `{exempt}` \
                 (no such crate under crates/). Remove it."
            );
        }
        if crate_implements_raster_edit_tool(&crate_dir) {
            panic!(
                "RASTER_EDIT_EXCEPTIONS contains `{exempt}` but the crate \
                 NOW implements RasterEditTool. Remove the entry — the \
                 exception is no longer needed."
            );
        }
    }
}
