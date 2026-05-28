//! Gate for the Painter sidebar size/opacity slider↔chip wiring.
//!
//! The chips display engineering units (px / %) while the slider stores a
//! normalized `0..1` value, so they MUST be linked with
//! `link_slider_number_mapped*` — NOT identity `link_slider_number`. An
//! identity link is the recurring 2026-05-27 split-brain bug: typing the
//! pixel value you see in the chip clamps the slider to its max. This test
//! traps re-introduction of the identity path and pins the affine mapping +
//! seed parity to `PainterUiSnapshot::default()` (audit W-1/W-2/W-3/Z-1,
//! 2026-05-28).

use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::Panel;
use ph2d_panel_painter_sidebar::PainterSidebarPanel;
use ph2d_panel_painter_sidebar::ids;
use ph2d_tool_painter::{
    PainterUiSnapshot, opacity01_to_pct, opacity_chip_mapping, size01_to_px, size_chip_mapping,
};

fn populated_store() -> WidgetStore {
    let mut store = WidgetStore::with_capacity(32);
    PainterSidebarPanel::populate(&mut store);
    store
}

#[test]
fn size_chip_uses_px_affine_mapping_not_identity() {
    let s = populated_store();
    assert_eq!(
        s.linked_number(ids::SIZE_SLIDER),
        Some(ids::SIZE_CHIP),
        "size slider→chip link missing"
    );
    assert_eq!(
        s.linked_slider(ids::SIZE_CHIP),
        Some(ids::SIZE_SLIDER),
        "size chip→slider link missing"
    );
    let (scale, offset) = s.linked_slider_mapping(ids::SIZE_CHIP);
    let (exp_scale, exp_offset) = size_chip_mapping();
    assert!(
        (scale - exp_scale).abs() < f32::EPSILON,
        "size mapping scale {scale} != {exp_scale} (identity scale=1 = split-brain bug)"
    );
    assert!(
        (offset - exp_offset).abs() < f32::EPSILON,
        "size mapping offset {offset} != {exp_offset}"
    );
    // Guard against silent regression to identity.
    assert!(
        (scale - 1.0).abs() > f32::EPSILON,
        "size chip must NOT be identity-linked (chip displays px, slider stores 0..1)"
    );
}

#[test]
fn opacity_chip_uses_percent_affine_mapping_not_identity() {
    let s = populated_store();
    assert_eq!(s.linked_number(ids::OPACITY_SLIDER), Some(ids::OPACITY_CHIP));
    assert_eq!(s.linked_slider(ids::OPACITY_CHIP), Some(ids::OPACITY_SLIDER));
    let (scale, offset) = s.linked_slider_mapping(ids::OPACITY_CHIP);
    let (exp_scale, exp_offset) = opacity_chip_mapping();
    assert!((scale - exp_scale).abs() < f32::EPSILON, "opacity scale {scale} != {exp_scale}");
    assert!((offset - exp_offset).abs() < f32::EPSILON, "opacity offset {offset} != {exp_offset}");
    assert!(
        (scale - 1.0).abs() > f32::EPSILON,
        "opacity chip must NOT be identity-linked (chip displays %, slider stores 0..1)"
    );
}

#[test]
fn seed_matches_snapshot_default() {
    // Audit W-3: populate seeds from PainterUiSnapshot::default() so the
    // boot seed and the paint fallback can never diverge.
    let s = populated_store();
    let def = PainterUiSnapshot::default();

    let (_, size_slider) = s.slider(ids::SIZE_SLIDER).expect("size slider seeded");
    assert!((size_slider - def.size01).abs() < f32::EPSILON, "size slider seed != snapshot default");
    let size_chip = s.number_value(ids::SIZE_CHIP).expect("size chip seeded");
    assert!(
        (size_chip as f32 - size01_to_px(def.size01)).abs() < 1e-3,
        "size chip seed must equal px(default), got {size_chip}"
    );

    let (_, op_slider) = s.slider(ids::OPACITY_SLIDER).expect("opacity slider seeded");
    assert!((op_slider - def.opacity01).abs() < f32::EPSILON, "opacity slider seed != snapshot default");
    let op_chip = s.number_value(ids::OPACITY_CHIP).expect("opacity chip seeded");
    assert!(
        (op_chip as f32 - opacity01_to_pct(def.opacity01)).abs() < 1e-3,
        "opacity chip seed must equal pct(default), got {op_chip}"
    );
}
