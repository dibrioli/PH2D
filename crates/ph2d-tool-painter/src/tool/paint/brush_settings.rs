//! Brush/Stroke parameter snapshot & setters (the single UI-edit clamp source); a submodule of
//! `paint`, split from `paint.rs` for the workspace LOC cap. Per-dab-jitter setters: `jitter_settings`.

use super::shape_layers::MAX_SHAPE_LAYERS;
use super::{
    BRUSH_AIRBRUSH_RATE_MAX_S, BRUSH_AIRBRUSH_RATE_MIN_S, BRUSH_COUNT_SLIDER_MAX,
    BRUSH_JITTER_ABS_MAX_PX, BRUSH_SIZE_MAX_PX, BRUSH_SIZE_MIN_PX, BRUSH_SPACING_MAX,
};
use crate::tool::PainterTool;
use ph2d_painter_brush::{
    BrushBlend, Falloff, FalloffPoint, HandleType, JitterUnit, MAX_FALLOFF_POINTS,
};

/// Deform **temperament** values for [`BrushSettings::deform_temperament`]. The panel opens with `NONE`
/// (neither segment selected) so the artist must pick — `RESHAPE` (brush) or `TRANSFORM` (gizmo).
pub const DEFORM_TEMPERAMENT_NONE: u8 = 0;
pub const DEFORM_TEMPERAMENT_RESHAPE: u8 = 1;
pub const DEFORM_TEMPERAMENT_TRANSFORM: u8 = 2;

// `BrushTextureImage` lives in the sibling `brush_image` module (LOC cap); re-exported so the existing
// `super::brush_settings::BrushTextureImage` import paths stay stable.
pub(super) use super::brush_image::BrushTextureImage;

/// Snap `cursor` to the nearest 45° ray from `anchor` (Blender Line Alt-constrain), projecting onto it.
/// Transcendental-free (abs/signum/mul + the `tan(22.5°)`/`tan(67.5°)`/`√½` constants) for HR-5.
pub(super) fn snap_to_45(anchor: [f32; 2], cursor: [f32; 2]) -> [f32; 2] {
    const TAN_22_5: f32 = 0.414_213_56; // tan(22.5°)
    const TAN_67_5: f32 = 2.414_213_5; // tan(67.5°)
    const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2; // √½
    let dx = cursor[0] - anchor[0];
    let dy = cursor[1] - anchor[1];
    let (adx, ady) = (dx.abs(), dy.abs());
    // The snapped unit direction (one of the 8 rays).
    let (ux, uy) = if ady <= adx * TAN_22_5 {
        (dx.signum(), 0.0) // horizontal
    } else if ady >= adx * TAN_67_5 {
        (0.0, dy.signum()) // vertical
    } else {
        (dx.signum() * DIAG, dy.signum() * DIAG) // diagonal
    };
    // Project the cursor onto the ray (dot product = signed distance along the unit direction).
    let proj = dx * ux + dy * uy;
    [anchor[0] + ux * proj, anchor[1] + uy * proj]
}

