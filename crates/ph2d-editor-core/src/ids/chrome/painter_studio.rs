//! Brush Studio (W5) panel widget NodeIds (PAINTER_STUDIO_*).
use super::{NodeId, hash_node_id};

// ── Brush Studio (W5) — widget ids for `ph2d-panel-brush-studio` ──────────────
//
// The Brush Studio is the full brush-parameter editor (the sidebar carries only
// the live painting essentials). It shares the right-dock geometry slot. Live
// here in editor-core (single source of truth) so `PainterTool::handle_panel_event`
// can reference them without a tool→panel cycle (mirror of the sidebar ids).
//
// Float params are slider+chip pairs; bool params are checkboxes; enum params
// (grain type, rendering mode) are cycling buttons (the narrow-dock-friendly
// pattern Enio accepted for the sidebar grain control). All route through the
// generic `PainterUiEdit::SetBrushParam(BrushParam, f32)` so the cap stays low.

/// "Open Brush Studio" button in the brush sidebar header — flips the shared
/// right-dock slot to the Brush Studio (`PainterUiEdit::OpenBrushStudio`).
pub const PAINTER_SIDEBAR_BRUSH_STUDIO: NodeId = hash_node_id("painter_sidebar.brush_studio");
/// Close (X) button of the Brush Studio panel — returns the dock slot to the
/// brush sidebar (`PainterTool::close_brush_studio`).
pub const PAINTER_STUDIO_CLOSE: NodeId = hash_node_id("painter_studio.close");

// Section headers (collapsible).
pub const PAINTER_STUDIO_SEC_STROKE: NodeId = hash_node_id("painter_studio.sec_stroke");
pub const PAINTER_STUDIO_SEC_SHAPE: NodeId = hash_node_id("painter_studio.sec_shape");
pub const PAINTER_STUDIO_SEC_RENDERING: NodeId = hash_node_id("painter_studio.sec_rendering");
// Per-section "reset to default" buttons (one beside each Brush Studio subsection header).
pub const PAINTER_STUDIO_RESET_STROKE: NodeId = hash_node_id("painter_studio.reset_stroke");
pub const PAINTER_STUDIO_RESET_SHAPE: NodeId = hash_node_id("painter_studio.reset_shape");
pub const PAINTER_STUDIO_RESET_RENDERING: NodeId = hash_node_id("painter_studio.reset_rendering");
pub const PAINTER_STUDIO_RESET_COLOR: NodeId = hash_node_id("painter_studio.reset_color");
pub const PAINTER_STUDIO_RESET_DYNAMICS: NodeId = hash_node_id("painter_studio.reset_dynamics");

// ── Stroke Path section — float sliders (slider + editable chip) ──────────────
pub const PAINTER_STUDIO_SPACING_SLIDER: NodeId = hash_node_id("painter_studio.spacing_slider");
pub const PAINTER_STUDIO_SPACING_CHIP: NodeId = hash_node_id("painter_studio.spacing_chip");
pub const PAINTER_STUDIO_SPACING_JITTER_SLIDER: NodeId =
    hash_node_id("painter_studio.spacing_jitter_slider");
pub const PAINTER_STUDIO_SPACING_JITTER_CHIP: NodeId =
    hash_node_id("painter_studio.spacing_jitter_chip");
pub const PAINTER_STUDIO_JITTER_LATERAL_SLIDER: NodeId =
    hash_node_id("painter_studio.jitter_lateral_slider");
pub const PAINTER_STUDIO_JITTER_LATERAL_CHIP: NodeId =
    hash_node_id("painter_studio.jitter_lateral_chip");
pub const PAINTER_STUDIO_FALLOFF_SLIDER: NodeId = hash_node_id("painter_studio.falloff_slider");
pub const PAINTER_STUDIO_FALLOFF_CHIP: NodeId = hash_node_id("painter_studio.falloff_chip");
pub const PAINTER_STUDIO_TAPER_SLIDER: NodeId = hash_node_id("painter_studio.taper_slider");
pub const PAINTER_STUDIO_TAPER_CHIP: NodeId = hash_node_id("painter_studio.taper_chip");
pub const PAINTER_STUDIO_STREAMLINE_SLIDER: NodeId =
    hash_node_id("painter_studio.streamline_slider");
pub const PAINTER_STUDIO_STREAMLINE_CHIP: NodeId = hash_node_id("painter_studio.streamline_chip");
pub const PAINTER_STUDIO_STABILIZATION_SLIDER: NodeId =
    hash_node_id("painter_studio.stabilization_slider");
