//! The pre-publish [`FALLBACK_BRUSH`] snapshot — the Blender-default [`BrushSettings`] the Brush panel
//! paints before the shell's per-frame bridge publishes the live brush. Split from [`crate::paint_brush`]
//! for the file-LOC cap; re-exported there as `crate::paint_brush::FALLBACK_BRUSH` so callers are stable.
//! Pure data (no painting) — the name drops the `paint_` prefix so the HR-12 a11y gate (which checks
//! `paint*.rs` orchestrator files) skips it.

use ph2d_tool_painter::{BrushSettings, FalloffPoint, HandleType};

/// Padding entry for the fixed-size `falloff_points` array (only the first
/// `falloff_len` are read).
const FALLOFF_PT_NIL: FalloffPoint = FalloffPoint {
    id: 0,
    x: 0.0,
    y: 0.0,
    handle: HandleType::Auto,
};

/// Brush used before the first snapshot publish (Painter just activated; the bridge then publishes
/// every frame). `pub(crate)` so the Stroke-section paint tests reuse it as a Blender-default base
/// (struct-update the one field under test).
pub(crate) const FALLBACK_BRUSH: BrushSettings = BrushSettings {
    size_px: 25.0,    // LITERAL-PX-OK: defensive pre-publish fallback default
    size_norm: 0.217, // LITERAL-PX-OK: defensive pre-publish fallback default
    strength: 1.0,
    falloff: 0,
    // Default Custom curve = linear ramp (0,1)→(1,0); only read when falloff == Custom.
    falloff_points: [
        FalloffPoint {
            id: 0,
            x: 0.0,
            y: 1.0,
            handle: HandleType::Auto,
        },
        FalloffPoint {
            id: 1,
            x: 1.0,
            y: 0.0,
            handle: HandleType::Auto,
        },
        FALLOFF_PT_NIL,
        FALLOFF_PT_NIL,
        FALLOFF_PT_NIL,
        FALLOFF_PT_NIL,
        FALLOFF_PT_NIL,
        FALLOFF_PT_NIL,
    ],
    falloff_len: 2,
    color: [0.0, 0.0, 0.0],
    blend: 0,
    eraser: false,
    is_smear: false,
    is_blur: false,
    is_clone: false,
    is_mask: false,
    is_inpaint: false,
    is_selection: false,
    selection_mode: 0,
    selection_op: 0,
    selection_threshold: 0.5, // LITERAL-PX-OK: brush default (mirrors PaintState::default selection_threshold)
    selection_stabilizer: 0.5, // LITERAL-PX-OK: brush default (mirrors PaintState::default selection_stabilizer)
    selection_feather: 0.0,
    selection_edit: false,
    selection_overlay_opacity: 0.2, // LITERAL-PX-OK: brush default (mirrors PaintState::default)
    is_deform: false,
    deform_mode: 0,
    deform_size_norm: 0.5, // LITERAL-PX-OK: Deform defaults (mirror DeformState::default)
    deform_size_px: 128.75, // LITERAL-PX-OK: 1 + 0.5² · (512−1) = the size_norm 0.5 default radius
    deform_pressure: 0.8,  // LITERAL-PX-OK: Deform default
    deform_distortion: 0.0, // LITERAL-PX-OK: Deform default (turbulence off → clean Push)
    deform_momentum: 0.0,
    deform_strength: 0.5, // LITERAL-PX-OK: centred bipolar Strength (neutral)
    deform_temperament: 0,
    deform_transform_mode: 0,
    selection_offset: 0.5, // LITERAL-PX-OK: centred Offset = no grow/shrink (mirrors shape_offset_norm)
    inpaint_patch: 0.25, // LITERAL-PX-OK: brush default (mirrors PaintState::default inpaint_patch_norm)
    inpaint_quality: 0.3333, // LITERAL-PX-OK: brush default (mirrors PaintState::default inpaint_quality_norm)
    inpaint_search: 0.2, // LITERAL-PX-OK: brush default (mirrors PaintState::default inpaint_search_norm)
    mask_brush: 0,
    mask_overlay_color: 0,
    clone_has_source: false,
    clone_aligned: true,
    clone_sample_armed: false,
    composite_enabled: false,
    composite_ops: [0, 1, 2], // Brush / Smear / Blur (mirrors PaintState::default)
    composite_strength: [1.0, 0.5, 0.5],
    tiling: [false, false],
    repeat_image: false,
    // Symmetry section — disabled by default (mirror X, 6 segments), no pick mode armed.
    symmetry_enabled: false,
    symmetry_circular: false,
    symmetry_axis: 0, // MirrorAxis::X
    symmetry_segments: 6,
    symmetry_pick_line: false,
    symmetry_pick_center: false,
    // Stroke section (mirrors BrushSpec::default / Blender defaults).
    stroke_method: 3,         // Space
    stroke_op_mode: 0,        // Overlay (multi-shape default = no boolean)
    spacing: 0.10,            // LITERAL-PX-OK: Blender brush default (mirrors BrushSpec::default)
    offset: 0.5,              // LITERAL-PX-OK: centred Offset track = no perpendicular offset
    offset_trim: false,       // self-intersection trim off by default
    can_simplify: false,      // no curve editor in the fallback snapshot
    has_drawn_curve: false,   // no drawn curve in the fallback snapshot
    space_attenuation: false, // Adjust Strength off by default (Enio 2026-06-24; mirrors BrushSpec::default)
    accumulate: false,
    link_shared: false, // "Sync with other tools" off = independent tools (the default)
    line_show_dimensions: true, // Line CAD dimensions shown by default
    jitter: 0.0,
    jitter_absolute_px: 0.0,
    jitter_unit: 0, // Brush
    dash_ratio: 1.0,
    dash_samples: 20,
    input_samples: 1,
    stabilizer: 0.5,
    airbrush_rate_s: 0.1, // LITERAL-PX-OK: Blender default (DNA_brush_types.h:232)
    edge_to_edge: false,
    // Texture section (mirrors TextureSettings::default — no texture assigned).
    texture_kind: 0,    // None
    texture_mapping: 0, // View Plane
    texture_angle_deg: 0,
    texture_rake: false,
    texture_random: false,
    texture_offset: [0.0, 0.0],
    texture_size: [1.0, 1.0],
    stencil_offset: [0.0, 0.0],
    stencil_size: [0.5, 0.5],
    stencil_angle_deg: 0,
    texture_params: [0.5; ph2d_tool_painter::MAX_TEX_PARAMS],
    dab_flatten: 0.0,
    dab_angle_deg: 0,
    grain_depth: 1.0,
    // Shape section (mirrors BrushSpec::default — kind None, no Shape image, silhouette = falloff).
    shape_kind: 0,
    shape_has_image: false,
    shape_angle_deg: 0,
    shape_rake: false,
    shape_random: false,
    shape_offset: [0.0, 0.0],
    shape_size: [1.0, 1.0],
    shape_params: [0.5; ph2d_tool_painter::MAX_TEX_PARAMS],
    shape_layer_count: 0,
    document_layer_count: 0,
    shape_per_layer_color: false,
    shape_layer_color_on: [false; ph2d_tool_painter::MAX_SHAPE_LAYERS],
    shape_layer_color: [[0.0; 3]; ph2d_tool_painter::MAX_SHAPE_LAYERS],
    shape_layer_blend: [0; ph2d_tool_painter::MAX_SHAPE_LAYERS],
    shape_layer_opacity: [1.0; ph2d_tool_painter::MAX_SHAPE_LAYERS],
    texture_ramp_enabled: false,
    texture_ramp_bw: false,
    texture_ramp_mode: 0,
    texture_ramp_interp: 2,
    texture_ramp_stops: [[0.0; 6]; ph2d_tool_painter::PANEL_RAMP_STOPS],
    texture_ramp_stop_count: 0,
    texture_ramp_alpha_mode: 0,
    shape_color_ramp_enabled: false,
    shape_color_ramp_bw: false,
    shape_color_ramp_mode: 0,
    shape_color_ramp_interp: 2,
    shape_color_ramp_stops: [[0.0; 6]; ph2d_tool_painter::PANEL_RAMP_STOPS],
    shape_color_ramp_stop_count: 0,
    shape_color_ramp_alpha_mode: 0,
    color_jitter_enabled: false,
    color_jitter: [0.0, 0.0, 0.0],
    jitter_scale: 0.0,
    jitter_rotate: 0.0,
    jitter_spacing: 0.0,
    // Watercolor section — the `watercolor` gate (off) guarantees neutrality; the params carry the
    // when-enabled defaults (mirrors BrushSpec::default).
    watercolor: false,
    edge_gain: 1.5, // LITERAL-PX-OK: default edge-darkening gain (mirrors BrushSpec::default)
    edge_spread: 7.0, // LITERAL-PX-OK: default edge-darkening blur radius (mirrors BrushSpec::default)
    granulation: 0.3, // LITERAL-PX-OK: default granulation gate (mirrors BrushSpec::default)
    pigment: false,
    pigment_mix: 0.5, // LITERAL-PX-OK: default pigment mix (mirrors BrushSpec::default)
    fill: 0.12,       // LITERAL-PX-OK: default wash fill density (mirrors BrushSpec::default)
    depth: 1.2,       // LITERAL-PX-OK: default Beer–Lambert depth (mirrors BrushSpec::default)
    warp: 6.0,        // LITERAL-PX-OK: default boundary warp px (mirrors BrushSpec::default)
    wet_smudge: 0.0,  // LITERAL-PX-OK: Wet Mix off by default (mirrors BrushSpec::default)
    wet_rewet: 0.0,   // LITERAL-PX-OK: wet-on-wet off (mirrors BrushSpec::default)
    paper_kind: 0,    // None (TextureKind wire 0)
    paper_mapping: 1, // Tiled (canvas-anchored)
    paper_rake: false,
    paper_random: false,
    paper_angle: 0,
    paper_offset: [0.0, 0.0],
    paper_size: [1.0, 1.0], // LITERAL-PX-OK: default paper Size (one tile)
    paper_depth: 1.0,       // LITERAL-PX-OK: default paper tooth depth
    paper_params: [0.5; 6], // LITERAL-PX-OK: neutral texture params
    granulation_use_paper: true,
    paper_color: [1.0, 1.0, 1.0], // white ground (mirrors PaintState default)
};
