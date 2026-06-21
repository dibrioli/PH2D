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

    /// True when dabs are gated by the dash pattern (Blender: SPACE/LINE/CURVE).
    #[must_use]
    pub fn uses_dash(self) -> bool {
        matches!(self, Self::Space | Self::Line | Self::Curve)
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

    /// True when the stabilize (smooth-stroke) spring applies (Blender disables it for
    /// ANCHORED/DRAG_DOT/LINE).
    #[must_use]
    pub fn supports_smooth(self) -> bool {
        !matches!(self, Self::Anchored | Self::DragDot | Self::Line)
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
    fn jitter_unit_wire_roundtrips() {
        assert_eq!(JitterUnit::from_u8(JitterUnit::Brush.to_u8()), JitterUnit::Brush);
        assert_eq!(JitterUnit::from_u8(JitterUnit::View.to_u8()), JitterUnit::View);
        assert_eq!(JitterUnit::default(), JitterUnit::Brush);
        assert_eq!(JitterUnit::from_u8(9), JitterUnit::Brush);
    }
}
