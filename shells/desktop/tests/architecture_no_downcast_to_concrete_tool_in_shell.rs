//! Wave 10 / Etapa 3 arch-gate: scanning `shells/desktop/src/` for
//! downcasts to concrete `ph2d_tool_*` types.
//!
//! ## What this gate enforces
//!
//! ADR-0040 §2.1 + ADR-0041 establish `RasterEditTool` as the
//! generic channel for raster I/O lifecycle. Bridges that drive raster
//! tools should reach them via `as_raster_edit_mut()` (the upcast),
//! NOT via `downcast_mut::<SomeConcreteTool>()`.
//!
//! Genuine exceptions exist (documented in ADR-0040 §3) where a
//! tool-specific affordance can't fit the generic contract:
//!   - eyedropper.rs — BgR-specific colour pick UI
//!   - protect_brush.rs — BgR-specific painting input dispatch
//!   - bridges with panel-snapshot / overlay-tint / brush-ring needs
//!
//! The allowlist below freezes the current legitimate-exception set.
//! NEW bridges or NEW exceptions require either:
//!   1. Adding to the allowlist with explicit justification (Coord-A
//!      decision), OR
//!   2. Reworking to use the trait surface instead.
//!
//! The gate counts downcasts in non-allowlisted files. Adding a NEW
//! downcast to a non-allowlisted file fails. Adding a downcast inside
//! an allowlisted file is OK — those files are documented as
//! tool-specific.

use std::fs;
use std::path::{Path, PathBuf};

/// Files whose tool-concrete downcasts are documented exceptions
/// (ADR-0040 §3). Paths are relative to `shells/desktop/`.
///
/// **Adding to this list is a Coord-A decision** with explicit
/// justification in the per-tool ADR or in DIRETRIZ §3.8.3.1.
const DOWNCAST_ALLOWLIST: &[&str] = &[
    // Eyedropper: BgR-specific UI affordance (clicks canvas, samples
    // pixel under cursor, feeds add_extra_color). Genuinely BgR-only.
    "src/input_dispatch/eyedropper.rs",
    // Protect-brush: BgR-specific painting input dispatch (dabs into
    // protect_mask). Genuinely BgR-only.
    "src/input_dispatch/protect_brush.rs",
    // BgR bridge: needs concrete-type access for panel snapshot publish,
    // protect-mask tint overlay, brush-size ring. Each is a documented
    // BgR-specific affordance.
    "src/render_loop/bgremoval_preview.rs",
    // CEQ bridge: needs concrete-type access for panel snapshot publish
    // + dropdown-close drain. Other bits go through the trait + helpers.
    "src/render_loop/color_equalization_bridge.rs",
    // Upscale bridge: needs concrete-type access for panel snapshot
    // publish. Other bits go through the trait + helpers.
    "src/render_loop/upscale_bridge.rs",
    // Padding bridge: tool is geometric-only (DIRETRIZ §3.8.3.1
    // exception), no RasterEditTool impl, must downcast for spec/Apply.
    "src/render_loop/padding_bridge.rs",
    // EqualizeSizes bridge: tool is multi-sprite-required (DIRETRIZ
    // §3.8.3.1 exception), no RasterEditTool impl, must downcast.
    "src/render_loop/equalize_sizes_bridge.rs",
    // Painter bridge: PainterTool is a stroke/vector tool with NO
    // RasterEditTool impl (it doesn't bake an Individual texture via the
    // raster lifecycle), so stroke-state queries (is_stroke_active,
    // has_painted_since_source) + the Apply-path preview-cache management
    // require the concrete downcast. Same exception class as
    // padding_bridge / equalize_sizes_bridge. (Coord decision, 2026-05-29.)
    "src/render_loop/painter_bridge.rs",
    // Painter bridge-queries: `painter_has_unflushed_strokes` + `apply_layer_
    // reparent` split out of painter_bridge.rs (HR-18 LOC cap); same downcast
    // exception class as painter_bridge.rs.
    "src/render_loop/painter_bridge_queries.rs",
    // Painter fluid bridge: W15.3 GPU wet-field drive (feature `fluid`). Downcasts
    // to PainterTool for the concrete wet-field hooks (step on GPU + composite);
    // same exception class as painter_bridge.rs. (Coord decision, 2026-06-07.)
    "src/render_loop/painter_fluid_bridge.rs",
    // image_edit drain: per-tool bake dispatch. Some downcasts retire
    // in later Etapas as OneShotImageOp routes via Registry kind.
    "src/render_loop/image_edit.rs",
    // Vector inspector bridge: VectorShape/Pen/Pencil/Direct tools expose
    // shape-kind / point-type affordances the docked inspector drives, which
    // need concrete-type access — same exception class as painter_bridge.
    // (Coord ship-fix, 2026-06-04.)
    "src/render_loop/vector_inspector_bridge.rs",
    // render_loop/mod.rs: ONE VectorDirectTool downcast (set-vertex-kind from
    // the pending-point-type pill, ~L1149). It belongs in a vector bridge
    // (mirror painter_bridge::apply_layer_reparent); allowlisted as a Coord
    // ship-fix because the file carries live Vector WIP right now (can't edit
    // without collision). FOLLOW-UP (Vector impl): move it to a bridge and
    // drop this entry — the central dispatch must stay downcast-free.
    "src/render_loop/mod.rs",
    // Removed in Wave 10 / Etapa 3 audit [C1]: hero_intents/image_edit/*.rs
    // entries were pre-emptive — none of them actually downcast today.
    // The stale-check below ensures the allowlist only contains files
    // with REAL downcasts. Future bridges that need a downcast must
    // earn the entry with explicit justification.
];

fn collect_rs_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip target/ etc.
                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if name.starts_with('.') || name == "target" {
                    continue;
                }
                collect_rs_files_recursive(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
}

#[test]
fn no_downcast_to_concrete_tool_in_non_allowlisted_files() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = crate_root.join("src");
    let mut files = Vec::new();
    collect_rs_files_recursive(&src_dir, &mut files);

    let mut violations: Vec<String> = Vec::new();
    for path in &files {
        // Compute path relative to crate root for allowlist check.
        let rel = path.strip_prefix(&crate_root).unwrap_or(path);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if DOWNCAST_ALLOWLIST.iter().any(|a| *a == rel_str) {
            continue;
        }
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        for (line_no, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            // Match patterns like `downcast_mut::<ph2d_tool_…::…>` or
            // `downcast_ref::<ph2d_tool_…::…>`. Generous match: any
            // downcast targeting a `ph2d_tool_*` path.
            if line.contains("downcast_mut::<ph2d_tool_")
                || line.contains("downcast_ref::<ph2d_tool_")
            {
                violations.push(format!("{}:{}: {}", rel_str, line_no + 1, line.trim()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Downcasts to concrete ph2d_tool_* types found in files NOT on \
         the allowlist:\n{}\n\n\
         If this is a legitimate tool-specific affordance, add the file\n\
         path to DOWNCAST_ALLOWLIST in this test with explicit\n\
         justification — and that's a Coord-A decision. Otherwise, route\n\
         through RasterEditTool / ph2d-tool-runtime helpers instead.",
        violations.join("\n")
    );
}