/// A compact snapshot of the active brush for the layers panel's Brush section, published each frame
/// by the shell bridge — the panel reads it to position the size/colour sliders + blend chip.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BrushSettings {
    /// Radius in image pixels (UI label "Size").
    pub size_px: f32,
    /// [`Self::size_px`] mapped onto the size slider's `0..1` track (squared, so small brushes get more).
    pub size_norm: f32,
    /// Overall opacity, `0..1` (UI "Strength").
    pub strength: f32,
    /// Distance-falloff preset ([`Falloff::to_u8`]) — the dab profile (replaces Hardness); [`Falloff::Custom`] (`9`) reads [`Self::falloff_points`].
    pub falloff: u8,
    /// The `Custom` falloff curve's control points (id + `[distance, strength]` + handle), first [`Self::falloff_len`] valid, ascending by distance; panel plots + drags by stable id.
    pub falloff_points: [FalloffPoint; MAX_FALLOFF_POINTS],
    /// Count of valid entries in [`Self::falloff_points`] (`2..=MAX_FALLOFF_POINTS`).
    pub falloff_len: u8,
    /// Straight-RGB paint colour in `[0, 1]`.
    pub color: [f32; 3],
    /// Blend-mode wire discriminant ([`BrushBlend::to_u8`]).
    pub blend: u8,
    /// Eraser mode — paints with Erase Alpha regardless of [`Self::blend`].
    pub eraser: bool,
    /// Active-op flags: **Smear**/**Blur**/**Clone** process pixels (no colour → hide colour controls +
    /// incremental-only methods; [`Self::paints_no_color`]); **Mask** paints a grayscale value (keeps all methods).
    pub is_smear: bool,
    pub is_blur: bool,
    pub is_clone: bool,
    pub is_mask: bool,
    /// **Inpaint** heal mode — shows the Inpaint card (Patch Size / Quality / Search) + hides colour/Strength.
    pub is_inpaint: bool,
    /// **Selection** mode — the panel shows ONLY the selection section (ADR-0103): mode (`0` Automatic /
    /// `1` Freehand / `2` Rectangle / `3` Ellipse) · boolean op (`0` New / `1` Add / `2` Remove) · Automatic
    /// threshold · Feather amount (all `0..1`) · Edit-Selection toggle.
    pub is_selection: bool,
    pub selection_mode: u8,
    pub selection_op: u8,
    pub selection_threshold: f32,
    /// Free (lasso) path **Stabilization** amount (`0..1`) — shown only in Freehand mode.
    pub selection_stabilizer: f32,
    pub selection_feather: f32,
    pub selection_edit: bool,
    pub selection_overlay_opacity: f32,
    /// **Deform** (Liquify) mode — the panel shows ONLY the deform section (mode-exclusive, like Selection):
    /// sub-mode segmented (`0` Push · `1` Twist · `2` Pinch · `3` Wrinkle · `4` Fold · `5` Reconstruct) ·
    /// Size/Pressure/Distortion/Momentum/Strength (all `0..1`; Strength `0.5`-centred bipolar).
    /// Distortion/Momentum are hidden in Reconstruct. Deform is confined to the active selection (or the
    /// whole sprite when none) automatically — no Freeze toggle.
    pub is_deform: bool,
    pub deform_mode: u8,
    pub deform_size_norm: f32,
    /// Deform brush radius in image px (mapped from [`Self::deform_size_norm`]) — the cursor ring reads this
    /// so the on-canvas ring shows the DEFORM footprint, not the paint brush's.
    pub deform_size_px: f32,
    pub deform_pressure: f32,
    pub deform_distortion: f32,
    pub deform_momentum: f32,
    pub deform_strength: f32,
    /// **Temperament** (Wave 2): `0` none picked · `1` Reshape (brush) · `2` Transform (gizmo) — see the
    /// `DEFORM_TEMPERAMENT_*` consts. Opens at `NONE` each time the panel is entered so the artist must
    /// choose; decides which body the mode-exclusive Deform section shows.
    pub deform_temperament: u8,
    /// Transform sub-mode (`0` Uniform aspect-locked · `1` Free independent axes) — only shown in Transform.
    pub deform_transform_mode: u8,
    /// **Offset** (grow/shrink) slider position (`0..1`, `0.5` = no change) — expands/contracts the edited
    /// boundary; only meaningful (and shown) in Edit mode.
    pub selection_offset: f32,
    /// Inpaint **Patch Size** slider track (`0..1`; chip shows the mapped `2..6` patch radius).
    pub inpaint_patch: f32,
    /// Inpaint **Quality** slider track (`0..1`; chip shows the mapped `3..12` EM iterations).
    pub inpaint_quality: f32,
    /// Inpaint **Search** slider track (`0..1`; chip shows the mapped `50..300` % context margin).
    pub inpaint_search: f32,
    /// Mask sub-brush (`0` Paint/`1` Erase/`2` Blur/`3` Smear) + overlay-tint colour index (`0` gray + 4 fluorescent).
    pub mask_brush: u8,
    pub mask_overlay_color: u8,
    /// Clone flags: source sampled? · **Aligned** (offset persists across strokes)? · "Set Source" armed?
    pub clone_has_source: bool,
    pub clone_aligned: bool,
    pub clone_sample_armed: bool,
    /// **Composite Brush** on: the Strength slider hides + the 3-layer stack card shows (panel).
    pub composite_enabled: bool,
    /// Composite stack op per position `[layer1, layer2, layer3]` (`CompositeOp::to_u8`: 0 Brush/1 Smear/2 Blur).
    pub composite_ops: [u8; 3],
    /// Composite stack Strength per position `[layer1, layer2, layer3]` (`0..1`).
    pub composite_strength: [f32; 3],
    /// Seamless **Tiling** (wrap-around painting) flags `[x, y]`.
    pub tiling: [bool; 2],
    /// **Repeat Image** tile-preview toggle (the on-canvas 3×3 grid).
    pub repeat_image: bool,

    // ── Symmetry (drawing mirror / radial; section above Tiling). `symmetry_axis` = `MirrorAxis::to_u8`
    //    (0 X / 1 Y / 2 Custom); segments 3..12; the two `*_pick_*` flags = a canvas pick mode is armed. ──
    pub symmetry_enabled: bool,
    pub symmetry_circular: bool,
    pub symmetry_axis: u8,
    pub symmetry_segments: u32,
    pub symmetry_pick_line: bool,
    pub symmetry_pick_center: bool,

    // ── Stroke section (raw values; the panel maps to slider tracks via the BRUSH_*_MAX consts) ──
    /// Stroke-method wire discriminant ([`StrokeMethod::to_u8`]).
    pub stroke_method: u8,
    /// Multi-shape **Operation** the next shape is created with (`0`=Overlay `1`=Add `2`=Remove) — the
    /// selected segment of the Stroke OPERATION card. See `tool::paint::stroke_multi::StrokeOp`.
    pub stroke_op_mode: u8,
    /// Spacing as a fraction of diameter (`0.10` = 10%); the slider track is this value.
    pub spacing: f32,
    /// **Offset** slider track (`0..1`, `0.5` = no offset) — perpendicular path offset for the shape editors.
    pub offset: f32,
    /// **Trim** the offset's self-intersections (Offset card checkbox).
    pub offset_trim: bool,
    /// Whether the Simplify button shows: a curve is editing AND (Free Hand, or the user has added a point).
    pub can_simplify: bool,
    /// A curve with points is being edited (Curve / Free Hand / converted shape) — gates the Save-As-Object button.
    pub has_drawn_curve: bool,
    /// "Adjust Strength for Spacing" on/off.
    pub space_attenuation: bool,
    /// **Accumulate** on/off: off (default) caps a stroke at Strength.
    pub accumulate: bool,
    /// "Sync with other tools" on/off: on = every paint tool shares these settings; off (default) = each
    /// tool independent. Drives the checkbox at the top of the brush panel.
    pub link_shared: bool,
    /// Line "Dimensions" on/off: show the live dx/dy + corner angles while drawing a Line. Drives the
    /// checkbox below the Stroke Method dropdown (Line method only).
    pub line_show_dimensions: bool,
    /// Relative jitter (`0..1`, fraction of diameter) — the Jitter slider under the Brush unit.
    pub jitter: f32,
    /// Absolute jitter in pixels — the Jitter slider under the View unit.
    pub jitter_absolute_px: f32,
    /// Jitter-unit wire discriminant ([`JitterUnit::to_u8`]; `0` = Brush, `1` = View).
    pub jitter_unit: u8,
    /// Dash on-fraction (`0..1`).
    pub dash_ratio: f32,
    /// Dash period in dab-slots.
    pub dash_samples: u32,
    /// Input-samples averaging window (`>= 1`).
    pub input_samples: u32,
    /// Stroke stabilizer intensity, `0..1` (the "how regular" knob).
    pub stabilizer: f32,
    /// Airbrush "Rate" — emission period in seconds (Airbrush only; track via `BRUSH_AIRBRUSH_RATE_*_S`).
    pub airbrush_rate_s: f32,
    /// "Edge to Edge" toggle — Anchored only (the stamp spans anchor→cursor, not grows from the anchor).
    pub edge_to_edge: bool,

    // ── Texture section (the brush texture mask; raw values — the panel maps to slider tracks) ──
    /// Texture kind wire discriminant ([`TextureKind::to_u8`]; `0` = None = no texture assigned).
    pub texture_kind: u8,
    /// Texture mapping wire discriminant ([`TextureMapping::to_u8`]).
    pub texture_mapping: u8,
    /// Texture rotation in whole degrees (`0..=360`).
    pub texture_angle_deg: u16,
    /// "Rake" — the texture rotation follows the stroke direction.
    pub texture_rake: bool,
    /// "Random" — the texture rotation is randomised per dab.
    pub texture_random: bool,
    /// Texture offset in tile fractions, per axis (`−1..1`).
    pub texture_offset: [f32; 2],
    /// Texture per-axis scale (`0.1..10`; `1.0` = one tile).
    pub texture_size: [f32; 2],
    /// **Stencil** rect centre, per axis (`−1..1`) — the gizmo placement, independent of the texture tiling.
    pub stencil_offset: [f32; 2],
    /// **Stencil** rect half-extent as a canvas fraction, per axis (`0.1..10`, default `0.5`); Stencil mapping only.
    pub stencil_size: [f32; 2],
    /// **Stencil** rect rotation in whole degrees (`0..=360`). Independent of [`Self::texture_angle_deg`].
    pub stencil_angle_deg: u16,
    /// Per-pattern parameter slots, normalized `[0, 1]`; meaning per kind (`param_specs`).
    pub texture_params: [f32; ph2d_painter_brush::MAX_TEX_PARAMS],
    /// **Grain Depth** (`0..1`; `1` = full bite, the default). How strongly the Grain modulates.
    pub grain_depth: f32,

    // ── Shape section (the silhouette tip; the falloff is its procedural default) ──
    /// Shape **source** kind (`TextureKind::to_u8`): `None` falloff · `Image` replaces it · procedural is masked. Drives the panel "Texture" picker.
    pub shape_kind: u8,
    /// Whether a Shape **image** is assigned (meaningful only when [`Self::shape_kind`] is `Image`).
    pub shape_has_image: bool,
    /// Shape rotation in whole degrees (`0..=360`).
    pub shape_angle_deg: u16,
    /// "Rake" — the Shape rotation follows the stroke direction.
    pub shape_rake: bool,
    /// "Random" — the Shape rotation is randomised per dab.
    pub shape_random: bool,
    /// Shape offset in tile fractions, per axis (`−1..1`).
    pub shape_offset: [f32; 2],
    /// Shape per-axis scale (`0.1..10`; `1.0` = the image fills the footprint).
    pub shape_size: [f32; 2],
    /// Procedural Shape per-pattern params (Contrast / Brightness + the kind's knob, `[0,1]`) — the Grain's `texture_params` twin for the Shape slot.
    pub shape_params: [f32; ph2d_painter_brush::MAX_TEX_PARAMS],
    /// Number of captured Shape layers (`0` = single-image / falloff; `> 1` shows the Per-Layer Color UI).
    pub shape_layer_count: u8,
    /// Capturable layers in the active document (visible top-level rasters); "Use Document Layers" shows when `> 1`.
    pub document_layer_count: u8,
    /// "Per-Layer Color" mode — each Shape layer paints its own colour, higher above lower; hides the ramp.
    pub shape_per_layer_color: bool,
    /// Per-layer "use a custom colour" toggle (entries `0..shape_layer_count` valid).
    pub shape_layer_color_on: [bool; MAX_SHAPE_LAYERS],
    /// Per-layer custom colour (straight RGB), used when [`Self::shape_layer_color_on`]`[i]`. With the
    /// checkbox OFF (default) the layer paints its own captured texture colour instead.
    pub shape_layer_color: [[f32; 3]; MAX_SHAPE_LAYERS],
    /// Per-layer blend mode ([`ph2d_painter_effects::BlendMode`] discriminant; the "B" chip).
    pub shape_layer_blend: [u8; MAX_SHAPE_LAYERS],
    /// Per-layer **opacity** `0..1` — a BRUSH-only scale on that layer's tip contribution (the numeric
    /// box), seeded from the captured document layer's opacity. Does NOT edit the painted document.
    pub shape_layer_opacity: [f32; MAX_SHAPE_LAYERS],
    /// **Dab Flatten** (`0..1`; `0` = round) — the Shape gizmo squishes the dab footprint into an ellipse.
    pub dab_flatten: f32,
    /// **Dab rotation** of the flatten/rotate gizmo, whole degrees (`0..=360`).
    pub dab_angle_deg: u16,

    // ── Grain Colour Ramp (maps the Grain scalar to a colour when enabled) ──
    /// Whether the Color Ramp drives the paint colour.
    pub texture_ramp_enabled: bool,
    /// **B&W** filter: desaturate the ramp to luminance (paint + display).
    pub texture_ramp_bw: bool,
    /// Ramp colour-interpolation space (`RampColorMode::to_u8`).
    pub texture_ramp_mode: u8,
    /// Ramp interpolation mode (`RampInterp::to_u8`).
    pub texture_ramp_interp: u8,
    /// Ramp stops `(pos, r, g, b, a, id)` display sRGB, first [`Self::texture_ramp_stop_count`] valid, sorted by `pos`.
    pub texture_ramp_stops: [[f32; 6]; PANEL_RAMP_STOPS],
    pub texture_ramp_stop_count: u8,
    /// Ramp alpha action (`RampAlphaMode::to_u8`): `0` off · `1` scales Strength · `2` drives sprite alpha.
    pub texture_ramp_alpha_mode: u8,

    // ── Shape Colour Ramp — colour twin of the Grain ramp (same fields). B&W off = owns colour, on = tone. ──
    pub shape_color_ramp_enabled: bool,
    pub shape_color_ramp_bw: bool,
    pub shape_color_ramp_mode: u8,
    pub shape_color_ramp_interp: u8,
    pub shape_color_ramp_stops: [[f32; 6]; PANEL_RAMP_STOPS],
    pub shape_color_ramp_stop_count: u8,
    pub shape_color_ramp_alpha_mode: u8,

    /// Per-dab randomize: Randomize-Color enable + HSV amounts + Jitter Scale/Rotate/Spacing (`0..1`).
    pub color_jitter_enabled: bool,
    pub color_jitter: [f32; 3],
    pub jitter_scale: f32,
    pub jitter_rotate: f32,
    pub jitter_spacing: f32,

    // ── Watercolor section (wet-media look; `docs/Painter/08_plano_aquarela_edge_grain_pigment.md`) ──
    /// Master enable for the Watercolor section (edge darkening + granulation + pigment).
    pub watercolor: bool,
    /// **Edge** darkening gain (`0..`) — the wet-edge "fringe" pooled at stroke boundaries.
    pub edge_gain: f32,
    /// **Spread** (canvas px) — blur radius of the coverage feeding the edge-darkening pass.
    pub edge_spread: f32,
    /// **Granulation** (`0..1`) — non-linear gate of deposition into the paper-tooth valleys.
    pub granulation: f32,
    /// **Pigment** — subtractive (Kubelka–Munk) wet-on-wet colour mixing toggle.
    pub pigment: bool,
    /// **Mix** (`0..1`) — how much the subtractive pigment path is applied.
    pub pigment_mix: f32,
    /// **Fill** (`0..1`) — interior density of the optical wash (render-path `fillDensity`).
    pub fill: f32,
    /// **Depth** (`> 0`) — Beer–Lambert optical-depth scale (render-path `DEPTH`).
    pub depth: f32,
    /// **Warp** (canvas px) — organic-boundary displacement of the coverage sampling (render-path).
    pub warp: f32,
    /// **Paper** slot kind (`TextureKind` wire u8) — the substrate tooth (its own full section).
    pub paper_kind: u8,
    /// **Paper** slot Size (x, y), each `0.1..100`.
    pub paper_size: [f32; 2],
    /// **Paper** slot Angle in whole degrees (fibre orientation).
    pub paper_angle: u16,
    /// **Granulation "Same as Paper"** — when true the granulation settles into the paper's own tooth
    /// (the Grain slot texture is ignored). Shown in the Grain section in watercolor mode.
    pub granulation_use_paper: bool,
}

