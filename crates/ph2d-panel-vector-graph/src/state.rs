//! Geometry-Graph panel state + the panel→shell "param bus".
//!
//! Per-instance state is empty (ADR-0029 §4.3, mirror of the Vector
//! Inspector): the authoritative param values live in a thread-local
//! [`VectorGraphParams`] bus published by the slider event handler. The
//! shell render bridge reads [`current_graph_params`] each frame to drive
//! the `vector.source` cook + render. The bus is seeded to
//! [`VectorGraphParams::default`] so a cook BEFORE any slider move still
//! produces the default Rect (100×100).

use crate::ids;
use ph2d_a11y::NodeId;
use std::cell::{Cell, RefCell};

/// One geometry-graph param: its slider id, the real-value range
/// (`lo..hi`) the normalized `0..1` track maps onto, and its default
/// real value. Single source of truth shared by `populate` (seeds each
/// slider's track from `default`), `event` (maps `track → real`), and
/// `paint` (labels + value display). Order matches the spec table /
/// the paint-row order.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ParamSpec {
    pub id: NodeId,
    pub label: &'static str,
    pub lo: f32,
    pub hi: f32,
    pub default: f32,
}

impl ParamSpec {
    /// Map a normalized `0..1` track value to this param's real range.
    pub(crate) fn track_to_value(&self, track: f32) -> f32 {
        self.lo + track.clamp(0.0, 1.0) * (self.hi - self.lo)
    }

    /// Inverse of [`track_to_value`]: the `0..1` track that yields
    /// `value`. Used to seed each slider from its default.
    pub(crate) fn value_to_track(&self, value: f32) -> f32 {
        if (self.hi - self.lo).abs() < f32::EPSILON {
            0.0
        } else {
            ((value - self.lo) / (self.hi - self.lo)).clamp(0.0, 1.0)
        }
    }
}

/// The 8 geometry-graph params in spec / paint-row order. The ranges +
/// defaults here are the canonical contract the shell render bridge
/// depends on (see [`VectorGraphParams::default`], which is derived from
/// these defaults).
pub(crate) const PARAMS: [ParamSpec; 8] = [
    ParamSpec {
        id: ids::VGRAPH_KIND,
        label: "Kind",
        lo: 0.0,
        hi: 4.0,
        default: 0.0,
    },
    ParamSpec {
        id: ids::VGRAPH_WIDTH,
        label: "Width",
        lo: 0.0,
        hi: 400.0,
        default: 100.0,
    },
    ParamSpec {
        id: ids::VGRAPH_HEIGHT,
        label: "Height",
        lo: 0.0,
        hi: 400.0,
        default: 100.0,
    },
    ParamSpec {
        id: ids::VGRAPH_SIDES,
        label: "Sides",
        lo: 3.0,
        hi: 24.0,
        default: 6.0,
    },
    ParamSpec {
        id: ids::VGRAPH_INNER_RATIO,
        label: "Inner Ratio",
        lo: 0.0,
        hi: 1.0,
        default: 0.4,
    },
    ParamSpec {
        id: ids::VGRAPH_TURNS,
        label: "Turns",
        lo: 1.0,
        hi: 12.0,
        default: 3.0,
    },
    ParamSpec {
        id: ids::VGRAPH_SAMPLES_PER_TURN,
        label: "Samples/Turn",
        lo: 4.0,
        hi: 64.0,
        default: 24.0,
    },
    ParamSpec {
        id: ids::VGRAPH_ROTATION,
        // Full-turn radians range (spec `0.0 .. 6.2832` ≈ 2π); use the
        // exact `TAU` constant so clippy::approx_constant is satisfied.
        label: "Rotation",
        lo: 0.0,
        hi: std::f32::consts::TAU,
        default: 0.0,
    },
];

/// The 5 geometry kinds the `kind` param selects, in index order
/// (`0=Rect 1=Ellipse 2=Polygon 3=Star 4=Spiral`). `paint` rounds the
/// `kind` real value to the nearest int and shows the variant name.
pub(crate) const KIND_NAMES: [&str; 5] = ["Rect", "Ellipse", "Polygon", "Star", "Spiral"];

/// The current geometry-graph param values, as driven by the panel
/// sliders. Each field is a **real** value in its param's range (the
/// `0..1` slider track is mapped to the range in [`crate::event`]). The
/// shell render bridge reads these each frame to cook + render the
/// `vector.source` node.
///
/// `kind` is a float in `0.0..=4.0`; round to the nearest int on read
/// (`0=Rect 1=Ellipse 2=Polygon 3=Star 4=Spiral`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VectorGraphParams {
    pub kind: f32,
    pub width: f32,
    pub height: f32,
    pub sides: f32,
    pub inner_ratio: f32,
    pub turns: f32,
    pub samples_per_turn: f32,
    pub rotation: f32,
}

impl Default for VectorGraphParams {
    fn default() -> Self {
        // Derived from the canonical defaults in `PARAMS` so the bus
        // seed + the slider seeds can never drift.
        Self {
            kind: PARAMS[0].default,
            width: PARAMS[1].default,
            height: PARAMS[2].default,
            sides: PARAMS[3].default,
            inner_ratio: PARAMS[4].default,
            turns: PARAMS[5].default,
            samples_per_turn: PARAMS[6].default,
            rotation: PARAMS[7].default,
        }
    }
}

