//! [`VectorPencilTool`] — freehand Pencil authoring with Hobby auto-smooth.
//!
//! Per plan **T2.1** (`docs/Vector Module/17_plano_de_implementacao.md` §5).
//! Stateful `Tool` with inherent `begin_stroke` / `extend_stroke` /
//! `finish_stroke` / `cancel_stroke` methods the shell bridge calls after
//! downcasting via [`Tool::as_any_mut`] (ADR-0040 §3 documented exception,
//! mirror of [`ph2d_tool_vector_pen::VectorPenTool::on_canvas_click`] and
//! Painter `PainterTool::queue_pointer`).
//!
//! ## Pipeline (commit on pointer-up)
//!
//! 1. **Record** raw [`StrokeSample`]s during the drag (a near-duplicate
//!    distance filter drops jitter as they arrive).
//! 2. **Decimate** to knots at ≈ 1 knot per [`DEFAULT_DECIMATION_STRIDE`]
//!    samples (the plan DoD: "1 cubic per 10 input samples").
//! 3. **Fit** a smooth interpolating cubic spline through the knots via
//!    [`ph2d_vector_doc::hobby::fit_hobby_open`] (Hobby's algorithm, NOT
//!    Schneider — the plan anti-pattern).
//! 4. **Emit** an open, stroked [`Ph2dVectorAsset`]: one vertex per knot,
//!    one cubic segment per interval, each segment carrying the seeded
//!    stroke style. No region (a Pencil stroke is an open path, unlike the
//!    Pen which closes into a filled region).
//!
//! ## Pressure / tilt / azimuth
//!
//! [`StrokeSample`] records pressure / tilt / azimuth **hooks** so the
//! shell input layer can plumb them today, but W2 stroking is
//! constant-width: the real pressure→width / tilt→envelope mapping lands
//! in **W5** (GPU stroke expansion). Recording them now keeps the data
//! path ready without a premature `ph2d-input` dependency.

use glam::Vec2;
use ph2d_editor_core::floating_panel::{FloatingPanel, ToolId};
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_vector_doc::{
    EditLog, Ph2dVectorAsset, StrokeCap, StrokeJoin, StrokeStyle, StyleRef, StyleTable,
    VectorNetwork, VectorOp, VertexKind, WidthProfile, hobby,
};

/// A knot's pressure within this of `1.0` counts as "no pressure" (mouse /
/// trackpad). Below the whole stroke that holds, the W2 constant-width path is
/// kept (one shared style, no per-segment profiles).
const PRESSURE_EPS: f32 = 1.0e-3;

/// Raw-sample count per emitted cubic (plan DoD: "1 cubic per 10 input
/// samples"). The decimator keeps every `stride`-th filtered sample plus
/// the first and last.
pub const DEFAULT_DECIMATION_STRIDE: usize = 10;

/// Default world-space distance under which a freshly-arrived sample is
/// dropped as a near-duplicate (pointer jitter / a paused pen spamming the
/// same coordinate). Keeps the raw recording from ballooning and prevents
/// zero-length chords reaching the Hobby fitter.
pub const DEFAULT_MIN_SAMPLE_DISTANCE_PX: f32 = 1.5;

/// Defensive cap on the raw sample buffer for one stroke. A 16 k-sample
/// stroke at ~120 Hz is > 2 minutes of continuous drawing — far beyond any
/// real gesture, but bounds memory if the input layer misbehaves.
pub const MAX_STROKE_SAMPLES: usize = 16_384;

/// Maximum absolute world-coordinate magnitude a sample may carry. A
/// pointer projected absurdly far off-canvas under extreme zoom is finite
/// but degenerate once serialized (mirror of the Pen tool's `MAX_COORD_MAGNITUDE`).
pub const MAX_COORD_MAGNITUDE: f32 = 1.0e7;

/// Default stroke width (network-local px) for a committed Pencil path.
/// Constant in W2; variable-width via pressure arrives W5.
pub const DEFAULT_STROKE_WIDTH_PX: f32 = 2.0;

