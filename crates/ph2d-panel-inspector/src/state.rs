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
    InspectorNameInfo, InspectorSpriteInfo, InspectorTransformInfo, InspectorVisibilityInfo,
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

#[allow(dead_code)]
pub(crate) fn current_pixels_per_meter() -> f32 {
    CURRENT_PIXELS_PER_METER.with(|c| c.get())
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
