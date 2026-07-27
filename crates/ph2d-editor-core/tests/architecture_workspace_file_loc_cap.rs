//! Architecture gate (blindagem Fase 0.4) — workspace-wide **file** LOC cap.
//!
//! ## Why this exists
//!
//! Before this gate the 600-LOC discipline covered <15% of `crates/`: only
//! `ph2d-panel-*` (`architecture_panel_loc_cap`), `editor-core/src/widget/**`
//! (`architecture_widget_loc_cap`), `ph2d-tool-runtime` and `shells/desktop`.
//! The foundational crates where the real god-files live — `ph2d-render`
//! (compositor 2396 LOC), `ph2d-painter-effects` (compute 1636), `editor-core`
//! pointer dispatch (1261), every `ph2d-tool-*` — had **no** size discipline.
//! This gate closes that gap: every production `crates/*/src/**` file is
//! capped at 700 LOC (raised from 600 by [ADR-0105] — the raw-line metric
//! misfires on cohesive data-heavy files like a 60-field struct + its
//! `Default`, where splitting the definition fragments one responsibility;
//! 700 still flags the genuine god-files as split candidates). Existing
//! offenders above 700 are FROZEN at their 2026-06-20 baseline (the allowlist
//! below) — they may shrink but never grow, and any NEW file is born under 700.
//! Decomposing the frozen entries is Fase 3.
//!
//! Function-level caps are deliberately NOT enforced here: the brace-walk
//! parser the panel cap uses miscounts on apostrophes inside `//` comments
//! (see `architecture_panel_loc_cap.rs:118`). The fn cap lands workspace-wide
//! in Fase 3.1 once that parser is comment-aware.
//!
//! ## What is excluded (covered elsewhere, or not production)
//!
//! - `**/tests/**` and `**/tests.rs` — test code (other crates' src-inline
//!   test modules are allowed to be long, same policy as the panel cap).
//! - `ph2d-panel-*` — `architecture_panel_loc_cap` owns these.
//! - `ph2d-editor-core/src/widget/**` — `architecture_widget_loc_cap` (500).
//! - `ph2d-tool-runtime/src/**` — `architecture_runtime_loc_cap` (650).
//! - `*registry-init*` — codegen targets, not hand-written surfaces.
//!
//! Dep-free (std only), mirroring the other architecture gates.

use std::fs;
use std::path::{Path, PathBuf};

const FILE_LOC_CAP: usize = 700;

/// Per-file frozen ceiling — pre-existing >600 files at the 2026-06-20
/// blindagem Fase 0.4 baseline. Each is exempt up to the recorded LOC (may
/// shrink, never grow); driving every entry to ≤600 by splitting is Fase 3.2.
/// A NEW entry, or raising a number, requires Coordenador sign-off + an ADR
/// note. Keys are relative to `crates/`.
const FILE_OVERAGE_OK: &[(&str, usize)] = &[
    // 784 → 768 (ADR-0131 W5): the ancestor walk + the inverse of `compose`
    // moved to the sibling `transform_inverse.rs`. Ratcheted DOWN on the way
    // past, which is the only direction this table is supposed to move.
    ("ph2d-ecs/src/transform.rs", 768),
    ("ph2d-editor-core/src/grid_snap/state.rs", 796),
    // Aposentada 2026-07-27 (W-FK): `paint.rs` tinha 884 LOC de primitivas de
    // GEOMETRIA + a família do TEXTO inteira; a segunda saiu para o irmão
    // `paint_text.rs` (com re-export, então nenhum chamador mudou de endereço) e
    // o arquivo caiu para 685, sob o teto simples de 700. Entrada DELETADA em vez
    // de baixada — é a única direção que esta tabela anda.
    ("ph2d-imageio-apng/src/lib.rs", 768),
    ("ph2d-imageio-ph2d-native/src/schema.rs", 746),
    ("ph2d-imageio-tiff/src/lib.rs", 905),
    // Retired 2026-07-09 (M2.N1): `cook.rs` was 864 LOC of engine + inline
    // tests; the tests moved to `cook_tests.rs` + `cook_scope_tests.rs`, so the
    // engine now sits at ~459 LOC under the plain 700 cap. Entry deleted rather
    // than raised, per this gate's own instruction.
    ("ph2d-painter-effects/src/adjustments/mod.rs", 946),
    ("ph2d-painter-effects/src/adjustments/spatial.rs", 856),
    ("ph2d-render/src/compressed_pipeline.rs", 993),
    ("ph2d-render/src/individual.rs", 969),
    ("ph2d-render/src/layer_compositor/mod.rs", 934),
    ("ph2d-render/src/renderer.rs", 1000),
    ("ph2d-tool-bgremoval/src/algorithm/chroma/mod.rs", 704),
    ("ph2d-tool-bgremoval/src/algorithm/compose.rs", 931),
    ("ph2d-tool-color-equalization/src/gpu/auto_wb.rs", 748),
    ("ph2d-tool-color-equalization/src/gpu/tonal_batch.rs", 744),
    ("ph2d-tool-color-equalization/src/params.rs", 888),
    ("ph2d-tool-equalize-sizes/src/algorithm.rs", 755),
    ("ph2d-tool-rasterize/src/algorithm.rs", 734),
    ("ph2d-vector/src/vector_network.rs", 780),
];

