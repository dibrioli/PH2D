//! Wave 9 Eixo B.3 — cap individual widget primitives at 500 LOC.
//!
//! HR-18 already caps shell files at 600 LOC. Widget primitives are
//! held to a tighter 500 LOC bar because each one is meant to be
//! navigable end-to-end by a fresh agent in a single read pass: data
//! struct + state enum + `paint_X` helper + AccessKit node + a
//! handful of helpers. A widget that grows past 500 is a candidate
//! for splitting into a sub-module (`<slug>/mod.rs` + `<slug>/<part>.rs`).
//!
//! Sub-files inside a widget folder (e.g. `blender_color_picker/
//! channels.rs`) are NOT exempt from the cap — split further if needed.

use std::path::{Path, PathBuf};

const WIDGET_LOC_CAP: usize = 500;

/// Per-file overage allowance — **VAZIA, e é assim que ela fica**.
///
/// Ela carregou `src/widget/panel_chrome.rs` desde a Wave 11 (640 → 654),
/// com a própria justificativa a prometer *"sub-folder split is a follow-up"*.
/// O follow-up aconteceu: a família do selector segmentado saiu para o irmão
/// `panel_chrome/segmented.rs` (o precedente do `command_palette.rs` +
/// `command_palette/layout.rs`), e as duas metades ficaram sob o teto — o pai
/// responde *que forma tem um painel*, o filho *como um grupo de opções se
/// desenha*.
///
/// ⚠️ **Uma entrada aqui é uma dívida, nunca um ajuste de barra:** o gate
/// existe para pedir o corte, e uma allowance que sobrevive ao corte permite
/// o arquivo re-crescer até ela em silêncio. Se um arquivo passar de 500,
/// **corte por responsabilidade primeiro** e só escreva aqui se o corte
/// honesto não existir — com o motivo, não com o número.
const FILE_OVERAGE_OK: &[(&str, usize, &str)] = &[];

fn widget_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/widget")
}

#[test]
fn widget_primitives_under_loc_cap() {
    let mut offenders: Vec<(String, usize)> = Vec::new();
    visit(&widget_dir(), &mut offenders);
    offenders.retain(|(path, loc)| {
        let allow = FILE_OVERAGE_OK.iter().find(|(p, _, _)| path.ends_with(p));
        match allow {
            Some((_, cap, _)) => *loc > *cap,
            None => true,
        }
    });
    offenders.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    assert!(
        offenders.is_empty(),
        "widget primitives over {WIDGET_LOC_CAP}-LOC cap:\n  {}\n\
         fix: split the widget into sub-files (`<slug>/mod.rs` + helpers) \
         OR add an entry to FILE_OVERAGE_OK with justification.",
        offenders
            .iter()
            .map(|(p, n)| format!("{p} ({n} LOC)"))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

fn visit(dir: &Path, offenders: &mut Vec<(String, usize)>) {
    for entry in std::fs::read_dir(dir).expect("dir readable") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        // Skip the showcase tree — it's a canvas of demo sections, not
        // a primitive, and its files are intentionally larger than the
        // widget cap (the section painters compose multiple widgets).
        if path.is_dir() {
            if path.file_name().and_then(|s| s.to_str()) == Some("showcase") {
                continue;
            }
            visit(&path, offenders);
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let body = std::fs::read_to_string(&path).expect("widget file readable");
        let loc = body.lines().count();
        if loc > WIDGET_LOC_CAP {
            let rel = path
                .strip_prefix(widget_dir().parent().unwrap().parent().unwrap())
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            offenders.push((rel, loc));
        }
    }
}