/// Max ramp stops the panel snapshot carries (a ramp may hold up to `MAX_RAMP_STOPS = 32`; the editor
/// shows the first this-many — more than enough for hand-authored gradients).
pub const PANEL_RAMP_STOPS: usize = 16;

/// Map a radius in pixels onto the size slider's `0..1` track (inverse of [`size_norm_to_px`]).
/// Squared track → finer control at small sizes.
pub(super) fn size_px_to_norm(px: f32) -> f32 {
    let span = BRUSH_SIZE_MAX_PX - BRUSH_SIZE_MIN_PX;
    ((px - BRUSH_SIZE_MIN_PX) / span).clamp(0.0, 1.0).sqrt()
}

/// Map the size slider's `0..1` track onto a radius in pixels.
fn size_norm_to_px(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    BRUSH_SIZE_MIN_PX + t * t * (BRUSH_SIZE_MAX_PX - BRUSH_SIZE_MIN_PX)
}

/// Map a slider's `0..1` track onto a count in `1..=BRUSH_COUNT_SLIDER_MAX` (Input Samples /
/// Dash Length). Inverse of `count_to_norm` in the panel.
fn count_from_norm(t: f32) -> u32 {
    let span = (BRUSH_COUNT_SLIDER_MAX - 1) as f32;
    1 + (t.clamp(0.0, 1.0) * span).round() as u32
}

