//! Asserts the Padding panel's 4 edge slider↔chip pairs are wired
//! with the affine mapping that matches the tool's `px_to_slider` /
//! `slider_to_px` projection. The pair was migrated 2026-05-27 from a
//! manual mirror in `event.rs` to `link_slider_number_mapped` — these
//! tests trap silent re-introduction of the manual path.

use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::Panel;
use ph2d_panel_padding::PaddingPanel;
use ph2d_tool_padding::params::PAD_SLIDER_FULL_SCALE;

fn populated_store() -> WidgetStore {
    let mut store = WidgetStore::with_capacity(32);
    PaddingPanel::populate(&mut store);
    store
}

#[test]
fn each_edge_pair_uses_bipolar_px_mapping() {
    let s = populated_store();
    let full = PAD_SLIDER_FULL_SCALE as f32;
    let expected_scale = 2.0 * full;
    let expected_offset = -full;
    for (slider, chip, label) in [
        (
            ph2d_tool_padding::ids::PAD_TOP,
            ph2d_tool_padding::ids::PAD_TOP_NUM,
            "TOP",
        ),
        (
            ph2d_tool_padding::ids::PAD_RIGHT,
            ph2d_tool_padding::ids::PAD_RIGHT_NUM,
            "RIGHT",
        ),
        (
            ph2d_tool_padding::ids::PAD_BOTTOM,
            ph2d_tool_padding::ids::PAD_BOTTOM_NUM,
            "BOTTOM",
        ),
        (
            ph2d_tool_padding::ids::PAD_LEFT,
            ph2d_tool_padding::ids::PAD_LEFT_NUM,
            "LEFT",
        ),
    ] {
        assert_eq!(
            s.linked_number(slider),
            Some(chip),
            "{label}: slider→chip link missing"
        );
        assert_eq!(
            s.linked_slider(chip),
            Some(slider),
            "{label}: chip→slider link missing"
        );
        let (scale, offset) = s.linked_slider_mapping(chip);
        assert!(
            (scale - expected_scale).abs() < f32::EPSILON,
            "{label}: mapping scale {scale} != {expected_scale}"
        );
        assert!(
            (offset - expected_offset).abs() < f32::EPSILON,
            "{label}: mapping offset {offset} != {expected_offset}"
        );
    }
}

#[test]
fn each_edge_chip_seeded_at_zero_px() {
    // Default neutral: chip stores 0 px (display-space natural unit),
    // slider stores 0.5 (centre of 0..1 track). Affine projection
    // 0.5 * 1024 - 512 = 0 — round-trip exact.
    let s = populated_store();
    for chip in [
        ph2d_tool_padding::ids::PAD_TOP_NUM,
        ph2d_tool_padding::ids::PAD_RIGHT_NUM,
        ph2d_tool_padding::ids::PAD_BOTTOM_NUM,
        ph2d_tool_padding::ids::PAD_LEFT_NUM,
    ] {
        let v = s.number_value(chip).expect("chip");
        assert!(v.abs() < 1e-9, "chip {chip:?} seed expected 0 px, got {v}");
    }
    for slider in [
        ph2d_tool_padding::ids::PAD_TOP,
        ph2d_tool_padding::ids::PAD_RIGHT,
        ph2d_tool_padding::ids::PAD_BOTTOM,
        ph2d_tool_padding::ids::PAD_LEFT,
    ] {
        let (_, v) = s.slider(slider).expect("slider");
        assert!(
            (v - 0.5).abs() < f32::EPSILON,
            "slider {slider:?} seed expected 0.5, got {v}"
        );
    }
}