impl VectorGraphParams {
    /// Write the real `value` for the param identified by `id`. No-op if
    /// `id` is not one of the 8 graph params. Called by the slider event
    /// handler after mapping the `0..1` track to the param range.
    pub(crate) fn set_by_id(&mut self, id: NodeId, value: f32) {
        if id == ids::VGRAPH_KIND {
            self.kind = value;
        } else if id == ids::VGRAPH_WIDTH {
            self.width = value;
        } else if id == ids::VGRAPH_HEIGHT {
            self.height = value;
        } else if id == ids::VGRAPH_SIDES {
            self.sides = value;
        } else if id == ids::VGRAPH_INNER_RATIO {
            self.inner_ratio = value;
        } else if id == ids::VGRAPH_TURNS {
            self.turns = value;
        } else if id == ids::VGRAPH_SAMPLES_PER_TURN {
            self.samples_per_turn = value;
        } else if id == ids::VGRAPH_ROTATION {
            self.rotation = value;
        }
    }
}

thread_local! {
    /// The "param bus": authoritative current param values, seeded to
    /// [`VectorGraphParams::default`] so a cook before any slider move
    /// produces the default Rect. Updated by [`crate::event`] on each
    /// slider `ValueChanged`; read by the shell render bridge via
    /// [`current_graph_params`].
    static CURRENT_PARAMS: RefCell<VectorGraphParams> =
        const { RefCell::new(VectorGraphParams::DEFAULT_CONST) };

    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
}

impl VectorGraphParams {
    /// `const` twin of [`Default::default`] (a `thread_local!` initializer
    /// must be `const`). Kept in lock-step with `PARAMS` by the unit test
    /// `default_matches_param_specs`.
    const DEFAULT_CONST: Self = Self {
        kind: 0.0,
        width: 100.0,
        height: 100.0,
        sides: 6.0,
        inner_ratio: 0.4,
        turns: 3.0,
        samples_per_turn: 24.0,
        rotation: 0.0,
    };
}

/// The current param values from the panel sliders. Updated by the slider
/// event handler; the shell render bridge reads this each frame to drive
/// the cook. Returns [`VectorGraphParams::default`] until the user moves a
/// slider.
#[must_use]
pub fn current_graph_params() -> VectorGraphParams {
    CURRENT_PARAMS.with(|p| *p.borrow())
}

/// Write the real `value` for param `id` into the bus. Called by
/// [`crate::event`] after mapping a slider's `0..1` track to its range.
pub(crate) fn set_param(id: NodeId, value: f32) {
    CURRENT_PARAMS.with(|p| p.borrow_mut().set_by_id(id, value));
}

/// Retained per-instance state slot for `VectorGraphPanel`. Intentionally
/// empty — the authoritative param values live in the thread-local bus so
/// the shell render bridge can read them. `Default` is required by the
/// `Panel::State: Default` bound.
#[derive(Clone, Debug, Default)]
pub struct VectorGraphPanelState;

#[must_use]
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(|c| c.get())
}

#[must_use]
pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(|c| c.get())
}

pub(crate) fn set_last_content_h(v: f32) {
    LAST_CONTENT_H.with(|c| c.set(v));
}

pub(crate) fn set_last_visible_h(v: f32) {
    LAST_VISIBLE_H.with(|c| c.set(v));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_param_specs() {
        // The bus seed + the `Default` impl must derive from `PARAMS`.
        let d = VectorGraphParams::default();
        assert_eq!(d, VectorGraphParams::DEFAULT_CONST);
        assert_eq!(d.kind, PARAMS[0].default);
        assert_eq!(d.width, PARAMS[1].default);
        assert_eq!(d.height, PARAMS[2].default);
        assert_eq!(d.sides, PARAMS[3].default);
        assert_eq!(d.inner_ratio, PARAMS[4].default);
        assert_eq!(d.turns, PARAMS[5].default);
        assert_eq!(d.samples_per_turn, PARAMS[6].default);
        assert_eq!(d.rotation, PARAMS[7].default);
    }

    #[test]
    fn current_graph_params_returns_defaults_before_any_edit() {
        // Fresh thread-local ⇒ default Rect (100×100). This is the
        // contract the shell bridge relies on for a pre-interaction cook.
        assert_eq!(current_graph_params(), VectorGraphParams::default());
    }

    #[test]
    fn set_param_writes_real_value_into_bus() {
        // Mutates the thread-local bus; uses a distinct field so it does
        // not race the defaults assertion above (separate test threads).
        set_param(ids::VGRAPH_SIDES, 12.0);
        assert_eq!(current_graph_params().sides, 12.0);
    }

    #[test]
    fn track_to_value_maps_endpoints_and_midpoint() {
        let sides = PARAMS[3]; // 3.0..24.0
        assert_eq!(sides.track_to_value(0.0), 3.0);
        assert_eq!(sides.track_to_value(1.0), 24.0);
        assert!((sides.track_to_value(0.5) - 13.5).abs() < 1e-4);
        // Round-trip: default → track → default.
        let t = sides.value_to_track(sides.default);
        assert!((sides.track_to_value(t) - sides.default).abs() < 1e-4);
    }
}
