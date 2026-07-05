//! Stroke method + jitter unit — the discrete options of the Blender "Stroke" panel.
//!
//! Behavioural reference (clean-room, no code copied): Blender
//! `editors/sculpt_paint/paint_stroke.cc` (the `eBrushStrokeType` dispatch in `PaintStroke::modal`)
//! and `makesdna/DNA_brush_enums.h` (the enum and the `BRUSH_ABSOLUTE_JITTER` flag). The wire
//! discriminants returned by [`StrokeMethod::to_u8`] mirror Blender's `eBrushStrokeType` numeric
//! values (Dots=0 … Arc=6) so the cross-crate encoding is a documented, stable anchor.

/// How a pointer path is turned into dab positions — Blender's `eBrushStrokeType`.
///
/// Only [`StrokeMethod::Space`] resamples the path at fixed arc-length intervals; the per-event
/// methods ([`Dots`](Self::Dots)/[`Airbrush`](Self::Airbrush)/[`DragDot`](Self::DragDot)) emit one
/// dab per processed sample. [`Anchored`](Self::Anchored)/[`Line`](Self::Line)/
/// [`Arc`](Self::Arc) are *interactive* (finalise on release / Bézier authoring) and the
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
    /// **Arc** — a Bézier paint-curve filled with spaced dabs, stamped on finalise; created with a
    /// subtle initial bow (an arc, not a straight line). Blender's `BRUSH_STROKE_CURVE` (6).
    Arc,
    /// **PH2D extension (beyond Blender's `eBrushStrokeType`):** an editable ellipse — drawn
    /// centre-out, then adjusted with 4 axis handles + a rotation handle, its perimeter filled with
    /// spaced dabs on finalise. Wire discriminant `7`.
    Ellipse,
    /// **PH2D extension:** an editable regular polygon inscribed in an ellipse — 4 axis handles +
    /// rotation + a handle to change the side count (3..12) + a centre handle. Wire discriminant `8`.
    Polygon,
    /// **PH2D extension:** **Free Hand** — paint a freehand path; on release the captured path is
    /// simplified into the same editable control-point curve as [`Self::Arc`] (handles for later
    /// manipulation / re-apply), its spine filled with spaced dabs on finalise. Wire discriminant `9`.
    FreeHand,
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
            Self::Arc => 6,
            Self::Ellipse => 7,
            Self::Polygon => 8,
            Self::FreeHand => 9,
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
            6 => Self::Arc,
            7 => Self::Ellipse,
            8 => Self::Polygon,
            9 => Self::FreeHand,
            _ => Self::Space,
        }
    }

    /// True when this method resamples the path at the brush spacing (only `Space`; `Line`/`Arc`
    /// also fill by spacing but on finalise, not interactively).
    #[must_use]
    pub fn is_spaced(self) -> bool {
        matches!(self, Self::Space)
    }

    /// Whether this method only ever shows the **latest** pointer position, so a host may coalesce a burst
    /// of raw pointer Moves into ONE delivery per frame with zero visible change.
    ///
    /// True for the restore + whole-shape re-stamp **fill** methods (Arc / Ellipse / Polygon / Line /
    /// Anchored / Drag Dot): each Move restores the previous footprint and re-stamps from scratch, so an
    /// intermediate Move's pixels are reverted by the next Move anyway — only the last one per frame
    /// survives to the preview drain. Coalescing them is byte-identical AND removes the per-event re-stamp
    /// storm (the FPS-drop / "Raw rises" path — `HANDOFF_per_layer_color_perf_artifacts` §1.R).
    ///
    /// False for the **incremental** methods (Space / Dots / Airbrush — each Move deposits permanent dabs
    /// along the path) and **Free Hand** (each Move captures a path sample): those must see every event or
    /// the stroke loses dabs / path points. They are cheap per Move regardless.
    #[must_use]
    pub fn coalesces_canvas_motion(self) -> bool {
        matches!(
            self,
            Self::Arc | Self::Ellipse | Self::Polygon | Self::Line | Self::Anchored | Self::DragDot
        )
    }

    /// True when the **Spacing** param (and its "Adjust Strength" attenuation toggle)
    /// is meaningful — the methods that lay dabs at a fixed arc-length interval
    /// (Blender: SPACE/LINE/CURVE). Same set as [`uses_dash`](Self::uses_dash) today
    /// (both controls are spacing-driven), but kept as its own predicate so the UI
    /// gate reads by intent rather than by coincidence, and the two can diverge.
    #[must_use]
    pub fn uses_spacing(self) -> bool {
        matches!(
            self,
            Self::Space | Self::Line | Self::Arc | Self::Ellipse | Self::Polygon | Self::FreeHand
        )
    }

    /// True when dabs are gated by the dash pattern (Blender: SPACE/LINE/CURVE; PH2D adds CIRCLE,
    /// whose perimeter is a spaced fill too).
    #[must_use]
    pub fn uses_dash(self) -> bool {
        matches!(
            self,
            Self::Space | Self::Line | Self::Arc | Self::Ellipse | Self::Polygon | Self::FreeHand
        )
    }

    /// True when the **Rate** param (the airbrush timer period) is meaningful — Blender shows it
    /// only for `AIRBRUSH`, the one method whose dabs are emitted on a timer rather than on input.
    #[must_use]
    pub fn uses_rate(self) -> bool {
        matches!(self, Self::Airbrush)
    }

    /// True when the **Edge to Edge** toggle is meaningful — Blender's `BRUSH_EDGE_TO_EDGE` only
    /// affects `ANCHORED` (whether the drag-sized stamp grows from the anchor or spans anchor→cursor).
    #[must_use]
    pub fn uses_edge_to_edge(self) -> bool {
        matches!(self, Self::Anchored)
    }

    /// True when the stroke **stabilizer** (the "how regular" knob) applies — the methods that build a
    /// path from raw freehand input: `Space`/`Dots`/`Airbrush` and **Free Hand** (whose capture lazy-mouse
    /// filters the cursor before the points are fitted into the curve). NOT Line/Arc: PH2D's Line is an
    /// explicit anchor→cursor segment and Arc is a point-editor (author control points, auto-smooth
    /// between them) — no shaky path to filter, so the stabilizer would be a no-op and the panel hides it
    /// (DIRETIVA §2: no silent no-op). Drag Dot/Anchored also place dabs at the exact cursor.
    #[must_use]
    pub fn uses_stabilizer(self) -> bool {
        matches!(
            self,
            Self::Space | Self::Dots | Self::Airbrush | Self::FreeHand
        )
    }

    /// True when this method forces pressure to 1.0 (Blender: DRAG_DOT, ANCHORED, LINE).
    #[must_use]
    pub fn forces_full_pressure(self) -> bool {
        matches!(self, Self::DragDot | Self::Anchored | Self::Line)
    }

    /// True when the engine emits a dab at the down point on stroke begin (the continuous methods).
    /// The interactive methods (Anchored/Line/Arc) paint on finalise, so begin only anchors.
    #[must_use]
    pub fn emits_on_begin(self) -> bool {
        matches!(
            self,
            Self::Space | Self::Dots | Self::DragDot | Self::Airbrush
        )
    }

    /// True when per-dab position jitter applies (Blender disables it for DRAG_DOT/ANCHORED).
    #[must_use]
    pub fn allows_jitter(self) -> bool {
        !matches!(self, Self::DragDot | Self::Anchored)
    }

    /// True for the freehand methods that drive the texture **Rake** warm-up: the stroke holds its
    /// opening dabs until travel defines a confident heading, then releases them at that angle (see
    /// [`crate::Stroke`]). Only the motion-driven painters — Airbrush is timer-driven (often parked, so
    /// it would stall), DragDot is a single follow-dab, and the shape methods fill a known geometry.
    #[must_use]
    pub fn rake_warmup_eligible(self) -> bool {
        matches!(self, Self::Space | Self::Dots)
    }

    /// True when the method drives a persistent on-canvas **shape editor** that the panel's Apply /
    /// Apply & Keep buttons (and Enter/Esc) act on: `Arc`/`FreeHand` (the point editor — Free Hand
    /// captures then edits the same curve), `Ellipse` (the ellipse editor), `Polygon` (the N-gon editor)
    /// and `Line` (the polyline editor — multi-click points, then editable). Drag Dot/Anchored finalise on
    /// pen-up with no editing session, so they get no Apply row (DIRETIVA §2: no dead control).
    #[must_use]
    pub fn has_open_shape(self) -> bool {
        matches!(
            self,
            Self::Arc | Self::FreeHand | Self::Ellipse | Self::Polygon | Self::Line
        )
    }

    /// True for the parametric shapes that the panel's **Edit** (E) button can convert into an editable
    /// Bézier curve — `Ellipse` and `Polygon`. (Arc / Free Hand are already curves, so they get no E.)
    #[must_use]
    pub fn is_convertible_shape(self) -> bool {
        matches!(self, Self::Ellipse | Self::Polygon)
    }

    /// True for the five interactive **shape** methods the tool rail's Shapes flyout selects
    /// (Line / Arc / Ellipse / Polygon / Free Hand), as opposed to the freehand/dab methods
    /// (Space / Dots / Airbrush / Anchored / Drag Dot). The rail's Brush button restores the last
    /// non-shape method; picking a shape in the flyout switches to one of these.
    #[must_use]
    pub fn is_shape(self) -> bool {
        matches!(
            self,
            Self::Line | Self::Arc | Self::Ellipse | Self::Polygon | Self::FreeHand
        )
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
            StrokeMethod::Arc,
            StrokeMethod::Ellipse,
            StrokeMethod::Polygon,
            StrokeMethod::FreeHand,
        ] {
            assert_eq!(StrokeMethod::from_u8(m.to_u8()), m);
        }
        // Blender enum values are the wire contract; Ellipse/Polygon/FreeHand are PH2D extensions (7/8/9).
        assert_eq!(StrokeMethod::Space.to_u8(), 3);
        assert_eq!(StrokeMethod::Ellipse.to_u8(), 7);
        assert_eq!(StrokeMethod::Polygon.to_u8(), 8);
        assert_eq!(StrokeMethod::FreeHand.to_u8(), 9);
        assert_eq!(StrokeMethod::default(), StrokeMethod::Space);
        // Unknown → Space (paints), not Dots.
        assert_eq!(StrokeMethod::from_u8(200), StrokeMethod::Space);
    }

    #[test]
    fn stroke_panel_visibility_matches_blender() {
        use StrokeMethod::{Airbrush, Anchored, Arc, Dots, DragDot, Ellipse, Line, Polygon, Space};
        // The Blender "Stroke" panel row matrix (Spacing/Dash, Jitter) per method. Input
        // Samples is always shown, so it is not in the table. This locks the per-method gate
        // the layers panel paints against — a predicate edit that breaks parity goes red here,
        // not in a human smoke. Reference: paint_stroke.cc dispatch + DNA_brush_enums flags.
        let rows = [
            //         spacing  dash   jitter stabilizer rate   edge
            (Dots, false, false, true, true, false, false),
            (Airbrush, false, false, true, true, true, false),
            (Anchored, false, false, false, false, false, true),
            (Space, true, true, true, true, false, false),
            (DragDot, false, false, false, false, false, false),
            (Line, true, true, true, false, false, false),
            (Arc, true, true, true, false, false, false),
            (Ellipse, true, true, true, false, false, false),
            (Polygon, true, true, true, false, false, false),
        ];
        for (m, spacing, dash, jitter, stabilizer, rate, edge) in rows {
            assert_eq!(m.uses_spacing(), spacing, "{m:?} Spacing visibility");
            assert_eq!(m.uses_dash(), dash, "{m:?} Dash visibility");
            assert_eq!(m.allows_jitter(), jitter, "{m:?} Jitter visibility");
            assert_eq!(
                m.uses_stabilizer(),
                stabilizer,
                "{m:?} Stabilizer visibility"
            );
            assert_eq!(m.uses_rate(), rate, "{m:?} Rate visibility");
            assert_eq!(m.uses_edge_to_edge(), edge, "{m:?} Edge-to-Edge visibility");
        }
    }

    #[test]
    fn has_open_shape_is_exactly_the_editor_methods() {
        // The Apply / Apply & Keep row shows for the persistent on-canvas shape editors only.
        for m in [
            StrokeMethod::Arc,
            StrokeMethod::FreeHand,
            StrokeMethod::Ellipse,
            StrokeMethod::Polygon,
            StrokeMethod::Line,
        ] {
            assert!(m.has_open_shape(), "{m:?} should have an open-shape editor");
        }
        for m in [
            StrokeMethod::Space,
            StrokeMethod::Dots,
            StrokeMethod::DragDot,
            StrokeMethod::Airbrush,
            StrokeMethod::Anchored,
        ] {
            assert!(
                !m.has_open_shape(),
                "{m:?} finalises on pen-up — no Apply row"
            );
        }
    }

    #[test]
    fn is_convertible_shape_is_circle_and_polygon_only() {
        assert!(StrokeMethod::Ellipse.is_convertible_shape());
        assert!(StrokeMethod::Polygon.is_convertible_shape());
        for m in [
            StrokeMethod::Arc,
            StrokeMethod::FreeHand,
            StrokeMethod::Line,
            StrokeMethod::Space,
        ] {
            assert!(
                !m.is_convertible_shape(),
                "{m:?} is not a convertible parametric shape"
            );
        }
    }

    #[test]
    fn jitter_unit_wire_roundtrips() {
        assert_eq!(
            JitterUnit::from_u8(JitterUnit::Brush.to_u8()),
            JitterUnit::Brush
        );
        assert_eq!(
            JitterUnit::from_u8(JitterUnit::View.to_u8()),
            JitterUnit::View
        );
        assert_eq!(JitterUnit::default(), JitterUnit::Brush);
        assert_eq!(JitterUnit::from_u8(9), JitterUnit::Brush);
    }
}