fn crates_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/ph2d-editor-core; parent = crates/.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/ dir")
        .to_path_buf()
}

/// True if `rel` (path relative to `crates/`, forward-slashed) is owned by a
/// different cap or is non-production (see module docs).
fn is_excluded(rel: &str) -> bool {
    rel.contains("/tests/")
        || rel.ends_with("/tests.rs")
        || rel.starts_with("ph2d-panel-")
        || rel.starts_with("ph2d-editor-core/src/widget/")
        || rel.starts_with("ph2d-tool-runtime/src/")
        || rel.contains("registry-init")
}

fn visit(dir: &Path, cb: &mut dyn FnMut(&Path)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit(&path, cb);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            cb(&path);
        }
    }
}

#[test]
fn workspace_src_files_under_loc_cap() {
    let root = crates_root();
    let mut offenders: Vec<(String, usize, usize)> = Vec::new(); // (rel, loc, allowed)

    let Ok(crate_dirs) = fs::read_dir(&root) else {
        panic!("cannot read crates/ at {}", root.display());
    };
    for entry in crate_dirs.flatten() {
        let crate_dir = entry.path();
        let src = crate_dir.join("src");
        if !src.is_dir() {
            continue;
        }
        visit(&src, &mut |path| {
            let rel = path
                .strip_prefix(&root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| path.display().to_string());
            if is_excluded(&rel) {
                return;
            }
            let loc = fs::read_to_string(path)
                .map(|b| b.lines().count())
                .unwrap_or(0);
            if loc <= FILE_LOC_CAP {
                return;
            }
            let allowed = FILE_OVERAGE_OK
                .iter()
                .find(|(k, _)| *k == rel)
                .map(|(_, n)| *n)
                .unwrap_or(FILE_LOC_CAP);
            if loc > allowed {
                offenders.push((rel, loc, allowed));
            }
        });
    }

    offenders.sort_by_key(|(_, loc, _)| std::cmp::Reverse(*loc));
    assert!(
        offenders.is_empty(),
        "workspace src files over their LOC cap ({} default):\n  {}\n\n\
         fix: split the file into sibling modules by responsibility (a 2396-LOC \
         compositor is pipeline-setup + N effect kernels + buffer marshalling — \
         each its own module). If the growth is a legitimate frozen baseline, \
         raise its entry in FILE_OVERAGE_OK with Coordenador sign-off — but the \
         goal is to drive entries DOWN, never up (blindagem Fase 3.2).",
        FILE_LOC_CAP,
        offenders
            .iter()
            .map(|(p, loc, allowed)| format!("{p} ({loc} LOC, cap {allowed})"))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// Guard the allowlist itself: an entry whose file dropped to ≤600 (or was
/// deleted) is stale and must be removed, so the ratchet keeps tightening.
#[test]
fn overage_allowlist_has_no_stale_entries() {
    let root = crates_root();
    let mut stale: Vec<String> = Vec::new();
    for (rel, frozen) in FILE_OVERAGE_OK {
        let path = root.join(rel);
        let loc = fs::read_to_string(&path)
            .map(|b| b.lines().count())
            .unwrap_or(0);
        if loc <= FILE_LOC_CAP {
            stale.push(format!(
                "{rel} (frozen {frozen}, now {loc} ≤ {FILE_LOC_CAP}) — remove the entry"
            ));
        }
    }
    assert!(
        stale.is_empty(),
        "stale FILE_OVERAGE_OK entries (file now under cap — delete to keep the ratchet honest):\n  {}",
        stale.join("\n  ")
    );
}
