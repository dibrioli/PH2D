//! Stroke method + jitter unit — the discrete options of the Blender "Stroke" panel.
//!
//! Behavioural reference (clean-room, no code copied): Blender
//! `editors/sculpt_paint/paint_stroke.cc` (the `eBrushStrokeType` dispatch in `PaintStroke::modal`)
//! and `makesdna/DNA_brush_enums.h` (the enum and the `BRUSH_ABSOLUTE_JITTER` flag). The wire
//! discriminants returned by [`StrokeMethod::to_u8`] mirror Blender's `eBrushStrokeType` numeric
//! values (Dots=0 … Curve=6) so the cross-crate encoding is a documented, stable anchor.

/// How a pointer path is turned into dab positions — Blender's `eBrushStrokeType`.
///
/// Only [`StrokeMethod::Space`] resamples the path at fixed arc-length intervals; the per-event
/// methods ([`Dots`](Self::Dots)/[`Airbrush`](Self::Airbrush)/[`DragDot`](Self::DragDot)) emit one
/// dab per processed sample. [`Anchored`](Self::Anchored)/[`Line`](Self::Line)/
/// [`Curve`](Self::Curve) are *interactive* (finalise on release / Bézier authoring) and the
/// engine only fills their geometry — the interaction lives in the tool/shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StrokeMethod {
    /// One dab per input event at the (smoothed) cursor — no resampling. `BRUSH_STROKE_DOTS` (0).
    Dots,
    /// Like [`Dots`](Self::Dots) but also emitted on a timer at `rate` Hz while held.
    /// `BRUSH_STROKE_AIRBRUSH` (1).
    Airbrush,
    /// A single stamp centred at the press point, resized by drag distance.
    /// `BRUSH_STROKE_ANCHORED` (2).
    Anchored,
    /// Dabs at fixed arc-length intervals (`spacing × diameter`) along the path — the default.
    /// `BRUSH_STROKE_SPACE` (3).
    #[default]
    Space,
    /// A single stamp that follows the cursor; pressure forced 1, no jitter/smoothing.
    /// `BRUSH_STROKE_DRAG_DOT` (4).
    DragDot,
    /// A straight line filled with spaced dabs, finalised on release. `BRUSH_STROKE_LINE` (5).
    Line,
    /// A Bézier paint-curve filled with spaced dabs, stamped on finalise. `BRUSH_STROKE_CURVE` (6).
    Curve,
}

impl StrokeMethod {
    /// Wire discriminant (matches Blender's `eBrushStrokeType` value).
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Dots => 0,
            Self::Airbrush => 1,
            Self::Anchored => 2,
            Self::Space => 3,
            Self::DragDot => 4,
            Self::Line => 5,
            Self::Curve => 6,
        }
    }

    /// Decode a wire discriminant; unknown values fall back to [`StrokeMethod::Space`] (the brush
    /// default) rather than `Dots`, so a corrupt/forward value paints sensibly.
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Dots,
            1 => Self::Airbrush,
            2 => Self::Anchored,
            4 => Self::DragDot,
            5 => Self::Line,
            6 => Self::Curve,
            _ => Self::Space,
        }
    }

    /// True when this method resamples the path at the brush spacing (only `Space`; `Line`/`Curve`
    /// also fill by spacing but on finalise, not interactively).
    #[must_use]
    pub fn is_spaced(self) -> bool {
        matches!(self, Self::Space)
    }

    /// True when the **Spacing** param (and its "Adjust Strength" attenuation toggle)
    /// is meaningful — the methods that lay dabs at a fixed arc-length interval
    /// (Blender: SPACE/LINE/CURVE). Same set as [`uses_dash`](Self::uses_dash) today
    /// (both controls are spacing-driven), but kept as its own predicate so the UI
    /// gate reads by intent rather than by coincidence, and the two can diverge.
    #[must_use]
    pub fn uses_spacing(self) -> bool {
        matches!(self, Self::Space | Self::Line | Self::Curve)
    }

    /// True when dabs are gated by the dash pattern (Blender: SPACE/LINE/CURVE).
    #[must_use]
    pub fn uses_dash(self) -> bool {
        matches!(self, Self::Space | Self::Line | Self::Curve)
    }

    /// True when the stroke **stabilizer** (the "how regular" knob) applies. Mirrors Blender's
    /// `paint_supports_smooth_stroke`, which enables smooth-stroke for every method **except**
    /// `ANCHORED`/`DRAG_DOT`/`LINE` — so `Space`/`Dots`/`Airbrush`/`Curve` all run the position
    /// filter (Space additionally runs the spline). Drag Dot/Anchored need exact cursor placement.
    #[must_use]
    pub fn uses_stabilizer(self) -> bool {
        !matches!(self, Self::Anchored | Self::DragDot | Self::Line)
    }

    /// True when this method forces pressure to 1.0 (Blender: DRAG_DOT, ANCHORED, LINE).
    #[must_use]
    pub fn forces_full_pressure(self) -> bool {
        matches!(self, Self::DragDot | Self::Anchored | Self::Line)
    }

    /// True when the engine emits a dab at the down point on stroke begin (the continuous methods).
    /// The interactive methods (Anchored/Line/Curve) paint on finalise, so begin only anchors.
    #[must_use]
    pub fn emits_on_begin(self) -> bool {
        matches!(self, Self::Space | Self::Dots | Self::DragDot | Self::Airbrush)
    }

    /// True when per-dab position jitter applies (Blender disables it for DRAG_DOT/ANCHORED).
    #[must_use]
    pub fn allows_jitter(self) -> bool {
        !matches!(self, Self::DragDot | Self::Anchored)
    }
}

