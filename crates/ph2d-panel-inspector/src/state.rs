//! Inspector panel state — owned by `ErasedPanel<InspectorPanel>` after
//! ADR-0029 Phase C.1. Pre-Phase-C this struct lived on `HeroScreen`
//! as a flat field; now it's typed-down behind the trait registry and
//! reached via `host.panel_state_mut::<InspectorPanel>()` semantics
//! (typed `state: &mut InspectorState` handed to each `Panel` fn).
//!
//! Thread-locals here hold per-frame snapshots the host publishes
//! BEFORE the panel paints (`set_current_inspector_*`). They live as
//! thread-locals (not on `InspectorState`) because the publish/clear
//! cadence matches the paint loop, and re-routing every shell-side
//! `hero.inspector.sprite = …` to a typed-state mutation would be a
//! larger churn than the move warrants.

use ph2d_editor_core::screens::hero::{
    InspectorNameInfo, InspectorOrderingInfo, InspectorSpriteInfo, InspectorTransformInfo,
    InspectorVisibilityInfo,
};

/// Inspector panel retained state. Held inside `ErasedPanel<InspectorPanel>`
/// after Phase C.1; mutated by the panel's `paint` / `apply_event` and
/// by `sync_inspector_from_snapshots` (also panel-owned).
#[derive(Clone, Debug, Default)]
pub struct InspectorState {
    /// Entity bits of the last selection whose snapshot seeded the 5
    /// Transform NumberInputs + the editable Name field. When the
    /// current snapshot's entity bits differ, the panel force-rewrites
    /// the buffers so an in-progress edit on entity A doesn't apply
    /// to entity B after a selection switch.
    pub last_entity: Option<u64>,
    /// BulkSelect (T2.0): the selection size at the last checkbox reseed.
    /// Checkboxes seed once per selection change (so a just-toggled box
    /// doesn't flicker back). The primary entity switching is caught by
    /// `last_entity`; adding/removing an EXTRA keeps the primary but
    /// changes the count — so we reseed on a count change too, picking up
    /// the new Mixed state.
    pub last_selected_count: usize,
}

thread_local! {
    /// Content height measured during the previous paint pass of the
    /// LIVE Inspector. The wheel dispatch reads this via
    /// [`last_inspector_content_h`] to clamp `scroll_y`.
    pub(crate) static LAST_CONTENT_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };

    /// Exact visible body height of the inspector's last paint.
    pub(crate) static LAST_VISIBLE_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };

    /// M14.5: live sprite snapshot published by the host before
    /// each paint_inspector call and cleared after.
    pub(crate) static CURRENT_INSPECTOR_SPRITE: std::cell::RefCell<Option<InspectorSpriteInfo>> =
        const { std::cell::RefCell::new(None) };

    /// M14.A: live Transform snapshot for the Transform editor section.
    pub(crate) static CURRENT_INSPECTOR_TRANSFORM: std::cell::RefCell<Option<InspectorTransformInfo>> =
        const { std::cell::RefCell::new(None) };

    /// M14.D: live Visibility checkbox row.
    pub(crate) static CURRENT_INSPECTOR_VISIBILITY:
        std::cell::Cell<Option<InspectorVisibilityInfo>> =
        const { std::cell::Cell::new(None) };

    /// M14.E: editable entity-name snapshot.
    pub(crate) static CURRENT_INSPECTOR_NAME: std::cell::RefCell<Option<InspectorNameInfo>> =
        const { std::cell::RefCell::new(None) };

    /// W3 §7: live ordering/sorting snapshot for the Ordering section.
    pub(crate) static CURRENT_INSPECTOR_ORDERING:
        std::cell::RefCell<Option<InspectorOrderingInfo>> =
        const { std::cell::RefCell::new(None) };

    /// W3 §7: when the Sorting Layer dropdown is open, the section stashes
    /// `(selected_index, chip_rect)` here so `paint_inspector` paints its
    /// popover LAST (above every other widget). Panel-local — distinct
    /// from the showcase's shared `PENDING_DROPDOWN_CHIP`.
    pub(crate) static PENDING_ORDERING_DD:
        std::cell::Cell<Option<(usize, ph2d_editor_core::zones::Rect)>> =
        const { std::cell::Cell::new(None) };

    /// Per-paint display unit + pixels_per_meter for Transform section
    /// labels + value formatting.
    pub(crate) static CURRENT_DISPLAY_UNIT: std::cell::Cell<ph2d_editor_core::project::DisplayUnit> =
        const { std::cell::Cell::new(ph2d_editor_core::project::DisplayUnit::Meters) };
    pub(crate) static CURRENT_PIXELS_PER_METER: std::cell::Cell<f32> =
        const { std::cell::Cell::new(ph2d_editor_core::project::DEFAULT_PIXELS_PER_METER) };
}