pub const PAINTER_STUDIO_STABILIZATION_CHIP: NodeId =
    hash_node_id("painter_studio.stabilization_chip");
/// One-Euro motion filtering (ADR-0077 D10): adaptive low-pass amount + the
/// speed-responsiveness ("expression") that keeps fast strokes lag-free.
pub const PAINTER_STUDIO_MOTION_FILTER_SLIDER: NodeId =
    hash_node_id("painter_studio.motion_filter_slider");
pub const PAINTER_STUDIO_MOTION_FILTER_CHIP: NodeId =
    hash_node_id("painter_studio.motion_filter_chip");
pub const PAINTER_STUDIO_MOTION_EXPR_SLIDER: NodeId =
    hash_node_id("painter_studio.motion_expr_slider");
pub const PAINTER_STUDIO_MOTION_EXPR_CHIP: NodeId = hash_node_id("painter_studio.motion_expr_chip");
/// Velocity dynamics (ADR-0077 D10): stroke speed → size / opacity / spacing.
/// Bipolar (−1..1): −1 = fast→less, +1 = fast→more.
pub const PAINTER_STUDIO_SPEED_SIZE_SLIDER: NodeId =
    hash_node_id("painter_studio.speed_size_slider");
pub const PAINTER_STUDIO_SPEED_SIZE_CHIP: NodeId = hash_node_id("painter_studio.speed_size_chip");
pub const PAINTER_STUDIO_SPEED_OPACITY_SLIDER: NodeId =
    hash_node_id("painter_studio.speed_opacity_slider");
pub const PAINTER_STUDIO_SPEED_OPACITY_CHIP: NodeId =
    hash_node_id("painter_studio.speed_opacity_chip");
pub const PAINTER_STUDIO_SPEED_SPACING_SLIDER: NodeId =
    hash_node_id("painter_studio.speed_spacing_slider");
pub const PAINTER_STUDIO_SPEED_SPACING_CHIP: NodeId =
    hash_node_id("painter_studio.speed_spacing_chip");

// ── Shape section — sliders + checkboxes ─────────────────────────────────────
pub const PAINTER_STUDIO_SHAPE_SCATTER_SLIDER: NodeId =
    hash_node_id("painter_studio.shape_scatter_slider");
pub const PAINTER_STUDIO_SHAPE_SCATTER_CHIP: NodeId =
    hash_node_id("painter_studio.shape_scatter_chip");
pub const PAINTER_STUDIO_SHAPE_COUNT_SLIDER: NodeId =
    hash_node_id("painter_studio.shape_count_slider");
pub const PAINTER_STUDIO_SHAPE_COUNT_CHIP: NodeId = hash_node_id("painter_studio.shape_count_chip");
pub const PAINTER_STUDIO_SHAPE_COUNT_JITTER_SLIDER: NodeId =
    hash_node_id("painter_studio.shape_count_jitter_slider");
pub const PAINTER_STUDIO_SHAPE_COUNT_JITTER_CHIP: NodeId =
    hash_node_id("painter_studio.shape_count_jitter_chip");
pub const PAINTER_STUDIO_SHAPE_ROUNDNESS_SLIDER: NodeId =
    hash_node_id("painter_studio.shape_roundness_slider");
pub const PAINTER_STUDIO_SHAPE_ROUNDNESS_CHIP: NodeId =
    hash_node_id("painter_studio.shape_roundness_chip");
// Roundness modulators (W2.11/W2.12/W2.13): pressure/tilt flatten the nib, jitter
// randomizes the squash per dab.
pub const PAINTER_STUDIO_SHAPE_ROUNDNESS_PRESSURE_SLIDER: NodeId =
    hash_node_id("painter_studio.shape_roundness_pressure_slider");
pub const PAINTER_STUDIO_SHAPE_ROUNDNESS_PRESSURE_CHIP: NodeId =
    hash_node_id("painter_studio.shape_roundness_pressure_chip");
pub const PAINTER_STUDIO_SHAPE_ROUNDNESS_TILT_SLIDER: NodeId =
    hash_node_id("painter_studio.shape_roundness_tilt_slider");
pub const PAINTER_STUDIO_SHAPE_ROUNDNESS_TILT_CHIP: NodeId =
    hash_node_id("painter_studio.shape_roundness_tilt_chip");
pub const PAINTER_STUDIO_SHAPE_ROUNDNESS_JITTER_SLIDER: NodeId =
    hash_node_id("painter_studio.shape_roundness_jitter_slider");