/// The unit the per-dab position jitter is measured in — Blender's `BRUSH_ABSOLUTE_JITTER` flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum JitterUnit {
    /// Relative to brush size: max radial offset ≈ `jitter × diameter`. Flag OFF (Blender "Brush").
    #[default]
    Brush,
    /// Absolute view pixels: max radial offset ≈ `2 × jitter_absolute_px`. Flag ON (Blender "View").
    View,
}

impl JitterUnit {
    /// Wire discriminant (`Brush` = 0, `View` = 1).
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Brush => 0,
            Self::View => 1,
        }
    }

    /// Decode a wire discriminant; unknown values fall back to [`JitterUnit::Brush`] (the default).
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::View,
            _ => Self::Brush,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stroke_method_wire_roundtrips_all_variants() {
        for m in [
            StrokeMethod::Dots,
            StrokeMethod::Airbrush,
            StrokeMethod::Anchored,
            StrokeMethod::Space,
            StrokeMethod::DragDot,
            StrokeMethod::Line,
            StrokeMethod::Curve,
        ] {
            assert_eq!(StrokeMethod::from_u8(m.to_u8()), m);
        }
        // Blender enum values are the wire contract.
        assert_eq!(StrokeMethod::Space.to_u8(), 3);
        assert_eq!(StrokeMethod::default(), StrokeMethod::Space);
        // Unknown → Space (paints), not Dots.
        assert_eq!(StrokeMethod::from_u8(200), StrokeMethod::Space);
    }

    #[test]
    fn stroke_panel_visibility_matches_blender() {
        use StrokeMethod::{Airbrush, Anchored, Curve, DragDot, Dots, Line, Space};
        // The Blender "Stroke" panel row matrix (Spacing/Dash, Jitter) per method. Input
        // Samples is always shown, so it is not in the table. This locks the per-method gate
        // the layers panel paints against — a predicate edit that breaks parity goes red here,
        // not in a human smoke. Reference: paint_stroke.cc dispatch + DNA_brush_enums flags.
        let rows = [
            //         spacing  dash   jitter stabilizer
            (Dots, false, false, true, true),
            (Airbrush, false, false, true, true),
            (Anchored, false, false, false, false),
            (Space, true, true, true, true),
            (DragDot, false, false, false, false),
            (Line, true, true, true, false),
            (Curve, true, true, true, true),
        ];
        for (m, spacing, dash, jitter, stabilizer) in rows {
            assert_eq!(m.uses_spacing(), spacing, "{m:?} Spacing visibility");
            assert_eq!(m.uses_dash(), dash, "{m:?} Dash visibility");
            assert_eq!(m.allows_jitter(), jitter, "{m:?} Jitter visibility");
            assert_eq!(m.uses_stabilizer(), stabilizer, "{m:?} Stabilizer visibility");
        }
    }

    #[test]
    fn jitter_unit_wire_roundtrips() {
        assert_eq!(JitterUnit::from_u8(JitterUnit::Brush.to_u8()), JitterUnit::Brush);
        assert_eq!(JitterUnit::from_u8(JitterUnit::View.to_u8()), JitterUnit::View);
        assert_eq!(JitterUnit::default(), JitterUnit::Brush);
        assert_eq!(JitterUnit::from_u8(9), JitterUnit::Brush);
    }
}