pub fn set_current_inspector_sprite(info: Option<InspectorSpriteInfo>) {
    CURRENT_INSPECTOR_SPRITE.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_sprite() -> Option<InspectorSpriteInfo> {
    CURRENT_INSPECTOR_SPRITE.with(|c| c.borrow().clone())
}

pub fn set_current_inspector_transform(info: Option<InspectorTransformInfo>) {
    CURRENT_INSPECTOR_TRANSFORM.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_transform() -> Option<InspectorTransformInfo> {
    CURRENT_INSPECTOR_TRANSFORM.with(|c| *c.borrow())
}

pub fn set_current_inspector_visibility(info: Option<InspectorVisibilityInfo>) {
    CURRENT_INSPECTOR_VISIBILITY.with(|c| c.set(info));
}

pub(crate) fn current_inspector_visibility() -> Option<InspectorVisibilityInfo> {
    CURRENT_INSPECTOR_VISIBILITY.with(|c| c.get())
}

pub fn set_current_inspector_name(info: Option<InspectorNameInfo>) {
    CURRENT_INSPECTOR_NAME.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_name() -> Option<InspectorNameInfo> {
    CURRENT_INSPECTOR_NAME.with(|c| c.borrow().clone())
}

pub(crate) fn current_inspector_name_is_some() -> bool {
    CURRENT_INSPECTOR_NAME.with(|c| c.borrow().is_some())
}

pub fn set_current_inspector_ordering(info: Option<InspectorOrderingInfo>) {
    CURRENT_INSPECTOR_ORDERING.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_ordering() -> Option<InspectorOrderingInfo> {
    CURRENT_INSPECTOR_ORDERING.with(|c| *c.borrow())
}

pub(crate) fn set_pending_ordering_dd(
    chip: Option<(usize, ph2d_editor_core::zones::Rect)>,
) {
    PENDING_ORDERING_DD.with(|c| c.set(chip));
}

pub(crate) fn take_pending_ordering_dd() -> Option<(usize, ph2d_editor_core::zones::Rect)> {
    PENDING_ORDERING_DD.with(|c| c.take())
}

pub fn set_current_display_unit(
    unit: ph2d_editor_core::project::DisplayUnit,
    pixels_per_meter: f32,
) {
    CURRENT_DISPLAY_UNIT.with(|c| c.set(unit));
    CURRENT_PIXELS_PER_METER.with(|c| c.set(pixels_per_meter));
}

pub(crate) fn current_display_unit() -> ph2d_editor_core::project::DisplayUnit {
    CURRENT_DISPLAY_UNIT.with(|c| c.get())
}

pub(crate) fn current_pixels_per_meter() -> f32 {
    CURRENT_PIXELS_PER_METER.with(|c| c.get())
}

/// Pack a linear/sRGB f32 RGBA in `[0, 1]` into `[u8; 4]` for the
/// color-swatch fill + `INSP_BLENDER_PICKER` seed. Round-to-nearest
/// (the `+ 0.5` before truncation) so a committed channel and its
/// re-decoded byte agree, and the picker doesn't reopen one step off.
pub(crate) fn tint_f32_to_u8(c: [f32; 4]) -> [u8; 4] {
    let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit denormalize, not a design token
    [q(c[0]), q(c[1]), q(c[2]), q(c[3])]
}

/// Inverse of [`tint_f32_to_u8`]: the picker round-trips the chosen
/// color through `widget_color(target)` as `[u8; 4]`; this unpacks it
/// back to the `[f32; 4]` the `Sprite` tint channels store.
pub(crate) fn tint_u8_to_f32(c: [u8; 4]) -> [f32; 4] {
    // LITERAL-PX-OK (×4): sRGB 8-bit normalize, not a design token.
    [
        c[0] as f32 / 255.0, // LITERAL-PX-OK: sRGB byte normalize
        c[1] as f32 / 255.0, // LITERAL-PX-OK: sRGB byte normalize
        c[2] as f32 / 255.0, // LITERAL-PX-OK: sRGB byte normalize
        c[3] as f32 / 255.0, // LITERAL-PX-OK: sRGB byte normalize
    ]
}

/// Last-known total content height of the inspector body. Used by
/// `dispatch_wheel` to clamp the scroll offset.
pub fn last_inspector_content_h() -> f32 {
    LAST_CONTENT_H.with(|c| c.get())
}

pub(crate) fn set_last_inspector_content_h(h: f32) {
    LAST_CONTENT_H.with(|c| c.set(h));
}

pub fn last_inspector_visible_h() -> f32 {
    LAST_VISIBLE_H.with(|c| c.get())
}

pub(crate) fn set_last_inspector_visible_h(h: f32) {
    LAST_VISIBLE_H.with(|c| c.set(h));
}

#[cfg(test)]
mod tint_color_tests {
    use super::{tint_f32_to_u8, tint_u8_to_f32};

    /// The convergence invariant `sync.rs` relies on: once a picked
    /// byte color is committed (→ `f32` channel) and the snapshot comes
    /// back, re-packing it must yield the SAME byte — otherwise the
    /// "picked != committed" guard never settles and the bus spins
    /// every frame. Must hold for ALL 256 levels on every channel.
    #[test]
    fn u8_to_f32_round_trips_exactly_for_every_level() {
        for b in 0u8..=255 {
            let rgba = [b, b, b, b];
            assert_eq!(
                tint_f32_to_u8(tint_u8_to_f32(rgba)),
                rgba,
                "level {b} did not round-trip"
            );
        }
    }

    #[test]
    fn f32_to_u8_clamps_out_of_unit_range() {
        assert_eq!(tint_f32_to_u8([2.0, -0.5, 1.0, 0.0]), [255, 0, 255, 0]);
    }

    #[test]
    fn white_default_maps_to_opaque_white() {
        assert_eq!(tint_f32_to_u8([1.0, 1.0, 1.0, 1.0]), [255, 255, 255, 255]);
    }

    #[test]
    fn f32_to_u8_rounds_to_nearest_not_truncating() {
        // 0.5 · 255 = 127.5 → rounds up to 128 (truncation would give 127).
        assert_eq!(tint_f32_to_u8([0.5, 0.5, 0.5, 0.5]), [128, 128, 128, 128]);
    }
}