/// One recorded point of a freehand stroke.
///
/// `pressure` / `tilt` / `azimuth` are recorded hooks (see module docs);
/// W2 geometry uses only `pos`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrokeSample {
    /// Position in network-local (world) coordinates.
    pub pos: Vec2,
    /// Normalized pen pressure `0.0..=1.0` (1.0 when the device has no
    /// pressure axis). W5 maps this to stroke width.
    pub pressure: f32,
    /// Pen tilt from vertical, radians. W5 maps this to an asymmetric
    /// stroke envelope.
    pub tilt: f32,
    /// Pen azimuth (bearing), radians. Recorded for W5.
    pub azimuth: f32,
}

impl StrokeSample {
    /// A sample with default device axes (full pressure, no tilt) — the
    /// mouse / trackpad case.
    #[must_use]
    pub fn point(pos: Vec2) -> Self {
        Self {
            pos,
            pressure: 1.0,
            tilt: 0.0,
            azimuth: 0.0,
        }
    }
}

/// Stateful Vector Pencil tool.
#[derive(Debug, Clone)]
pub struct VectorPencilTool {
    /// Raw stroke samples for the in-progress gesture. Empty when idle.
    samples: Vec<StrokeSample>,

    /// `true` between [`Self::begin_stroke`] and [`Self::finish_stroke`] /
    /// [`Self::cancel_stroke`].
    recording: bool,

    /// Document style table, pre-seeded with one default stroke at
    /// `default_stroke_ref`.
    styles: StyleTable,

    /// `StyleRef` assigned to every segment of a committed path.
    default_stroke_ref: StyleRef,

    /// Asset emitted by the most recent [`Self::finish_stroke`], awaiting
    /// the bridge's drain. `Some` for exactly one
    /// [`Self::take_committed_asset`] call.
    pending_committed: Option<Ph2dVectorAsset>,

    /// Raw samples per emitted cubic (decimation stride). Public so a
    /// future Tool Studio (W15) can tune Pencil smoothness.
    pub decimation_stride: usize,

    /// World-space near-duplicate filter distance.
    pub min_sample_distance_px: f32,
}

impl Default for VectorPencilTool {
    fn default() -> Self {
        let mut styles = StyleTable::default();
        // Visible default stroke (bluish OKLCH, readable on dark + light
        // canvas chrome — same rationale as the Pen tool's seeded fill).
        // `StrokeStyle` is `#[non_exhaustive]`, so build via `Default` +
        // field assignment rather than a struct literal.
        let mut stroke = StrokeStyle::default();
        stroke.width = DEFAULT_STROKE_WIDTH_PX;
        stroke.color = ph2d_color::OklchColor::opaque(0.6, 0.18, 250.0);
        stroke.cap = StrokeCap::Round;
        stroke.join = StrokeJoin::Round;
        let default_stroke_ref = styles.insert_stroke(stroke);
        Self {
            samples: Vec::new(),
            recording: false,
            styles,
            default_stroke_ref,
            pending_committed: None,
            decimation_stride: DEFAULT_DECIMATION_STRIDE,
            min_sample_distance_px: DEFAULT_MIN_SAMPLE_DISTANCE_PX,
        }
    }
}

impl VectorPencilTool {
    /// Construct a fresh Pencil tool with default state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Read-only view of the in-progress raw samples — the shell bridge
    /// reads this per frame to render the live (un-smoothed) preview
    /// polyline while the user is dragging.
    #[must_use]
    pub fn current_samples(&self) -> &[StrokeSample] {
        &self.samples
    }

    /// Read-only handle on the document style table (the bridge resolves
    /// `segment.style_ref` against this when stroking committed paths).
    #[must_use]
    pub fn current_styles(&self) -> &StyleTable {
        &self.styles
    }

    /// `true` while a stroke is being recorded.
    #[must_use]
    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// `true` if there are unsaved in-progress samples (used by the bridge
    /// to warn before a destructive tool switch — mirror of the Pen's
    /// `has_in_progress_path`).
    #[must_use]
    pub fn has_in_progress_stroke(&self) -> bool {
        !self.samples.is_empty()
    }

    /// Number of raw samples recorded so far.
    #[must_use]
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Drain the most-recently-committed asset, if any. Returns `Some`
    /// exactly once per finished stroke.
    pub fn take_committed_asset(&mut self) -> Option<Ph2dVectorAsset> {
        self.pending_committed.take()
    }