impl PainterTool {
    /// Set the brush distance-falloff preset from a wire discriminant (out-of-range → Smooth). `9` =
    /// the editable `Custom` curve ([`Self::set_brush_falloff_point`]).
    pub fn set_brush_falloff(&mut self, preset: u8) {
        self.paint.brush.falloff = Falloff::from_u8(preset);
    }

    /// Move `Custom` falloff control point `id` to `(distance, strength)` in `[0, 1]²` — may pass its
    /// neighbours (curve re-sorts; the stable `id` keeps the handle grabbed). Pure brush state.
    pub fn set_brush_falloff_point(&mut self, id: u8, distance: f32, strength: f32) {
        self.paint
            .brush
            .custom_falloff
            .set_point(id, distance, strength);
    }

    /// Insert a `Custom` falloff point at the widest gap; returns the new id, or `None` at the cap (panel "+").
    pub fn add_brush_falloff_point(&mut self) -> Option<u8> {
        self.paint.brush.custom_falloff.add_point()
    }

    /// Insert a `Custom` falloff control point at `(distance, strength)` — where the artist clicked.
    /// Returns the new stable id, or `None` at the point cap.
    pub fn add_brush_falloff_point_at(&mut self, distance: f32, strength: f32) -> Option<u8> {
        self.paint
            .brush
            .custom_falloff
            .add_point_at(distance, strength)
    }

