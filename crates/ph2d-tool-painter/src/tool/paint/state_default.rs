//! `PaintState::default` — the initial brush/stroke/texture/ramp state. Split from `paint.rs` for the
//! workspace LOC cap (the struct definition stays there; this is just its sizeable `Default` body). A
//! child module of `paint`, so it keeps construction access to `PaintState`'s module-private fields.

use super::*;

impl Default for PaintState {
    fn default() -> Self {
        // Moderate black brush (10 px); the brush-settings UI drives size/colour later.
        let base = BrushSpec {
            radius_px: 10.0,
            ..BrushSpec::default()
        };
        // Per-mode independent brush slots (default model): each mode starts from `base`, except the
        // continuous-processing tools (Smear/Blur/Clone), which want dense dabs — Spacing 5% (Krita's
        // smooth-smear guidance). A sparse chain leaves un-processed gaps between the disks.
        let dense = BrushSpec {
            spacing: 0.05,
            ..base
        };
        let mut brush_by_mode = [base; PAINT_MODE_COUNT];
        brush_by_mode[PaintMode::Smear.slot()] = dense;
        brush_by_mode[PaintMode::Blur.slot()] = dense;
        brush_by_mode[PaintMode::Clone.slot()] = dense;
        Self {
            brush: base,
            brush_by_mode,
            link_shared_settings: false,
            line_show_dimensions: true, // Line CAD dimensions on by default
            dynamics: Dynamics::default(),
            stroke: None,
            dabs: Vec::new(),
            seed: 0,
            tex_rng: 0,
            stroke_undo: None,
            eraser: false,
            paint_mode: PaintMode::default(),
            mask_brush: 0,         // Paint (conceal) — the default mask sub-brush
            mask_overlay_color: 0, // dark gray overlay by default
            eyedropper_armed: false,
            mask_scratch_rgba: Arc::new(Vec::new()),
            mask_scratch_target: None,
            selection_mask: Arc::new(Vec::new()),
            selection_active: false,
            selection_mode: 2, // Rectangle (Enio 2026-07-04 default; Automatic surprised on a flat canvas)
            selection_bool_op: 0, // New (replace)
            selection_threshold: 0.5, // mid tolerance for Automatic
            selection_overlay_opacity: 0.2, // subtle hatch by default (Enio 2026-07-02)
            selection_stabilizer: 0.5, // moderate lasso smoothing (mirrors the brush default)
            selection_drag: None,
            selection_base: Arc::new(Vec::new()),
            selection_crisp: Arc::new(Vec::new()),
            selection_feather: 0.0,
            selection_edit_mode: false,
            selection_shapes: Vec::new(),
            selection_raster_cache: Vec::new(),
            selection_grab: None,
            selection_clipboard: None,
            selection_offset_norm: 0.5, // centred → 0px offset (whole selection unchanged)
            selection_offset_active: false,
            selection_offset_rings: Vec::new(),
            selection_offset_source: Arc::new(Vec::new()),
            selection_offset_curves: Vec::new(),
            selection_offset_level_cache: Vec::new(),
            selection_ring_stack: false,

            // Composite off by default; the default stack is the natural read of the card (top→bottom):
            // Brush(1) over Smear(2) over Blur(3). Run bottom→top, that blurs → smears → paints on top.
            composite_enabled: false,
            composite: [
                CompositeLayer {
                    op: CompositeOp::Brush,
                    strength: 1.0,
                },
                CompositeLayer {
                    op: CompositeOp::Smear,
                    strength: 0.5,
                },
                CompositeLayer {
                    op: CompositeOp::Blur,
                    strength: 0.5,
                },
            ],
            clone_source: None,
            clone_offset: None,
            clone_aligned: true, // Aligned by default (standard clone-stamp)
            clone_sample_armed: false,
            last_smear_pos: None,
            tiling: [false, false],
            repeat_image: false,
            symmetry_pick: None,
            symmetry_line_start: None,
            symmetry_auto_center: true,
            moved_this_frame: false,
            drag_preview: None,
            line_anchor: None,
            line_constrain: false,
            scale_uniform: false,
            curve: None,
            last_non_shape_method: StrokeMethod::Space, // matches the default brush stroke method
            ellipse: None,
            line: None,
            line_snap: false,
            grid_snap_pos: None,
            polygon: None,
            parked_shapes: Vec::new(),
            active_op: stroke_multi::StrokeOp::default(),
            stroke_op_mode: stroke_multi::StrokeOp::default(),
            op_tap: None,
            selection_op_tap: None,
            shape_grab_tol_px: DEFAULT_SHAPE_GRAB_TOL_PX,
            shape_offset_norm: 0.5, // centred → 0px offset (default byte-identical)
            shape_offset_base_px: 0.0,
            offset_trim: false, // self-intersection trimming off by default
            stencil_grab: None,
            stencil_preview_s: 0.0,
            texture_image: None,
            paper_image: None,
            texture_image_pending: false,
            texture_image_version: 0,
            shape_image: None,
            shape_image_pending: false,
            shape_image_version: 0,
            shape_layers: shape_layers::ShapeLayers::default(),
            stamp_cache: None,
            color_stamp_cache: None,
            ramp_color_stamp_cache: None,
            canvas_tex_cache: None,
            texture_ramp: ph2d_color::ColorRamp::default(),
            texture_ramp_enabled: false,
            texture_ramp_bw: false,
            texture_ramp_lut: Vec::new(),
            texture_ramp_dirty: true,
            ramp_lut_version: 0,
            texture_ramp_alpha_mode: ph2d_painter_brush::RampAlphaMode::None,
            shape_color_ramp: ph2d_color::ColorRamp::default(),
            shape_color_ramp_enabled: false,
            shape_color_ramp_bw: false,
            shape_color_ramp_alpha_mode: ph2d_painter_brush::RampAlphaMode::None,
            shape_ramp_lut: Vec::new(),
            shape_ramp_dirty: true,
            shape_ramp_version: 0,
            ramp_lut_owner: ramp_lut::RampLutOwner::None,
            stroke_mask: Vec::new(),
            stroke_coverage: Vec::new(),
            stroke_color: Vec::new(),
            watercolor_base: None,
            inpaint_mask: Vec::new(),
            inpaint_patch_norm: 0.25, // patch radius 3 (today's InpaintParams::default)
            inpaint_quality_norm: 0.3333, // 6 EM iterations (today's default)
            inpaint_search_norm: 0.2, // margin ≈ hole/2 (mult 1.0)
            fill_threshold: 0.1,      // a small default ColorDrop tolerance
            fill_seed: None,
            fill_snapshot: Vec::new(),
            fill_last_rect: None,
            fill_return_mode: None,
            per_layer_stroke: stamp_color_cache::PerLayerStroke::default(),
            shape_color_preview: stamp_color_cache::ShapeColorPreview::default(),
            deform: warp::DeformState::default(),
        }
    }
}