pub const PAINTER_STUDIO_SHAPE_ROUNDNESS_JITTER_CHIP: NodeId =
    hash_node_id("painter_studio.shape_roundness_jitter_chip");
// Base nib angle (W2.10): the calligraphic chisel's resting orientation.
pub const PAINTER_STUDIO_SHAPE_ANGLE_SLIDER: NodeId =
    hash_node_id("painter_studio.shape_angle_slider");
pub const PAINTER_STUDIO_SHAPE_ANGLE_CHIP: NodeId = hash_node_id("painter_studio.shape_angle_chip");
// Shape Filtering edge-AA mode cycler (W2.14): None / Classic / Improved.
pub const PAINTER_STUDIO_SHAPE_FILTERING: NodeId = hash_node_id("painter_studio.shape_filtering");
pub const PAINTER_STUDIO_SHAPE_ROTATION_FOLLOW: NodeId =
    hash_node_id("painter_studio.shape_rotation_follow");
pub const PAINTER_STUDIO_SHAPE_RANDOMIZED: NodeId = hash_node_id("painter_studio.shape_randomized");
pub const PAINTER_STUDIO_SHAPE_FLIP_X: NodeId = hash_node_id("painter_studio.shape_flip_x");
pub const PAINTER_STUDIO_SHAPE_FLIP_Y: NodeId = hash_node_id("painter_studio.shape_flip_y");

// ── Rendering section — sliders + checkboxes + cyclers ────────────────────────
pub const PAINTER_STUDIO_FLOW_SLIDER: NodeId = hash_node_id("painter_studio.flow_slider");
pub const PAINTER_STUDIO_FLOW_CHIP: NodeId = hash_node_id("painter_studio.flow_chip");
pub const PAINTER_STUDIO_ALPHA_THRESHOLD_SLIDER: NodeId =
    hash_node_id("painter_studio.alpha_threshold_slider");
pub const PAINTER_STUDIO_ALPHA_THRESHOLD_CHIP: NodeId =
    hash_node_id("painter_studio.alpha_threshold_chip");
pub const PAINTER_STUDIO_GRAIN_SCALE_SLIDER: NodeId =
    hash_node_id("painter_studio.grain_scale_slider");
pub const PAINTER_STUDIO_GRAIN_SCALE_CHIP: NodeId = hash_node_id("painter_studio.grain_scale_chip");
pub const PAINTER_STUDIO_GRAIN_DEPTH_SLIDER: NodeId =
    hash_node_id("painter_studio.grain_depth_slider");
pub const PAINTER_STUDIO_GRAIN_DEPTH_CHIP: NodeId = hash_node_id("painter_studio.grain_depth_chip");
pub const PAINTER_STUDIO_PIGMENT: NodeId = hash_node_id("painter_studio.pigment");
pub const PAINTER_STUDIO_ACCUMULATE: NodeId = hash_node_id("painter_studio.accumulate");
/// Paper tooth strength (0 = crisp ink, 1 = heavy paper) — world-space
/// substrate texture, independent of the brush grain source.
pub const PAINTER_STUDIO_PAPER_SLIDER: NodeId = hash_node_id("painter_studio.paper_slider");
pub const PAINTER_STUDIO_PAPER_CHIP: NodeId = hash_node_id("painter_studio.paper_chip");
/// Grain type cycler (Off → Simplex → Gabor → Weave → Spray → Off).
pub const PAINTER_STUDIO_GRAIN_TYPE: NodeId = hash_node_id("painter_studio.grain_type");
/// Rendering mode cycler (LightGlaze → … → IntenseBlending, 6 modes).
pub const PAINTER_STUDIO_RENDERING_MODE: NodeId = hash_node_id("painter_studio.rendering_mode");

// ── Color Dynamics section — per-stamp OKLab jitter (engine-wired) ────────────
pub const PAINTER_STUDIO_SEC_COLOR: NodeId = hash_node_id("painter_studio.sec_color");
pub const PAINTER_STUDIO_HUE_JITTER_SLIDER: NodeId =
    hash_node_id("painter_studio.hue_jitter_slider");
pub const PAINTER_STUDIO_HUE_JITTER_CHIP: NodeId = hash_node_id("painter_studio.hue_jitter_chip");
pub const PAINTER_STUDIO_SAT_JITTER_SLIDER: NodeId =
    hash_node_id("painter_studio.sat_jitter_slider");
