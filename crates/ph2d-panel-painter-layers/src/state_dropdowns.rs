//! Panel-local **brush-section dropdown** deferred-popover stashes (split from [`crate::state`] for the
//! panel LOC cap, mirroring [`crate::state_ramp`]): the one-open-at-a-time `(chip_rect, value)` slots for
//! the Preset / Blend / Falloff / Stroke-Method / Jitter-Unit / Texture-Kind / Shape-Kind / Texture-Mapping
//! chips. The backing thread-locals live in [`crate::state`] (marked `pub(crate)`); these are thin
//! set/take wrappers, re-exported from `crate::state` so callers keep using the `state::*` path.

use crate::state;
use ph2d_editor_core::zones::Rect;

/// Stash the open top-of-panel Preset dropdown for the deferred popover pass.
pub(crate) fn set_pending_brush_preset_dd(v: Option<(Rect, u8)>) {
    state::PENDING_BRUSH_PRESET_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending Preset dropdown for the deferred popover.
pub(crate) fn take_pending_brush_preset_dd() -> Option<(Rect, u8)> {
    state::PENDING_BRUSH_PRESET_DD.with(|c| c.take())
}

/// Stash the open Watercolor Paper kind dropdown for the deferred popover pass.
pub(crate) fn set_pending_paper_kind_dd(v: Option<(Rect, u8)>) {
    state::PENDING_PAPER_KIND_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending Watercolor Paper kind dropdown.
pub(crate) fn take_pending_paper_kind_dd() -> Option<(Rect, u8)> {
    state::PENDING_PAPER_KIND_DD.with(|c| c.take())
}

/// Stash the open brush blend dropdown for the deferred popover pass.
pub(crate) fn set_pending_brush_blend_dd(v: Option<(Rect, u8)>) {
    state::PENDING_BRUSH_BLEND_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending brush blend dropdown for the deferred popover paint.
pub(crate) fn take_pending_brush_blend_dd() -> Option<(Rect, u8)> {
    state::PENDING_BRUSH_BLEND_DD.with(|c| c.take())
}

/// Stash the open brush Falloff dropdown for the deferred popover pass.
pub(crate) fn set_pending_brush_falloff_dd(v: Option<(Rect, u8)>) {
    state::PENDING_BRUSH_FALLOFF_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending brush Falloff dropdown for the deferred popover.
pub(crate) fn take_pending_brush_falloff_dd() -> Option<(Rect, u8)> {
    state::PENDING_BRUSH_FALLOFF_DD.with(|c| c.take())
}

/// Stash the open Stroke-section Method dropdown for the deferred popover pass.
pub(crate) fn set_pending_brush_stroke_method_dd(v: Option<(Rect, u8)>) {
    state::PENDING_BRUSH_STROKE_METHOD_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending Stroke Method dropdown for the deferred popover.
pub(crate) fn take_pending_brush_stroke_method_dd() -> Option<(Rect, u8)> {
    state::PENDING_BRUSH_STROKE_METHOD_DD.with(|c| c.take())
}

/// Stash the open Stroke-section Jitter-Unit dropdown for the deferred popover pass.
pub(crate) fn set_pending_brush_jitter_unit_dd(v: Option<(Rect, u8)>) {
    state::PENDING_BRUSH_JITTER_UNIT_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending Stroke Jitter-Unit dropdown for the deferred popover.
pub(crate) fn take_pending_brush_jitter_unit_dd() -> Option<(Rect, u8)> {
    state::PENDING_BRUSH_JITTER_UNIT_DD.with(|c| c.take())
}

/// Stash the open Texture-section Kind picker for the deferred popover pass.
pub(crate) fn set_pending_brush_texture_kind_dd(v: Option<(Rect, u8)>) {
    state::PENDING_BRUSH_TEXTURE_KIND_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending Texture Kind picker for the deferred popover.
pub(crate) fn take_pending_brush_texture_kind_dd() -> Option<(Rect, u8)> {
    state::PENDING_BRUSH_TEXTURE_KIND_DD.with(|c| c.take())
}

/// Stash the open Shape-section source picker for the deferred popover pass.
pub(crate) fn set_pending_brush_shape_kind_dd(v: Option<(Rect, u8)>) {
    state::PENDING_BRUSH_SHAPE_KIND_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending Shape source picker for the deferred popover.
pub(crate) fn take_pending_brush_shape_kind_dd() -> Option<(Rect, u8)> {
    state::PENDING_BRUSH_SHAPE_KIND_DD.with(|c| c.take())
}

/// Stash the open Texture-section Mapping dropdown for the deferred popover pass.
pub(crate) fn set_pending_brush_texture_mapping_dd(v: Option<(Rect, u8)>) {
    state::PENDING_BRUSH_TEXTURE_MAPPING_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending Texture Mapping dropdown for the deferred popover.
pub(crate) fn take_pending_brush_texture_mapping_dd() -> Option<(Rect, u8)> {
    state::PENDING_BRUSH_TEXTURE_MAPPING_DD.with(|c| c.take())
}
