//! Wave 10 / Etapa 3 arch-gate: `shells/desktop/src/render_loop/mod.rs`
//! must NOT grow per-tool branches as new raster tools are added.
//!
//! ## What this gate enforces
//!
//! The render_loop should hold ONLY tool-id-agnostic dispatch (the
//! generic bus drain, the generic image_edit driver). Anything that
//! says "if tool_id == "bgremoval" ..." or branches on a concrete
//! tool name is an anti-pattern — that's the very thing
//! `ph2d-tool-runtime` helpers + `RasterEditTool` trait + Registry
//! `kind`-based resolution exist to retire.
//!
//! ## How it works
//!
//! Scan `shells/desktop/src/render_loop/mod.rs` for:
//!   - String literal `"bgremoval"`, `"color_equalization"`,
//!     `"upscale"`, `"padding"`, `"equalize_sizes"`, etc — UNLESS
//!     the line is in the allowlisted file-region comment.
//!   - Match-on-tool-id arms (`match tool_id { ... "X" ... }`).
//!
//! ## Allowlist
//!
//! Lines marked with `// ARCH-ALLOW: per-tool-branch (<reason>)` are
//! exempt. Use sparingly; reason should explain why the per-tool branch
//! cannot be retired (e.g., tool's input dispatch genuinely tool-specific
//! beyond the contract — like the BgR protect-brush gating).
//!
//! ## Future state (Etapas 4-7)
//!
//! As more bridges go through the `RasterEditTool` channel +
//! `OneShotImageOp` routes via Registry `kind` lookup, this gate
//! catches regressions where someone re-adds a tool-id branch in
//! render_loop/mod.rs.

use std::fs;
use std::path::PathBuf;

const KNOWN_TOOL_IDS: &[&str] = &[
    "bgremoval",
    "color_equalization",
    "upscale",
    "padding",
    "equalize_sizes",
    "rasterize",
    "trim_transparency",
    "make_square",
    "real_size",
];

const ALLOWLIST_MARKER: &str = "ARCH-ALLOW: per-tool-branch";

#[test]
fn render_loop_mod_has_no_per_tool_id_branch() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("render_loop")
        .join("mod.rs");
    let content = fs::read_to_string(&path).expect("render_loop/mod.rs readable");

    // Wave 10 / Etapa 3 baseline (audit fix [C2]): snapped to the EXACT
    // current count (16). The gate WILL FAIL if a new per-tool branch
    // is added, forcing the author to either:
    //   1. Generalize via Registry kind lookup / RasterEditTool channel
    //      (preferred — that's why Wave 10 exists), OR
    //   2. Mark the line with `// ARCH-ALLOW: per-tool-branch (<reason>)`
    //      with explicit justification (Coord-A decision).
    //
    // Bumping the cap UP requires explicit Coord-A decision + ADR.
    // Bumping DOWN is ENCOURAGED — every Etapa that retires a branch
    // should snap the cap to the new lower count.
    const BASELINE_MENTIONS: usize = 16;

    let mut total_mentions = 0usize;
    for (line_no, line) in content.lines().enumerate() {
        // Allowlisted lines don't count.
        if line.contains(ALLOWLIST_MARKER) {
            continue;
        }
        // Comments don't count (but doc-comments DO, because they often
        // copy the same string in examples; for now skip ALL comments to
        // avoid false-positives).
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        for tool_id in KNOWN_TOOL_IDS {
            // Match exact string literal "<tool_id>" or ToolId::new("<tool_id>").
            let quoted = format!("\"{tool_id}\"");
            if line.contains(&quoted) {
                eprintln!(
                    "[per-tool-branch] mod.rs:{}: \"{}\" → {}",
                    line_no + 1,
                    tool_id,
                    line.trim()
                );
                total_mentions += 1;
            }
        }
    }

    assert!(
        total_mentions <= BASELINE_MENTIONS,
        "render_loop/mod.rs has {total_mentions} per-tool-id string-literal mentions \
         (baseline cap: {BASELINE_MENTIONS}).\n\
         \n\
         Adding new ones is the anti-pattern Wave 10 retires (ADR-0040 §2.1):\n\
         the shell should dispatch via Registry `kind` + RasterEditTool channel,\n\
         not via `if tool_id == \"foo\"` branches. If a new branch is genuinely\n\
         tool-specific (e.g., a dispatch into a tool's own custom hook), mark\n\
         the line with `// ARCH-ALLOW: per-tool-branch (<reason>)`.\n\
         \n\
         Reducing the cap is ENCOURAGED — bump down when an etapa retires\n\
         per-tool branches. Raising the cap requires Coord-A decision + ADR."
    );
}