    /// Replace the default stroke color + width. The shell bridge calls
    /// this on activation / after a Color Picker change so committed paths
    /// use the user's chosen stroke (mirror of the Pen's `set_default_fill`).
    pub fn set_default_stroke(&mut self, color: ph2d_color::OklchColor, width: f32) {
        if let Some(slot) = self.styles.strokes.get_mut(&self.default_stroke_ref) {
            slot.color = color;
            slot.width = width.max(0.0);
        }
    }

    /// Begin recording a new stroke at `sample`. Discards any prior
    /// in-progress samples (a new gesture supersedes an abandoned one).
    /// Rejects a non-finite / out-of-range first point (returns
    /// [`PencilStrokeOutcome::Rejected`] and stays idle).
    pub fn begin_stroke(&mut self, sample: StrokeSample) -> PencilStrokeOutcome {
        if !Self::sample_is_sane(sample) {
            self.samples.clear();
            self.recording = false;
            return PencilStrokeOutcome::Rejected;
        }
        self.samples.clear();
        self.samples.push(sample);
        self.recording = true;
        PencilStrokeOutcome::Started
    }

    /// Append `sample` to the in-progress stroke. No-op (returns
    /// [`PencilStrokeOutcome::Ignored`]) if not recording, if the sample is
    /// within [`Self::min_sample_distance_px`] of the previous one, or if
    /// the buffer is at [`MAX_STROKE_SAMPLES`]. Rejects non-finite /
    /// out-of-range coords.
    pub fn extend_stroke(&mut self, sample: StrokeSample) -> PencilStrokeOutcome {
        if !self.recording {
            return PencilStrokeOutcome::Ignored;
        }
        if !Self::sample_is_sane(sample) {
            return PencilStrokeOutcome::Rejected;
        }
        if self.samples.len() >= MAX_STROKE_SAMPLES {
            return PencilStrokeOutcome::Ignored;
        }
        if let Some(last) = self.samples.last()
            && (sample.pos - last.pos).length() < self.min_sample_distance_px
        {
            return PencilStrokeOutcome::Ignored;
        }
        self.samples.push(sample);
        PencilStrokeOutcome::Extended
    }

    /// Finish the stroke: decimate → Hobby fit → emit the committed asset.
    /// Returns [`PencilStrokeOutcome::Committed`] on success, or
    /// [`PencilStrokeOutcome::TooShort`] if the gesture produced fewer than
    /// two distinct knots (a tap — no path). Always returns to the idle
    /// state.
    pub fn finish_stroke(&mut self) -> PencilStrokeOutcome {
        self.recording = false;
        let knots = decimate(
            &self.samples,
            self.decimation_stride,
            self.min_sample_distance_px,
        );
        self.samples.clear();
        if knots.len() < 2 {
            return PencilStrokeOutcome::TooShort;
        }
        let positions: Vec<Vec2> = knots.iter().map(|k| k.pos).collect();
        let tangents = hobby::fit_hobby_open(&positions);

        let mut network = VectorNetwork::empty();
        let mut edit_log = EditLog::new();

        for (i, knot) in knots.iter().enumerate() {
            // push_and_apply only fails on context-needing ops; AddVertex
            // is always network-applicable, so this cannot error here.
            let _ = edit_log.push_and_apply(
                VectorOp::AddVertex {
                    id: i as u32,
                    pos: knot.pos,
                    kind: VertexKind::Auto,
                },
                &mut network,
            );
        }
        for (i, tan) in tangents.iter().enumerate() {
            let _ = edit_log.push_and_apply(
                VectorOp::AddSegment {
                    id: i as u32,
                    start: i as u32,
                    end: (i + 1) as u32,
                    tangents: *tan,
                },
                &mut network,
            );
        }

        // Materialize the stroke style on each segment + a FRESH per-asset
        // `StyleTable` (the tool's table is the editing template; the committed
        // asset carries only its own styles). `SetStrokeStyle` needs asset-level
        // apply context (absent until T2.5), so the assignment lives in the
        // serialized network/styles, not the (network-applicable) edit_log —
        // the log stays pure geometry for replay-based undo.
        //
        // **W5 pressure → variable width:** when the device reported real
        // pressure variation, give every SEGMENT its own style carrying a
        // `WidthProfile` tapering between its two knots' pressures — the renderer
        // (`draw_vector_network`) expands `width_profile` to variable width
        // automatically, no baked geometry. Constant pressure (mouse / trackpad,
        // all `1.0`) keeps one shared style = the W2 constant-width path.
        let base = self
            .styles
            .strokes
            .get(&self.default_stroke_ref)
            .copied()
            .unwrap_or_default();
        let mut asset_styles = StyleTable::default();
        let has_pressure = knots
            .iter()
            .any(|k| (k.pressure - 1.0).abs() > PRESSURE_EPS);
        if has_pressure {
            for (i, seg) in network.segments.iter_mut().enumerate() {
                let mut style = base;
                style.width_profile = Some(WidthProfile {
                    start: knots[i].pressure.max(0.0),
                    end: knots[i + 1].pressure.max(0.0),
                    bulge: 0.0,
                });
                seg.style_ref = Some(asset_styles.insert_stroke(style));
            }
        } else {
            let stroke_ref = asset_styles.insert_stroke(base);
            for seg in network.segments.iter_mut() {
                seg.style_ref = Some(stroke_ref);
            }
        }

        let mut asset = Ph2dVectorAsset::from_network(network, asset_styles);
        asset.edit_log = edit_log;
        self.pending_committed = Some(asset);
        PencilStrokeOutcome::Committed
    }

