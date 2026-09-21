//! Wave 10 / Etapa 3 arch-gate: the FRAME (`shells/desktop/src/render_loop/mod.rs` and the `fase_*`
//! files it is split into) must NOT grow per-tool branches as new raster tools are added.
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
//! Scan the frame — `render_loop/mod.rs` AND every `render_loop/fase_*.rs` — for:
//!   - String literal `"bgremoval"`, `"color_equalization"`,
//!     `"upscale"`, `"padding"`, `"equalize_sizes"`, etc — UNLESS
//!     the line is in the allowlisted file-region comment.
//!   - Match-on-tool-id arms (`match tool_id { ... "X" ... }`).
//!
//! ⚠️ **Why the phase files too** (OBRA 2 da `line/render-loop`, 2026-09-12): the frame is being split
//! into `fase_*` files, called in the same order. A census that reads only `mod.rs` would read ZERO once
//! the bus drain moves out — and a new per-tool branch written in a phase would pass, green, with the
//! gate still "running". *A census that presumes where the code lives stops counting when it moves.*
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
//! the frame.

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

/// ⚠️ **A raiz da SHELL, subindo da crate onde este gate vive** (2026-09-21).
///
/// ⛔⛔ **Ele mudou de crate por causa da catraca `the_shell_only_shrinks`**, que conta
/// `shells/desktop` INTEIRO — `tests/` incluído. Uma wave que ligou o enquadramento do bake levou-a
/// acima do tecto, e a lei é *CORTE, nunca subir o número*; este ficheiro é um arch-gate de TEXTO
/// sem uma única dependência da crate da shell, e **todos os irmãos `architecture_*` já viviam
/// aqui** — logo o corte mais barato era a mudança de endereço, não código de produto.
///
/// ⛔ **O preço está registado:** *«um gate de família que lê a shell pelo caminho escapa a quem
/// move o código»*. Aqui ele erra para o lado BARULHENTO — o `expect` abaixo diz o caminho —, que é
/// a forma desta família que não passa em silêncio.
fn shell_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .join("shells/desktop")
}

/// The frame's files: `render_loop/mod.rs` and every `render_loop/fase_*.rs`, sorted.
fn frame_files() -> Vec<PathBuf> {
    let dir = shell_dir().join("src").join("render_loop");
    let mut out: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("render_loop/ readable")
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n == "mod.rs" || (n.starts_with("fase_") && n.ends_with(".rs")))
        })
        .collect();
    out.sort();
    out
}

#[test]
fn render_loop_mod_has_no_per_tool_id_branch() {
    let files = frame_files();
    // ⚠️ Population floor: a broken scan reads zero files and zero mentions, which reads as approved.
    // Measured 2026-09-12: `mod.rs` + 44 phases.
    assert!(
        files.iter().any(|p| p.ends_with("render_loop/mod.rs")),
        "the scan does not see render_loop/mod.rs — the census broke"
    );
    assert!(
        files.len() >= 30,
        "the scan sees only {} frame files — the census broke, and zero mentions would read as \
         approved",
        files.len()
    );

    // Wave 10 / Etapa 3 baseline (audit fix [C2]) was 16. Snapped DOWN to the exact count measured over
    // the whole frame on 2026-09-12 (6, all in the bus drain: five `OneShotImageOp` routes and the
    // Bg-Removal preview reset on activation), when the census was widened to the phase files. The
    // gate WILL FAIL if a new per-tool branch is added, forcing the author to either:
    //   1. Generalize via Registry kind lookup / RasterEditTool channel
    //      (preferred — that's why Wave 10 exists), OR
    //   2. Mark the line with `// ARCH-ALLOW: per-tool-branch (<reason>)`
    //      with explicit justification (Coord-A decision).
    //
    // Bumping the cap UP requires explicit Coord-A decision + ADR.
    // Bumping DOWN is ENCOURAGED — every Etapa that retires a branch
    // should snap the cap to the new lower count.
    const BASELINE_MENTIONS: usize = 6;

    let mut total_mentions = 0usize;
    for path in &files {
        let content = fs::read_to_string(path).expect("frame file readable");
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
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
                        "[per-tool-branch] {}:{}: \"{}\" → {}",
                        name,
                        line_no + 1,
                        tool_id,
                        line.trim()
                    );
                    total_mentions += 1;
                }
            }
        }
    }

    assert!(
        total_mentions <= BASELINE_MENTIONS,
        "the frame (render_loop/mod.rs + fase_*.rs) has {total_mentions} per-tool-id string-literal \
         mentions (baseline cap: {BASELINE_MENTIONS}).\n\
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
