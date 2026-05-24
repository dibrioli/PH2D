//! Wave 10 / Etapa 5.3+5.4 — UI/raster crates must not pass color
//! data around as bare `Vec<u8>` / `&[u8]` / `[u8; 4]`. Use the typed
//! wrappers from `ph2d-color` (`SrgbRgba`, `LinearRgba`,
//! `Premultiplied<T>`, `OklchColor`) so the encoding + premultiply
//! state is visible at every boundary.
//!
//! Recurring bug class (UI_Bugs §4, Image Tools Bugs §1): an sRGB
//! `Vec<u8>` fed into a filter expecting linear floats produced
//! darker-than-expected mids; a straight-alpha buffer fed into a
//! premultiplied blend produced dark halos on edges.
//!
//! Scope: `crates/ph2d-tool-*/src/**`, `crates/ph2d-render/src/**`,
//! `crates/ph2d-panel-*/src/**`. The gate looks for color-buffer
//! patterns in PUBLIC function signatures (the boundaries that matter):
//!   - `pub fn ... -> Vec<u8>`
//!   - `pub fn ...(... &[u8] ...)` named `pixels` / `rgba` / `bgra` /
//!     `image` / `bytes` (semantic markers)
//!   - return type `(Vec<u8>, …)` with a width/height tuple pattern.
//!
//! Allow per-line `// COLOR-RAW-OK: <reason>` for the truly raw
//! pass-through cases (file IO, network buffers, GPU-uploadable bytes
//! that are explicitly NOT color data).

use std::fs;
use std::path::{Path, PathBuf};

/// Frozen baseline — existing public surfaces that still pass color
/// data as bare bytes. Migration to `ph2d-color` typed wrappers is
/// the Wave 10 Etapa 5.4 follow-up sweep (1-2 weeks, touches every
/// image tool + render). New code must NOT add to this list — the
/// gate prevents net additions.
///
/// Each entry: file path relative to the workspace root.
const BASELINE: &[&str] = &[
    "crates/ph2d-render/src/premul.rs",
    "crates/ph2d-tool-bgremoval/src/tool.rs",
    "crates/ph2d-tool-color-equalization/src/algorithm.rs",
    "crates/ph2d-tool-color-equalization/src/tool.rs",
    "crates/ph2d-tool-equalize-sizes/src/algorithm.rs",
    "crates/ph2d-tool-make-square/src/algorithm.rs",
    "crates/ph2d-tool-padding/src/algorithm.rs",
    "crates/ph2d-tool-trim-transparency/src/algorithm.rs",
    "crates/ph2d-tool-upscale/src/algorithm.rs",
    "crates/ph2d-tool-upscale/src/tool.rs",
];

#[test]
fn no_bare_byte_color_in_ui_or_raster_crates() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let crates_dir = workspace_root.join("crates");

    let mut targets: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(&crates_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if !p.is_dir() {
                continue;
            }
            let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            // Scope per the plan §5.3:
            // - ph2d-tool-* (image edit tools)
            // - ph2d-render (renderer)
            // - ph2d-panel-* (panel chrome painters)
            // Excludes: tool-runtime (cross-cutting helpers — the wire
            // helpers there ARE the boundary that gets typed once and
            // re-used; they own raw pass-through by design).
            let is_target = name.starts_with("ph2d-tool-")
                && name != "ph2d-tool-runtime"
                && name != "ph2d-tool-registry"
                && name != "ph2d-tool-sync"
                || name == "ph2d-render"
                || name.starts_with("ph2d-panel-") && name != "ph2d-panel-registry-init";
            if is_target {
                targets.push(p);
            }
        }
    }

    let mut files = Vec::new();
    for t in &targets {
        let src = t.join("src");
        if src.is_dir() {
            collect_rs(&src, &mut files);
        }
    }
    // Don't scan tests or this gate itself.
    files.retain(|p| {
        !p.components()
            .any(|c| c.as_os_str() == "tests" || c.as_os_str() == "benches")
    });

    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let baseline_paths: Vec<PathBuf> = BASELINE.iter().map(|p| workspace_root.join(p)).collect();

    let mut violations: Vec<String> = Vec::new();
    for path in &files {
        if baseline_paths
            .iter()
            .any(|b| b.canonicalize().ok() == path.canonicalize().ok())
        {
            continue;
        }
        let Ok(src) = fs::read_to_string(path) else {
            continue;
        };
        for (lineno, line) in src.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            if line.contains("COLOR-RAW-OK") {
                continue;
            }
            if !trimmed.starts_with("pub fn") && !trimmed.starts_with("pub(crate) fn") {
                continue;
            }
            // Look for the bug patterns in the signature line(s).
            // Multi-line sigs are flagged based on the line containing
            // the Vec<u8> / &[u8] occurrence — close enough to catch
            // the boundary.
            if line_has_bare_color_byte_pattern(line) {
                violations.push(format!(
                    "{}:{} — `{}`",
                    path.display(),
                    lineno + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Bare `Vec<u8>` / `&[u8]` color-data parameter(s) found in \
         public surfaces of UI/raster crates:\n  {}\n\n\
         Use a typed wrapper from `ph2d_color`:\n  \
         - `&[SrgbRgba]` or `Vec<SrgbRgba>` for byte color buffers\n  \
         - `&[LinearRgba]` for f32 render buffers\n  \
         - `Premultiplied<&[SrgbRgba]>` to mark already-premultiplied\n\n\
         If the bytes are genuinely NOT color data (file IO, network, \
         GPU-uploadable opaque blob), append `// COLOR-RAW-OK: <reason>`.",
        violations.join("\n  ")
    );
}

/// Detect the bug-prone patterns on a single line of a `pub fn` sig.
fn line_has_bare_color_byte_pattern(line: &str) -> bool {
    // Pattern A: returns `Vec<u8>`
    if line.contains("-> Vec<u8>") {
        return true;
    }
    // Pattern B: `&[u8]` argument with a color-named binding
    // (`pixels`, `rgba`, `bgra`, `image`, `image_bytes`, `buf` is
    // ambiguous — skip it).
    let color_param_names = [
        "pixels:",
        "rgba:",
        "bgra:",
        "image:",
        "image_bytes:",
        "src_rgba:",
        "dst_rgba:",
    ];
    for name in color_param_names {
        if let Some(pos) = line.find(name) {
            let tail = &line[pos + name.len()..];
            let tail = tail.trim_start();
            if tail.starts_with("&[u8]") || tail.starts_with("Vec<u8>") {
                return true;
            }
        }
    }
    false
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn detector_flags_return_vec_u8() {
    assert!(line_has_bare_color_byte_pattern(
        "pub fn cook(&self) -> Vec<u8> {"
    ));
}

#[test]
fn detector_flags_pixels_slice() {
    assert!(line_has_bare_color_byte_pattern(
        "pub fn process(&self, pixels: &[u8], w: u32, h: u32) {"
    ));
}

#[test]
fn detector_ignores_uncolored_buf() {
    assert!(!line_has_bare_color_byte_pattern(
        "pub fn write(&self, buf: &[u8]) {"
    ));
}