    /// Set the handle type of `Custom` falloff control point `id` (`0` = Auto, `1` = Vector; right-click menu).
    pub fn set_brush_falloff_point_handle(&mut self, id: u8, handle: u8) {
        self.paint
            .brush
            .custom_falloff
            .set_handle(id, HandleType::from_u8(handle));
    }

    /// Remove `Custom` falloff control point `id` (no-op when only the two endpoints remain; "−" / Delete).
    pub fn remove_brush_falloff_point(&mut self, id: u8) {
        self.paint.brush.custom_falloff.remove_point(id);
    }

    /// Set the brush strength (`0..1`, overall opacity).
    pub fn set_brush_strength(&mut self, t: f32) {
        self.paint.brush.strength = t.clamp(0.0, 1.0);
    }

    /// Toggle eraser mode (overrides the blend with Erase Alpha while on).
    pub fn toggle_brush_eraser(&mut self) {
        self.paint.eraser = !self.paint.eraser;
    }

    // The paint-mode setters (`set_paint_tool_mode` / `is_smear_mode`) live in `stencil.rs` (LOC cap).

    /// Set the brush radius in pixels, clamped to the interactive size range.
    pub fn set_brush_size_px(&mut self, px: f32) {
        self.paint.brush.radius_px = px.clamp(BRUSH_SIZE_MIN_PX, BRUSH_SIZE_MAX_PX);
    }

