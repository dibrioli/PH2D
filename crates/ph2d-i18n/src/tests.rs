//! Os testes da tabela da `lib.rs` — irmão por tecto de LOC.

use super::*;

#[test]
fn known_keys_round_trip_to_english() {
    // Image-tool labels were abbreviated 2026-05-25 to fit the
    // 44-px chip column; tooltips keep the long English form.
    assert_eq!(tr("tool.trim_transparency.label"), "TRIM");
    assert_eq!(tr("tool.trim_transparency.tooltip"), "Trim Transparency");
    assert_eq!(tr("tool.make_square.label"), "SQUAR");
    assert_eq!(tr("edit.undo.label"), "Undo");
    // Timeline panel chrome (W2.E9).
    assert_eq!(tr("panel.timeline.title"), "Timeline");
    assert_eq!(tr("panel.timeline.loop"), "Loop");
    assert_eq!(tr("panel.timeline.prop.translate_x"), "Translate X");
    // Vector panel chrome — section headers + the shape-catalogue words.
    assert_eq!(tr("panel.vector.section.tool"), "Tool");
    assert_eq!(tr("panel.vector.mode.shape"), "Shape");
    assert_eq!(tr("panel.vector.category"), "Category");
    assert_eq!(tr("panel.vector.group.iso"), "3D");
}

#[test]
fn a_text_key_says_what_its_key_says() {
    const TITLE: TextKey = TextKey::new("panel.timeline.title");
    assert_eq!(TITLE.tr(), tr("panel.timeline.title"));
    assert_eq!(TITLE.tr(), "Timeline");
    assert_eq!(
        TextKey::new("tool.nonexistent.bar").tr(),
        "tool.nonexistent.bar"
    );
}

#[test]
fn unknown_key_returns_the_key_itself() {
    let leaked = tr("tool.nonexistent.foo");
    assert_eq!(leaked, "tool.nonexistent.foo");
}