    /// Abandon the in-progress stroke without committing (Esc / cancel).
    pub fn cancel_stroke(&mut self) {
        self.samples.clear();
        self.recording = false;
    }

    /// Reset all volatile state. Preserves the seeded default stroke.
    pub fn reset(&mut self) {
        self.samples.clear();
        self.recording = false;
        // `pending_committed` is intentionally NOT cleared here — see
        // `on_deactivate`. `reset` is the idle-state reset; callers that
        // also want to drop a pending commit drain it first.
    }

    /// `pos` finite and within [`MAX_COORD_MAGNITUDE`].
    fn sample_is_sane(s: StrokeSample) -> bool {
        s.pos.x.is_finite()
            && s.pos.y.is_finite()
            && s.pos.x.abs() <= MAX_COORD_MAGNITUDE
            && s.pos.y.abs() <= MAX_COORD_MAGNITUDE
    }
}

/// Decimate raw samples to interpolation knots.
///
/// 1. Drop consecutive near-duplicates (`< min_dist` apart) — also
///    guarantees no zero-length chord reaches the Hobby fitter.
/// 2. Keep every `stride`-th survivor plus the first and last point, so
///    the knot count is ≈ `survivors / stride` (→ ≈ 1 cubic per `stride`
///    raw samples, the plan DoD).
fn decimate(samples: &[StrokeSample], stride: usize, min_dist: f32) -> Vec<StrokeSample> {
    let mut filtered: Vec<StrokeSample> = Vec::new();
    for &s in samples {
        match filtered.last() {
            Some(last) if (s.pos - last.pos).length() < min_dist => {}
            _ => filtered.push(s),
        }
    }
    if filtered.len() <= 2 {
        return filtered;
    }
    let stride = stride.max(1);
    let last_idx = filtered.len() - 1;
    let mut knots: Vec<StrokeSample> = Vec::with_capacity(filtered.len() / stride + 2);
    knots.push(filtered[0]);
    let mut i = stride;
    while i < last_idx {
        knots.push(filtered[i]);
        i += stride;
    }
    knots.push(filtered[last_idx]);
    knots
}

/// Outcome reported by the Pencil pointer methods so the shell bridge can
/// drive preview invalidation / commit handling / toast feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PencilStrokeOutcome {
    /// A new stroke started ([`VectorPencilTool::begin_stroke`]).
    Started,
    /// A sample was appended ([`VectorPencilTool::extend_stroke`]).
    Extended,
    /// Stroke finished and a smoothed asset was committed; drain via
    /// [`VectorPencilTool::take_committed_asset`].
    Committed,
    /// Stroke finished but produced fewer than two distinct knots (a tap)
    /// — nothing committed.
    TooShort,
    /// Sample ignored (not recording, too close to the previous, or buffer
    /// full). No mutation.
    Ignored,
    /// Sample rejected (non-finite or out-of-range coordinate).
    Rejected,
}