    /// Set the brush radius from the size slider's `0..1` track.
    pub fn set_brush_size_norm(&mut self, t: f32) {
        self.set_brush_size_px(size_norm_to_px(t));
    }

    /// Nudge the brush radius by one step — `[` (`dir < 0`) / `]` (`dir >= 0`). Multiplicative (constant
    /// perceptual step) with a ±1 px floor so the smallest brushes still change. Returns the new px.
    pub fn nudge_brush_size(&mut self, dir: i32) -> f32 {
        const STEP: f32 = 1.15;
        let cur = self.paint.brush.radius_px;
        let next = if dir >= 0 {
            (cur * STEP).max(cur + 1.0)
        } else {
            (cur / STEP).min(cur - 1.0)
        };
        self.set_brush_size_px(next);
        self.paint.brush.radius_px
    }

    /// Set one straight-RGB colour channel (`0..3`) of the brush, clamped `0..1`.
    pub fn set_brush_color_channel(&mut self, ch: usize, v: f32) {
        if ch < 3 {
            self.paint.brush.color[ch] = v.clamp(0.0, 1.0);
            self.sync_brush_color_across_modes();
        }
    }

    /// The paint **colour** is shared across EVERY paint mode (unlike the per-mode size / hardness / spacing):
    /// broadcast the live brush colour into every `brush_by_mode` slot so it survives a mode switch. Without
    /// this, a colour picked in one mode was lost when the ColorDrop Fill (or any tool switch) swapped in
    /// another mode's slot — so Fill applied the previous / default (black) colour (Enio 2026-07-04). Mirrors
    /// the Photoshop / Procreate "one foreground colour for all tools" model.
    pub(super) fn sync_brush_color_across_modes(&mut self) {
        let color = self.paint.brush.color;
        for slot in &mut self.paint.brush_by_mode {
            slot.color = color;
        }
    }