pub const PAINTER_STUDIO_SAT_JITTER_CHIP: NodeId = hash_node_id("painter_studio.sat_jitter_chip");
pub const PAINTER_STUDIO_LIGHT_JITTER_SLIDER: NodeId =
    hash_node_id("painter_studio.light_jitter_slider");
pub const PAINTER_STUDIO_LIGHT_JITTER_CHIP: NodeId =
    hash_node_id("painter_studio.light_jitter_chip");
pub const PAINTER_STUDIO_DARK_JITTER_SLIDER: NodeId =
    hash_node_id("painter_studio.dark_jitter_slider");
pub const PAINTER_STUDIO_DARK_JITTER_CHIP: NodeId = hash_node_id("painter_studio.dark_jitter_chip");

// ── Wet Mix section — mixer-brush reservoir (W7, ADR-0097; engine-wired) ──────
pub const PAINTER_STUDIO_SEC_WET_MIX: NodeId = hash_node_id("painter_studio.sec_wet_mix");
/// Master enable for the Wet Mix reservoir (mirrors `wet_mix.wet_mix_enabled`).
/// Blending rendering modes auto-engage it; this checkbox lets any brush use it.
pub const PAINTER_STUDIO_WET_MIX_ENABLED: NodeId = hash_node_id("painter_studio.wet_mix_enabled");
pub const PAINTER_STUDIO_RESET_WET_MIX: NodeId = hash_node_id("painter_studio.reset_wet_mix");
pub const PAINTER_STUDIO_DILUTION_SLIDER: NodeId = hash_node_id("painter_studio.dilution_slider");
pub const PAINTER_STUDIO_DILUTION_CHIP: NodeId = hash_node_id("painter_studio.dilution_chip");
pub const PAINTER_STUDIO_CHARGE_SLIDER: NodeId = hash_node_id("painter_studio.charge_slider");
pub const PAINTER_STUDIO_CHARGE_CHIP: NodeId = hash_node_id("painter_studio.charge_chip");
pub const PAINTER_STUDIO_ATTACK_SLIDER: NodeId = hash_node_id("painter_studio.attack_slider");
pub const PAINTER_STUDIO_ATTACK_CHIP: NodeId = hash_node_id("painter_studio.attack_chip");
pub const PAINTER_STUDIO_PULL_SLIDER: NodeId = hash_node_id("painter_studio.pull_slider");
pub const PAINTER_STUDIO_PULL_CHIP: NodeId = hash_node_id("painter_studio.pull_chip");
pub const PAINTER_STUDIO_WETNESS_JITTER_SLIDER: NodeId =
    hash_node_id("painter_studio.wetness_jitter_slider");
pub const PAINTER_STUDIO_WETNESS_JITTER_CHIP: NodeId =
    hash_node_id("painter_studio.wetness_jitter_chip");
pub const PAINTER_STUDIO_GRADE_SLIDER: NodeId = hash_node_id("painter_studio.grade_slider");
pub const PAINTER_STUDIO_GRADE_CHIP: NodeId = hash_node_id("painter_studio.grade_chip");
pub const PAINTER_STUDIO_BLUR_SLIDER: NodeId = hash_node_id("painter_studio.blur_slider");
pub const PAINTER_STUDIO_BLUR_CHIP: NodeId = hash_node_id("painter_studio.blur_chip");
pub const PAINTER_STUDIO_BLUR_JITTER_SLIDER: NodeId =
    hash_node_id("painter_studio.blur_jitter_slider");
pub const PAINTER_STUDIO_BLUR_JITTER_CHIP: NodeId = hash_node_id("painter_studio.blur_jitter_chip");

// ── Dynamics section — per-stamp size/opacity jitter (engine-wired T1.7) ──────
pub const PAINTER_STUDIO_SEC_DYNAMICS: NodeId = hash_node_id("painter_studio.sec_dynamics");
pub const PAINTER_STUDIO_SIZE_JITTER_SLIDER: NodeId =
    hash_node_id("painter_studio.size_jitter_slider");
pub const PAINTER_STUDIO_SIZE_JITTER_CHIP: NodeId = hash_node_id("painter_studio.size_jitter_chip");
pub const PAINTER_STUDIO_OPACITY_JITTER_SLIDER: NodeId =
    hash_node_id("painter_studio.opacity_jitter_slider");
pub const PAINTER_STUDIO_OPACITY_JITTER_CHIP: NodeId =
    hash_node_id("painter_studio.opacity_jitter_chip");