impl Tool for VectorPencilTool {
    fn id(&self) -> ToolId {
        ToolId::new("vector_pencil")
    }

    fn label(&self) -> &str {
        "Vector Pencil"
    }

    fn icon_slug(&self) -> &str {
        // Hyphen for SVG/Lucide convention; id stays snake_case for the
        // HR-15 i18n key (mirror of the Pen's id/slug split).
        "vector-pencil"
    }

    fn build_panel(&self) -> FloatingPanel {
        // W2: canvas-driven; the panel is title-only. Tool Studio (W15)
        // installs smoothness / stroke-width sliders.
        FloatingPanel::new(self.id(), "Vector Pencil")
    }

    fn on_activate(&mut self) {
        self.reset();
    }

    fn on_deactivate(&mut self) {
        // Drop the in-progress stroke but PRESERVE `pending_committed`:
        // that asset was already commit-emitted by the user's pointer-up
        // and is owed to the bridge; clearing it would silently delete
        // user data if the bridge hasn't drained yet (mirror of the Pen's
        // R1 Lens-M HIGH-L-M-2 fix). The bridge drains post-deactivate.
        self.reset();
    }

    fn handle_panel_event(&mut self, _event: PanelEvent) {
        // No panel widgets in W2.
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drag(tool: &mut VectorPencilTool, pts: &[Vec2]) {
        tool.begin_stroke(StrokeSample::point(pts[0]));
        for &p in &pts[1..] {
            tool.extend_stroke(StrokeSample::point(p));
        }
    }

    #[test]
    fn fresh_tool_is_idle_with_one_seeded_stroke() {
        let t = VectorPencilTool::new();
        assert!(!t.is_recording());
        assert!(!t.has_in_progress_stroke());
        assert_eq!(t.current_styles().strokes.len(), 1);
    }

    #[test]
    fn begin_then_extend_records_samples() {
        let mut t = VectorPencilTool::new();
        assert_eq!(
            t.begin_stroke(StrokeSample::point(Vec2::ZERO)),
            PencilStrokeOutcome::Started
        );
        assert!(t.is_recording());
        assert_eq!(
            t.extend_stroke(StrokeSample::point(Vec2::new(50.0, 0.0))),
            PencilStrokeOutcome::Extended
        );
        assert_eq!(t.sample_count(), 2);
    }

    #[test]
    fn near_duplicate_samples_are_filtered() {
        let mut t = VectorPencilTool::new();
        t.begin_stroke(StrokeSample::point(Vec2::ZERO));
        // Within min distance → ignored.
        let out = t.extend_stroke(StrokeSample::point(Vec2::new(0.1, 0.0)));
        assert_eq!(out, PencilStrokeOutcome::Ignored);
        assert_eq!(t.sample_count(), 1);
    }

    #[test]
    fn extend_without_begin_is_ignored() {
        let mut t = VectorPencilTool::new();
        assert_eq!(
            t.extend_stroke(StrokeSample::point(Vec2::new(5.0, 5.0))),
            PencilStrokeOutcome::Ignored
        );
    }

    #[test]
    fn finish_commits_a_smoothed_open_stroked_path() {
        let mut t = VectorPencilTool::new();
        // A 25-sample wavy stroke → decimates to a few knots → smoothed
        // open path.
        let pts: Vec<Vec2> = (0..25)
            .map(|i| Vec2::new(i as f32 * 6.0, ((i as f32) * 0.5).sin() * 20.0))
            .collect();
        drag(&mut t, &pts);
        let out = t.finish_stroke();
        assert_eq!(out, PencilStrokeOutcome::Committed);
        assert!(!t.is_recording());
        assert!(!t.has_in_progress_stroke(), "samples cleared after commit");

        let asset = t.take_committed_asset().expect("committed asset");
        // Open path: vertices, segments, NO region.
        assert!(asset.network.vertices.len() >= 2);
        assert_eq!(
            asset.network.segments.len(),
            asset.network.vertices.len() - 1,
            "open chain: one fewer segment than vertices"
        );
        assert!(asset.network.regions.is_empty(), "Pencil emits no region");
        // Every segment carries the seeded stroke style.
        for seg in &asset.network.segments {
            assert_eq!(seg.style_ref, Some(0));
        }
        assert_eq!(asset.styles.strokes.len(), 1);
        assert!(asset.network.validate().is_ok());
    }

    #[test]
    fn one_cubic_per_stride_samples_dod() {
        // Plan DoD: ~1 cubic per `stride` input samples. 100 collinear
        // samples at stride 10 → ~10 cubics (segments).
        let mut t = VectorPencilTool::new();
        let pts: Vec<Vec2> = (0..100).map(|i| Vec2::new(i as f32 * 5.0, 0.0)).collect();
        drag(&mut t, &pts);
        t.finish_stroke();
        let asset = t.take_committed_asset().expect("committed");
        let segs = asset.network.segments.len();
        assert!(
            (9..=12).contains(&segs),
            "expected ~10 cubics for 100 samples @ stride 10, got {segs}"
        );
    }

    #[test]
    fn committed_edit_log_is_pure_geometry_and_replayable() {
        // edit_log holds only network-applicable AddVertex + AddSegment ops
        // (no SetStrokeStyle), so a from-genesis replay reconstructs the
        // geometry exactly — the foundation T2.5 undo builds on.
        let mut t = VectorPencilTool::new();
        let pts: Vec<Vec2> = (0..30)
            .map(|i| Vec2::new(i as f32 * 4.0, (i % 3) as f32 * 8.0))
            .collect();
        drag(&mut t, &pts);
        t.finish_stroke();
        let asset = t.take_committed_asset().expect("committed");
        let v = asset.network.vertices.len();
        let s = asset.network.segments.len();
        assert_eq!(asset.edit_log.ops.len(), v + s);
        // Replay from genesis reproduces the same vertex/segment counts.
        let mut replay = VectorNetwork::empty();
        for op in &asset.edit_log.ops {
            assert!(
                op.apply_to_network(&mut replay).is_ok(),
                "non-applicable op in log"
            );
        }
        assert_eq!(replay.vertices.len(), v);
        assert_eq!(replay.segments.len(), s);
    }

    #[test]
    fn finish_with_a_tap_commits_nothing() {
        let mut t = VectorPencilTool::new();
        t.begin_stroke(StrokeSample::point(Vec2::new(10.0, 10.0)));
        // No movement → single sample → TooShort.
        assert_eq!(t.finish_stroke(), PencilStrokeOutcome::TooShort);
        assert!(t.take_committed_asset().is_none());
    }

    #[test]
    fn cancel_drops_in_progress_without_commit() {
        let mut t = VectorPencilTool::new();
        drag(
            &mut t,
            &[Vec2::ZERO, Vec2::new(50.0, 0.0), Vec2::new(80.0, 30.0)],
        );
        t.cancel_stroke();
        assert!(!t.has_in_progress_stroke());
        assert!(!t.is_recording());
        assert!(t.take_committed_asset().is_none());
    }

    #[test]
    fn nan_first_point_is_rejected_and_stays_idle() {
        let mut t = VectorPencilTool::new();
        let out = t.begin_stroke(StrokeSample::point(Vec2::new(f32::NAN, 0.0)));
        assert_eq!(out, PencilStrokeOutcome::Rejected);
        assert!(!t.is_recording());
        assert_eq!(t.sample_count(), 0);
    }

    #[test]
    fn nan_extend_is_rejected_without_corrupting_stroke() {
        let mut t = VectorPencilTool::new();
        t.begin_stroke(StrokeSample::point(Vec2::ZERO));
        t.extend_stroke(StrokeSample::point(Vec2::new(50.0, 0.0)));
        let before = t.sample_count();
        assert_eq!(
            t.extend_stroke(StrokeSample::point(Vec2::new(0.0, f32::INFINITY))),
            PencilStrokeOutcome::Rejected
        );
        assert_eq!(t.sample_count(), before);
    }

    #[test]
    fn on_deactivate_preserves_pending_committed() {
        let mut t = VectorPencilTool::new();
        let pts: Vec<Vec2> = (0..20).map(|i| Vec2::new(i as f32 * 5.0, 0.0)).collect();
        drag(&mut t, &pts);
        t.finish_stroke();
        <VectorPencilTool as Tool>::on_deactivate(&mut t);
        assert!(
            t.take_committed_asset().is_some(),
            "pending_committed must survive on_deactivate for the bridge to drain"
        );
    }

    #[test]
    fn set_default_stroke_changes_committed_color_and_width() {
        use ph2d_color::OklchColor;
        let mut t = VectorPencilTool::new();
        let custom = OklchColor::opaque(0.7, 0.2, 30.0);
        t.set_default_stroke(custom, 5.0);
        let pts: Vec<Vec2> = (0..20).map(|i| Vec2::new(i as f32 * 5.0, 0.0)).collect();
        drag(&mut t, &pts);
        t.finish_stroke();
        let asset = t.take_committed_asset().expect("committed");
        let stroke = asset.styles.strokes.values().next().expect("a stroke");
        assert_eq!(stroke.color, custom);
        assert_eq!(stroke.width, 5.0);
    }

    #[test]
    fn constant_pressure_keeps_one_shared_constant_width_style() {
        // Mouse/trackpad (pressure 1.0 everywhere) → one shared style, no
        // width_profile (the W2 constant-width path is preserved).
        let mut t = VectorPencilTool::new();
        let pts: Vec<Vec2> = (0..30).map(|i| Vec2::new(i as f32 * 5.0, 0.0)).collect();
        drag(&mut t, &pts);
        t.finish_stroke();
        let asset = t.take_committed_asset().expect("committed");
        assert_eq!(asset.styles.strokes.len(), 1, "one shared style");
        let style = asset.styles.strokes.values().next().unwrap();
        assert!(
            style.width_profile.is_none(),
            "constant pressure = no profile"
        );
        for seg in &asset.network.segments {
            assert_eq!(seg.style_ref, Some(0));
        }
    }

    #[test]
    fn pressure_variation_assigns_per_segment_width_profiles() {
        // A real pressure ramp 0.2 → 1.0 → each segment gets its own style with
        // a WidthProfile tapering between its endpoints' pressures.
        let mut t = VectorPencilTool::new();
        t.begin_stroke(StrokeSample {
            pos: Vec2::ZERO,
            pressure: 0.2,
            tilt: 0.0,
            azimuth: 0.0,
        });
        for i in 1..30 {
            let p = 0.2 + 0.8 * (i as f32 / 29.0);
            t.extend_stroke(StrokeSample {
                pos: Vec2::new(i as f32 * 5.0, 0.0),
                pressure: p,
                tilt: 0.0,
                azimuth: 0.0,
            });
        }
        t.finish_stroke();
        let asset = t.take_committed_asset().expect("committed");

        let n = asset.network.segments.len();
        assert_eq!(
            asset.styles.strokes.len(),
            n,
            "one width-profile style per segment"
        );
        for seg in &asset.network.segments {
            let style = asset
                .styles
                .strokes
                .get(&seg.style_ref.expect("a style ref"))
                .expect("style resolves");
            assert!(
                style.width_profile.is_some(),
                "pressure stroke → width_profile"
            );
        }
        // Pressure rises along the stroke → the last segment is wider than the first.
        let prof = |seg_idx: usize| {
            let r = asset.network.segments[seg_idx].style_ref.unwrap();
            asset.styles.strokes.get(&r).unwrap().width_profile.unwrap()
        };
        assert!(
            prof(0).start < prof(n - 1).end,
            "rising pressure → wider toward the end"
        );
        assert!(asset.network.validate().is_ok());
    }

    #[test]
    fn second_stroke_after_commit_starts_fresh() {
        let mut t = VectorPencilTool::new();
        let pts: Vec<Vec2> = (0..20).map(|i| Vec2::new(i as f32 * 5.0, 0.0)).collect();
        drag(&mut t, &pts);
        t.finish_stroke();
        assert!(t.take_committed_asset().is_some());
        // Begin again: clean slate.
        assert_eq!(
            t.begin_stroke(StrokeSample::point(Vec2::new(200.0, 200.0))),
            PencilStrokeOutcome::Started
        );
        assert_eq!(t.sample_count(), 1);
    }

    #[test]
    fn tool_id_label_icon_are_stable() {
        let t = VectorPencilTool::new();
        assert_eq!(t.id(), ToolId::new("vector_pencil"));
        assert_eq!(t.label(), "Vector Pencil");
        assert_eq!(t.icon_slug(), "vector-pencil");
    }
}
