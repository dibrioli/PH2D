//! Panel-local **ramp** UI state accessors (split from [`crate::state`] for the panel LOC cap): the
//! Grain + Shape ramps' selected-stop ids and their deferred dropdown-popover stashes. The backing
//! thread-locals live in [`crate::state`] (marked `pub(crate)`); these are thin set/take wrappers,
//! re-exported from `crate::state` so callers keep using the `state::*` path.

use ph2d_editor_core::zones::Rect;

// ── Selected stop (stable id) ────────────────────────────────────────────────────────────────────
/// Set the Grain Color Ramp's selected stop id (clicked/dragged on the bar).
pub(crate) fn set_selected_ramp_stop(i: u8) {
    crate::state::SELECTED_RAMP_STOP.with(|c| c.set(i));
}

/// The Grain Color Ramp's selected stop id (default `0`).
pub(crate) fn selected_ramp_stop() -> u8 {
    crate::state::SELECTED_RAMP_STOP.with(|c| c.get())
}

/// Set the selected **Shape**-ramp stop (stable id).
pub(crate) fn set_selected_shape_ramp_stop(i: u8) {
    crate::state::SELECTED_SHAPE_RAMP_STOP.with(|c| c.set(i));
}

/// The selected **Shape**-ramp stop (stable id).
pub(crate) fn selected_shape_ramp_stop() -> u8 {
    crate::state::SELECTED_SHAPE_RAMP_STOP.with(|c| c.get())
}

// ── Grain ramp deferred dropdown popovers ────────────────────────────────────────────────────────
/// Stash the open Grain Color Ramp Mode dropdown for the deferred popover pass.
pub(crate) fn set_pending_ramp_mode_dd(v: Option<(Rect, u8)>) {
    crate::state::PENDING_RAMP_MODE_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending Grain Color Ramp Mode dropdown.
pub(crate) fn take_pending_ramp_mode_dd() -> Option<(Rect, u8)> {
    crate::state::PENDING_RAMP_MODE_DD.with(|c| c.take())
}

/// Stash the open Grain Color Ramp Interpolation dropdown for the deferred popover pass.
pub(crate) fn set_pending_ramp_interp_dd(v: Option<(Rect, u8)>) {
    crate::state::PENDING_RAMP_INTERP_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending Grain Color Ramp Interpolation dropdown.
pub(crate) fn take_pending_ramp_interp_dd() -> Option<(Rect, u8)> {
    crate::state::PENDING_RAMP_INTERP_DD.with(|c| c.take())
}

/// Stash the open Grain Color Ramp Alpha-action dropdown for the deferred popover pass.
pub(crate) fn set_pending_ramp_alpha_dd(v: Option<(Rect, u8)>) {
    crate::state::PENDING_RAMP_ALPHA_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending Grain Color Ramp Alpha-action dropdown.
pub(crate) fn take_pending_ramp_alpha_dd() -> Option<(Rect, u8)> {
    crate::state::PENDING_RAMP_ALPHA_DD.with(|c| c.take())
}

// ── Shape ramp deferred dropdown popovers ────────────────────────────────────────────────────────
/// Stash the open Shape-ramp Interpolation dropdown for the deferred popover pass.
pub(crate) fn set_pending_shape_ramp_interp_dd(v: Option<(Rect, u8)>) {
    crate::state::PENDING_SHAPE_RAMP_INTERP_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending Shape-ramp Interpolation dropdown.
pub(crate) fn take_pending_shape_ramp_interp_dd() -> Option<(Rect, u8)> {
    crate::state::PENDING_SHAPE_RAMP_INTERP_DD.with(|c| c.take())
}

/// Stash the open Shape-ramp colour-Mode dropdown for the deferred popover pass.
pub(crate) fn set_pending_shape_ramp_mode_dd(v: Option<(Rect, u8)>) {
    crate::state::PENDING_SHAPE_RAMP_MODE_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending Shape-ramp colour-Mode dropdown.
pub(crate) fn take_pending_shape_ramp_mode_dd() -> Option<(Rect, u8)> {
    crate::state::PENDING_SHAPE_RAMP_MODE_DD.with(|c| c.take())
}

/// Stash the open Shape-ramp Alpha-action dropdown for the deferred popover pass.
pub(crate) fn set_pending_shape_ramp_alpha_dd(v: Option<(Rect, u8)>) {
    crate::state::PENDING_SHAPE_RAMP_ALPHA_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending Shape-ramp Alpha-action dropdown.
pub(crate) fn take_pending_shape_ramp_alpha_dd() -> Option<(Rect, u8)> {
    crate::state::PENDING_SHAPE_RAMP_ALPHA_DD.with(|c| c.take())
}
