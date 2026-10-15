//! Panel-local state of the **Shape Color** layers (prompt 38): the Falloff Color ramp's selected stop
//! and its deferred Mode / Interpolation popovers, and the Blend popover. Its own file (not
//! [`crate::state`], at the panel LOC cap); the Texture Color ramp keeps the Shape ramp's state.

use ph2d_editor_core::zones::Rect;
use std::cell::Cell;

thread_local! {
    static SELECTED_FALLOFF_STOP: Cell<u8> = const { Cell::new(0) };
    static PENDING_FALLOFF_MODE_DD: Cell<Option<(Rect, u8)>> = const { Cell::new(None) };
    static PENDING_FALLOFF_INTERP_DD: Cell<Option<(Rect, u8)>> = const { Cell::new(None) };
    static PENDING_BLEND_DD: Cell<Option<(Rect, u8)>> = const { Cell::new(None) };
}

/// Set the Falloff Color ramp's selected stop (stable id).
pub(crate) fn set_selected_falloff_stop(i: u8) {
    SELECTED_FALLOFF_STOP.with(|c| c.set(i));
}

/// The Falloff Color ramp's selected stop (stable id, default `0`).
pub(crate) fn selected_falloff_stop() -> u8 {
    SELECTED_FALLOFF_STOP.with(Cell::get)
}

/// Stash the open Falloff-ramp Mode dropdown for the deferred popover pass.
pub(crate) fn set_pending_falloff_mode_dd(v: Option<(Rect, u8)>) {
    PENDING_FALLOFF_MODE_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending Falloff-ramp Mode dropdown.
pub(crate) fn take_pending_falloff_mode_dd() -> Option<(Rect, u8)> {
    PENDING_FALLOFF_MODE_DD.with(Cell::take)
}

/// Stash the open Falloff-ramp Interpolation dropdown for the deferred popover pass.
pub(crate) fn set_pending_falloff_interp_dd(v: Option<(Rect, u8)>) {
    PENDING_FALLOFF_INTERP_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending Falloff-ramp Interpolation dropdown.
pub(crate) fn take_pending_falloff_interp_dd() -> Option<(Rect, u8)> {
    PENDING_FALLOFF_INTERP_DD.with(Cell::take)
}

/// Stash the open layers' Blend dropdown for the deferred popover pass.
pub(crate) fn set_pending_blend_dd(v: Option<(Rect, u8)>) {
    PENDING_BLEND_DD.with(|c| c.set(v));
}

/// Take (and clear) the pending layers' Blend dropdown.
pub(crate) fn take_pending_blend_dd() -> Option<(Rect, u8)> {
    PENDING_BLEND_DD.with(Cell::take)
}