    /// The brush paint colour as straight sRGB bytes (`[r, g, b]`). The single source of truth for the paint
    /// colour — used to seed the C&F colour picker + Fill cursor directly, so the picker never falls back to a
    /// stale widget-thumb value (grey / black). Brush = Fill = picker are always this colour (Enio 2026-07-03).
    #[must_use]
    pub fn brush_color_srgb8(&self) -> [u8; 3] {
        let c = self.paint.brush.color;
        [
            (c[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (c[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (c[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        ]
    }

    /// Set the brush paint colour from straight sRGB bytes — the inverse of [`Self::brush_color_srgb8`].
    /// The C&F picker forwards its live value here every frame (in EVERY mode, and once on the open→close
    /// edge to catch the final pick that closes the picker), so brush = Fill = picker are always the one
    /// colour and Fill never applies the previous colour (Enio 2026-07-03).
    pub fn set_brush_color_srgb8(&mut self, rgb: [u8; 3]) {
        for (ch, &v) in rgb.iter().enumerate() {
            self.paint.brush.color[ch] = f32::from(v) / 255.0;
        }
        self.sync_brush_color_across_modes();
    }

    /// Set the brush blend mode from a wire discriminant (out-of-range → Mix).
    pub fn set_brush_blend(&mut self, mode: u8) {
        self.paint.brush.blend = BrushBlend::from_u8(mode);
    }

    // ── Stroke section setters (the single clamp source; the panel forwards raw UI values) ──

    /// Set spacing as a fraction of diameter (slider track), clamped to the interactive range.
    pub fn set_brush_spacing(&mut self, frac: f32) {
        self.paint.brush.spacing = frac.clamp(0.01, BRUSH_SPACING_MAX);
    }

    // The Inpaint heal-reconstruction setters (`set_inpaint_patch`/`_quality`/`_search`) + their SetValue
    // router live beside the heal itself in `paint::inpaint` (this file is at the workspace LOC cap).

    /// Toggle "Adjust Strength for Spacing".
    pub fn toggle_brush_space_attenuation(&mut self) {
        self.paint.brush.space_attenuation = !self.paint.brush.space_attenuation;
    }

    /// Toggle **Accumulate** (off caps a stroke at Strength; on lets overlapping dabs build up).
    pub fn toggle_brush_accumulate(&mut self) {
        self.paint.brush.accumulate = !self.paint.brush.accumulate;
    }

    /// Set the Jitter slider (`0..1` track), routed by the current unit: `Brush` → relative jitter
    /// (`0..1`), `View` → absolute pixels (`track × BRUSH_JITTER_ABS_MAX_PX`).
    pub fn set_brush_jitter_norm(&mut self, t: f32) {
        let t = t.clamp(0.0, 1.0);
        match self.paint.brush.jitter_unit {
            JitterUnit::View => self.paint.brush.jitter_absolute_px = t * BRUSH_JITTER_ABS_MAX_PX,
            JitterUnit::Brush => self.paint.brush.jitter = t,
        }
    }

    /// Set the jitter unit from a wire discriminant (out-of-range → Brush).
    pub fn set_brush_jitter_unit(&mut self, u: u8) {
        self.paint.brush.jitter_unit = JitterUnit::from_u8(u);
    }

    /// Set the dash on-fraction (`0..1`).
    pub fn set_brush_dash_ratio(&mut self, t: f32) {
        self.paint.brush.dash_ratio = t.clamp(0.0, 1.0);
    }

    /// Set the dash period from the slider's `0..1` track → `1..=BRUSH_COUNT_SLIDER_MAX` slots.
    pub fn set_brush_dash_length_norm(&mut self, t: f32) {
        self.paint.brush.dash_samples = count_from_norm(t);
    }

    /// Set the input-samples window from the slider's `0..1` track → `1..=BRUSH_COUNT_SLIDER_MAX`.
    pub fn set_brush_input_samples_norm(&mut self, t: f32) {
        self.paint.brush.input_samples = count_from_norm(t);
    }

    /// Set the stroke stabilizer intensity from the slider's `0..1` track (the "how regular" knob).
    pub fn set_brush_stabilizer(&mut self, t: f32) {
        self.paint.brush.stabilizer = t.clamp(0.0, 1.0);
    }

    /// Set the airbrush **Rate** (timer period, seconds) from the slider's `0..1` track, mapped
    /// linearly onto `[BRUSH_AIRBRUSH_RATE_MIN_S, BRUSH_AIRBRUSH_RATE_MAX_S]` (default `0.1`).
    pub fn set_brush_airbrush_rate_norm(&mut self, t: f32) {
        let t = t.clamp(0.0, 1.0);
        self.paint.brush.airbrush_rate_s =
            BRUSH_AIRBRUSH_RATE_MIN_S + t * (BRUSH_AIRBRUSH_RATE_MAX_S - BRUSH_AIRBRUSH_RATE_MIN_S);
    }

    /// Toggle "Edge to Edge" (Anchored: the stamp spans anchor→cursor instead of growing from it).
    pub fn toggle_brush_edge_to_edge(&mut self) {
        self.paint.brush.edge_to_edge = !self.paint.brush.edge_to_edge;
    }

    // The Grain-texture / Stencil / Dab setters (set_brush_texture_* / _stencil_* / _dab_*) live in the
    // sibling `brush_texture_settings` module (workspace file-LOC cap).
}
