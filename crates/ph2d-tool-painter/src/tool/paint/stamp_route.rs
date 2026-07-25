//! The stamp **route dispatcher**: given a batch of dabs, pick which of the four stamp paths
//! ([`super::stamp_cache`]) to use based on the Shape + Grain slots (cacheable / canvas-cached /
//! per-pixel / ramped). Split from `paint.rs` for the workspace LOC cap; the routes themselves live in
//! `stamp_cache`.

use super::PaintMode;
use super::ramp_lut::RampLutOwner;
use crate::tool::PainterTool;
use ph2d_painter_brush::{BrushBlend, Dab, StrokeMethod};

impl PainterTool {
    /// Whether the active stroke method lets the shell coalesce a burst of raw pointer Moves into ONE
    /// delivery per frame (the restore + whole-shape re-stamp fill methods only show the latest position,
    /// so it is byte-identical). Delegates to [`ph2d_painter_brush::StrokeMethod::coalesces_canvas_motion`].
    /// The FPS-drop / "Raw rises" fix (`HANDOFF_per_layer_color_perf_artifacts` §1.R).
    ///
    /// **Selection mode** coalesces too — EXCEPT the Freehand lasso (`selection_mode == 1`), which
    /// CAPTURES the pointer path and needs every raw event. The others (shape-gizmo drags, Rectangle /
    /// Ellipse rubber bands, Automatic) only ever act on the latest position, and each Move pays a full
    /// boolean-mask recompose (~5 ms with 8 shapes @2048²) — delivered per RAW event, a high-Hz mouse
    /// re-ran it several times per frame: the Bug #2 re-stamp storm on the selection axis (Enio
    /// 2026-07-04, "booleanas multi-shape lentas de novo").
    #[must_use]
    pub fn coalesces_canvas_motion(&self) -> bool {
        if matches!(self.paint.paint_mode, PaintMode::Selection) {
            return self.paint.selection_mode != 1; // 1 = Freehand (path capture)
        }
        self.paint.brush.stroke_method.coalesces_canvas_motion()
    }

    /// Stamp a batch of dabs into `canvas_rgba`, gated by the Sculpt-style **protection** mask. While a
    /// mask scratch is live on the active layer (and we're NOT in Mask mode — that edits the scratch), the
    /// painted region is FROZEN against EVERY paint tool: snapshot the dab footprint, stamp normally, then
    /// restore the protected texels ([`Self::restore_protected_region`]). Nothing is made invisible — only
    /// the paint is gated. Engine-agnostic: it wraps ALL the routes in [`Self::stamp_dabs_routed`].
    pub(super) fn stamp_dabs(&mut self, dabs: &[Dab]) {
        // Watercolor optical render-path: DON'T deposit dabs on the canvas — accumulate the coverage
        // (max-blended discs) + the deposited colour (source-over), and let `apply_watercolor` reconstruct
        // the whole wash over the frozen base ([`super::watercolor_render`]). Short-circuits every route.
        //
        // Gated on an OPEN watercolor stroke: the frozen base exists ⇔ some lifecycle froze it. Originally
        // that meant only `paint_begin`, and the shape editors (Line/Curve/Ellipse/Polygon/Free Hand) fell
        // through to the plain deposit. **THAT IS NO LONGER TRUE** (doc 13 #3): they run the optics through
        // their OWN entry, `stamp_drag_preview_watercolor`, which freezes the ground and calls
        // `apply_watercolor` itself — so EVERY stroke method washes. The gate still earns its keep (a batch
        // reaching here with no base frozen must not enter the optical path), but the old wording said the
        // opposite and misled an audit (2026-07-12) — hence this correction.
        //
        // What the wash does NOT cover is the MODE: `watercolor_render_active()` is
        // `watercolor && paint_mode == Paint && !eraser`, so Eraser / Mask / Smear / Blur / Clone / Inpaint
        // keep the plain stamp path even with the checkbox ticked. `BrushSettings::watercolor_active`
        // publishes exactly this predicate to the panel, which hides **Accumulate** under the wash (it is
        // read only by `accumulate_cap`, below this short-circuit — dead there, and redundant anyway: the
        // wash coverage is max-blended, which already IS "no build-up within a stroke").
        if self.watercolor_render_active() && self.paint.watercolor_base.is_some() {
            // Seamless Tiling (doc 13 #2): replicate the dabs across the wrapped sprite edges BEFORE the
            // accumulate, so an edge-crossing wash also FORMS on the opposite edge (the divergence above
            // short-circuits before `stamp_dabs_routed`'s own `tiled_dabs`). The splats are canvas-indexed
            // (a wrapped dab's disc lands on the far edge) and `dab_batch_region` already spans the whole
            // tiled axis, so the frame dirty rect + composite window follow. Both passes get the SAME
            // wrapped list so their rng streams stay in lock-step. Off ⇒ the raw dabs (byte-identical).
            let tiled;
            let wet_dabs: &[Dab] = if self.paint.tiling[0] || self.paint.tiling[1] {
                let (w, g) =
                    super::tiling::tiled_dabs_grouped(dabs, self.source_size, self.paint.tiling);
                tiled = w;
                self.paint.dab_groups = g; // wrapped copies share their dab's random frame
                &tiled
            } else {
                self.paint.dab_groups.clear();
                dabs
            };
            self.accumulate_wet_coverage(wet_dabs);
            // Smudge > 0 = TRUE SMEAR: each dab physically drags the frozen base's paint dab-to-dab
            // (`smear_wet_base`) and the wash composites over the smeared base. Cumulative methods
            // only — the re-stamp previews (Drag Dot/Anchored/Line) would re-mutate the base every
            // frame. Off (default) → byte-identical. The wet-on-wet dissolve/lift (`wet_rewet`) is
            // per-pixel in `apply_watercolor`, not here. Smear keeps the raw (UNtiled) dab CHAIN so the
            // dab-to-dab drag stays continuous, and applies the Tiling wrap INTERNALLY (per-dab offsets +
            // toroidal lift, `smear_wet_base`) — so an edge-crossing smear also drags the opposite edge.
            if self.paint.brush.wet_smudge > 0.0
                && !matches!(
                    self.paint.brush.stroke_method,
                    StrokeMethod::DragDot | StrokeMethod::Anchored | StrokeMethod::Line
                )
            {
                self.smear_wet_base(dabs);
            }
            self.accumulate_wet_color(wet_dabs);
            return;
        }
        // Wet Paint enforces the canvas gates ITSELF — keep-lerp toward the
        // frozen base in `wetpaint_composite` (`splat_keep`, the watercolor
        // stance above). The snapshot/restore wrapper below would not only be
        // redundant, it KILLS the wet session every batch: `restore_*` runs
        // `Arc::make_mut` on a canvas whose Arc the session also holds
        // (refcount 2 ⇒ clone ⇒ new pointer), and the canvas-identity guard
        // reads the new Arc as a foreign mutation — under a selection the
        // water would never live past one batch, which is the mode's whole
        // point. Found by a surviving mutation (W2.5). The predicate is the
        // SAME one the route arm asks (`wet_owns_the_dabs`): the sessionless
        // wet eraser (W2.6) falls through to the normal eraser below, and
        // must go through the wrapper with it.
        if self.wet_owns_the_dabs() {
            self.stamp_dabs_routed(dabs);
            return;
        }
        // Two footprint gates share one snapshot: the Sculpt-style protection mask (freeze painted texels)
        // and the Selection mask (restrict paint to the selected region, ADR-0103). Snapshot the footprint
        // once, stamp, then revert whatever each active gate protects.
        let mask_gate = self.mask_protection_active();
        let sel_gate = self.selection_restricts_paint();
        if (mask_gate || sel_gate)
            && let Some(region) = self.dab_batch_region(dabs)
        {
            let before = self.snapshot_region(region);
            self.stamp_dabs_routed(dabs);
            if mask_gate {
                self.restore_protected_region(region, &before);
            }
            if sel_gate {
                self.restore_deselected_region(region, &before);
            }
            return;
        }
        self.stamp_dabs_routed(dabs);
    }

    /// Route a batch of dabs to one of the stamp paths (Smear / Blur / Clone / Mask / composite / cached /
    /// per-pixel / …) by mode + Shape/Grain slots. With **Tiling** on, each dab is first replicated across
    /// the wrapped sprite edges (`tiling::tiled_dabs`) for a seamless tile. [`Self::stamp_dabs`] wraps this
    /// with the protection gate.
    pub(super) fn stamp_dabs_routed(&mut self, dabs: &[Dab]) {
        // Smear drags canvas content between consecutive dab centres — it ignores the brush colour /
        // blend / ramp routing, so short-circuit before all of it. It needs the UNtiled dab chain (a
        // single `last_smear_pos` source) + applies Shape/Grain/flatten via the mask and Tiling via
        // per-endpoint offsets itself, so it runs here, not in `stamp_dabs_inner`.
        // Composite Brush (a Brush-tool upgrade): run Brush + Smear + Blur together, bottom→top. Takes
        // precedence over the single-op routing when it's active (checkbox on + the plain Brush tool).
        if self.composite_active() {
            self.stamp_dabs_composite(dabs);
            return;
        }
        if self.paint.paint_mode.smears() {
            let (w, h) = self.source_size;
            self.stamp_dabs_smear(dabs, w, h);
            return;
        }
        // Blur softens the canvas under each dab (Blender Soften). Like Smear it ignores the brush
        // colour / blend / ramp routing and applies Shape/Grain/flatten via the mask + Tiling itself,
        // so it short-circuits here into its own route ([`super::blur_route`]).
        if matches!(self.paint.paint_mode, PaintMode::Blur) {
            let (w, h) = self.source_size;
            self.stamp_dabs_blur(dabs, w, h);
            return;
        }
        // Clone copies canvas pixels from the sampled source at a fixed offset. Like Smear/Blur it
        // ignores the brush colour/blend and applies Shape/Grain/flatten + Tiling itself → own route.
        if matches!(self.paint.paint_mode, PaintMode::Clone) {
            let (w, h) = self.source_size;
            self.stamp_dabs_clone(dabs, w, h);
            return;
        }
        // Mask paints a GRAYSCALE value (a layer mask) — desaturate then use the normal brush path.
        if matches!(self.paint.paint_mode, PaintMode::Mask) {
            self.stamp_dabs_mask(dabs);
            return;
        }
        // Inpaint marks the defect region into the heal mask + tints the canvas for live feedback; the
        // reconstruction runs on pen-up ([`super::inpaint`]), so no colour is painted here.
        if matches!(self.paint.paint_mode, PaintMode::Inpaint) {
            self.stamp_dabs_inpaint(dabs);
            return;
        }
        if self.paint.tiling[0] || self.paint.tiling[1] {
            // The wrapped copies are the SAME dabs seen across the seam — publish the group map so the
            // routes draw each dab's random frame ONCE and share it (see `tiling::DabRng`).
            let (wrapped, groups) =
                super::tiling::tiled_dabs_grouped(dabs, self.source_size, self.paint.tiling);
            self.paint.dab_groups = groups;
            self.stamp_dabs_inner(&wrapped);
            self.paint.dab_groups.clear();
        } else {
            self.paint.dab_groups.clear(); // no Tiling ⇒ every entry is its own dab (byte-identical)
            self.stamp_dabs_inner(dabs);
        }
    }

    /// **Mask** route: paint into the Mask brush's TRANSIENT scratch (a tool-side mask, NOT a stack
    /// layer). The scratch is swapped INTO `canvas_rgba` so the whole stamp pipeline edits it, then
    /// swapped back — the layer's pixels are never touched. Paint (0) = conceal (black) · Erase (1) =
    /// reveal (white) · Blur (2) · Smear (3); ramps / Shape / Grain / falloff / Tiling / Symmetry / every
    /// Stroke Method apply as in Paint. The composite hides the layer by the scratch live ([`mask`]).
    pub(super) fn stamp_dabs_mask(&mut self, dabs: &[Dab]) {
        if self.paint.mask_scratch_target.is_none() {
            return; // no scratch to paint (ensured at `paint_begin`)
        }
        // A pre-stamp copy of the scratch, so an active selection can revert mask strokes that landed
        // outside it (the scratch swap hides the edit from the canvas-side stamp gate, ADR-0103).
        let scratch_before = self
            .selection_restricts_paint()
            .then(|| (*self.paint.mask_scratch_rgba).clone());
        // Swap the scratch into `canvas_rgba` so the stamp pipeline edits the SCRATCH, then swap back.
        std::mem::swap(&mut self.canvas_rgba, &mut self.paint.mask_scratch_rgba);
        match self.mask_brush() {
            2 => {
                let (w, h) = self.source_size;
                self.stamp_dabs_blur(dabs, w, h);
            }
            3 => {
                let (w, h) = self.source_size;
                self.stamp_dabs_smear(dabs, w, h);
            }
            _ => {
                // The mask is a pure COVERAGE buffer: force every dab to the mask colour with a plain Mix
                // blend. Black = protect (Paint), white = unprotect (Erase). NB the stamp reads each dab's
                // OWN `color` (baked from the user's brush colour at stroke start), so overriding only
                // `brush.color` was a NO-OP — that's why Erase, and any non-black brush colour, painted the
                // wrong coverage. Recolour the dabs AND set the spec colour/blend so every route agrees.
                let color = if self.mask_brush() == 1 {
                    [1.0, 1.0, 1.0] // Erase → unprotect (white)
                } else {
                    [0.0, 0.0, 0.0] // Paint → protect (black)
                };
                let recolored: Vec<Dab> = dabs.iter().map(|d| Dab { color, ..*d }).collect();
                let saved_color = self.paint.brush.color;
                let saved_blend = self.paint.brush.blend;
                self.paint.brush.color = color;
                self.paint.brush.blend = BrushBlend::Mix;
                if self.paint.tiling[0] || self.paint.tiling[1] {
                    let wrapped =
                        super::tiling::tiled_dabs(&recolored, self.source_size, self.paint.tiling);
                    self.stamp_dabs_inner(&wrapped);
                } else {
                    self.stamp_dabs_inner(&recolored);
                }
                self.paint.brush.color = saved_color;
                self.paint.brush.blend = saved_blend;
            }
        }
        // Clip the mask stroke to the selection while the scratch still lives in `canvas_rgba`.
        if let Some(orig) = scratch_before.as_ref() {
            self.clip_canvas_to_selection(orig);
        }
        std::mem::swap(&mut self.canvas_rgba, &mut self.paint.mask_scratch_rgba);
    }

    /// **Smear** route: drag the canvas content along the stroke — as an accumulated **displacement
    /// field**, not a per-dab lift-and-blend.
    ///
    /// The blend version decayed geometrically over a stroke (`h·wⁿ` at ~1 px spacing) and delivered a
    /// one-texel filament with no mass; the field version transports by a SUM and resolves once from the
    /// pixels frozen at pen-down. Colour and impasto ride the SAME map through the same door, so the
    /// pigment and the body cannot disagree about where the paint went — which is what retired the
    /// separate `plow_dabs` transport. See [`super::smear_warp`] and
    /// [`ph2d_painter_brush::smear_field`] for the law and the measurement.
    ///
    /// The per-pixel weight is still the brush's full dab coverage — **Shape** silhouette × **Grain** ×
    /// **flatten/rotate** × Selection — because the accumulation rides `walk_dab`, the one place a dab is
    /// shaped. **Tiling** accumulates the wrapped copies at their own offsets, as the blend did.
    pub(super) fn stamp_dabs_smear(&mut self, dabs: &[Dab], w: u32, h: u32) {
        let _ = self.smear_dabs_field(dabs, w, h);
    }

    /// The actual stamp dispatch (already tiled if needed); [`Self::stamp_dabs`] wraps first.
    pub(super) fn stamp_dabs_inner(&mut self, dabs: &[Dab]) {
        if dabs.is_empty() {
            return;
        }
        let (w, h) = self.source_size;
        let mut brush = self.paint.brush;
        // ── WET PAINT: the fluid engine owns the whole deposit ───────────────────────────────────
        // Before even the impasto height pass: in this mode the dabs are not the final paint (the
        // pigment FLOWS), so relief laid on the dab footprint would outlive paint that moved away.
        // Hanging off this choke point is what hands Symmetry / Tiling / pressure / the stroke
        // engine to the fluid for free — the same argument as the Sculpt arm below. The predicate
        // is shared with `stamp_dabs`' wrapper bypass: the sessionless wet eraser (W2.6) is NOT
        // owned and falls through to the normal eraser (it erases the BAKED canvas).
        if self.wet_owns_the_dabs() {
            self.stamp_dabs_wetpaint(dabs, &brush);
            return;
        }
        // ── IMPASTO: the dab pipeline's SECOND output ────────────────────────────────────────────
        // THE choke point. The list here is already mirrored (Symmetry, in the stroke engine) and
        // already replicated (Tiling, just above), and every colour route below is fed from it — so
        // running the height off this one list is what makes Mirror / Tiling / Stroke / the shape
        // editors / Jitter sculpt the relief without a single line of code per feature, and keeps
        // them sculpting it when someone changes them. A height pass hung off any route, or off its
        // own geometry, is the way "Tiling doesn't work in Impasto" gets born six months from now.
        //
        // It runs BEFORE the colour routes because it reads a COPY of the `tex_rng` stream, which the
        // routes are about to consume — see rule 2 in `super::impasto`.
        // WET PAINT lays NO relief, explicitly (doc 21): in that mode even the flat AUTHORING
        // previews flow through here (un-owned batches), and relief pinned to a dab footprint
        // would outlive pigment that the commit deposit then makes FLOW away (gate G12).
        // ⚠️ The contract is honoured TWICE by design: `impasto_applies` (inside the pass) is
        // Paint-only, so mutating THIS gate alone survives G12 — the same layered-defense shape
        // as the GPU light's early-out. This branch states the intent where the routing lives.
        if !matches!(self.paint.paint_mode, PaintMode::WetPaint) {
            self.stamp_dabs_height(dabs, &brush);
        }
        // ── SCULPT: the same list, a different verb on the same plane ────────────────────────────
        // Sculpt reads the layer's committed relief and RESHAPES it (Smooth / Sharpen; the spatula
        // verbs follow in Wave 2). It hangs off THIS choke point, and not off geometry of its own, for
        // exactly the reason the height does: Symmetry, Tiling, the shape editors, pressure, Jitter,
        // falloff, Shape and Grain reach it for free — and keep reaching it when someone changes them.
        //
        // It lays no pigment (`docs/Painter/18…` §5: scraping the BODY of the paint does not take its
        // PIGMENT — a sculpt that erased colour would just be the eraser), so it stops here and the
        // colour routes below never run.
        if matches!(self.paint.paint_mode, PaintMode::Sculpt) {
            self.stamp_dabs_sculpt(dabs, &brush);
            return;
        }
        // `Draw To = Depth`: a sculpting brush — thickness, no pigment. The relief is already laid
        // down, so leave the canvas byte-identical and stop here. (With Impasto off, `deposits_color`
        // is unconditionally true, so the setting is not even read — the master switch is the gate.)
        if !brush.deposits_color() && matches!(self.paint.paint_mode, PaintMode::Paint) {
            return;
        }
        // Eraser overrides the blend with Erase Alpha (the drawing blend in `brush.blend` is kept).
        if self.paint.eraser {
            brush.blend = ph2d_painter_brush::BrushBlend::EraseAlpha;
        }
        // Alpha lock: the dab paints only into the active layer's existing alpha (clip/mask composite-time).
        let alpha_locked = self
            .layers
            .active()
            .and_then(|id| self.layers.get(id))
            .is_some_and(|l| l.alpha_locked);
        let has_shape_image = self.paint.shape_image.is_some();
        // Per-layer-colour Shape (multi-layer, mode on): the z-ordered tinted layers recomposite onto the
        // canvas. Guarded by an ACTIVE Image silhouette so a stale `per_layer_color` flag (e.g. after the
        // Shape was reset to None) can never route a non-Image Shape into the coloured path (Enio).
        if self.paint.shape_layers.is_color_mode() && brush.shape_silhouette_active(has_shape_image)
        {
            // Per-dab dynamics the constant-orientation cached path can't express → the per-pixel dynamic
            // path: Shape Rake / Random rotation, Randomize Color, or Grain Jitter Rotate. Randomize Color
            // is `has_colour_jitter_amount()` (Hue/Sat/Val > 0) — the engine's actual gate; the legacy
            // `color_jitter_enabled` flag is dead (the panel drives it by amount), so checking it here
            // meant Randomize Color alone never reached the dynamic path (it only worked WITH Rake/Random).
            // A canvas-fixed Grain (Tiled / Stencil) cannot be baked into the dab-LOCAL coloured stamp
            // (`render_color_stamp_mask` samples in stamp-local coords) — doing so ignored the Stencil
            // rect entirely, so the colour leaked OUTSIDE it (worst with a big Anchored dab). The per-pixel
            // dynamic path samples the Grain at each canvas pixel through its canvas-fixed basis, so it
            // masks the rect correctly (Enio 2026-06-27).
            // The Grain half is `!texture.is_cacheable()` — the SAME predicate the grayscale routes use.
            // Hand-rolling the disjunction here (Rake / Random-Angle / canvas-fixed) left a hole: Grain
            // **Mapping = Random Offset** randomises the OFFSET, not the angle, so it matched none of the
            // clauses and was served by the constant-orientation coloured stamp — every dab got the SAME
            // texture offset and the knob did nothing (sweep 2026-07-12). `is_cacheable()` covers Rake,
            // Random-Angle, Random-Offset, Tiled and Stencil in one place, so the hole cannot re-open.
            let per_dab_dynamic = brush.shape_has_per_dab_rotation(has_shape_image)
                || brush.has_colour_jitter_amount()
                || brush.has_per_dab_rotation()
                || (brush.texture.is_active() && !brush.texture.is_cacheable());
            if per_dab_dynamic {
                self.stamp_dabs_per_layer_dynamic(dabs, &brush, alpha_locked, w, h);
            } else if self.paint.shape_layers.any_texture_color() {
                // Texture colour (the capture default) with a CONSTANT orientation: the per-pixel RGB is
                // pre-baked into each layer's premul-RGBA stamp → the batched 4-channel scale-blit path
                // (was the dynamic per-pixel resample = the live "FPS 60→10" cliff).
                self.stamp_dabs_cached_color_rgba(dabs, &brush, alpha_locked, w, h);
            } else {
                self.stamp_dabs_cached_color(dabs, &brush, alpha_locked, w, h);
            }
            return;
        }
        // Grain Jitter-Rotate OR a per-dab Shape rotation (Rake / Random) → each dab needs its own
        // basis, so the constant-orientation caches are skipped (the per-pixel path resolves per dab).
        let per_dab_rotation =
            brush.has_per_dab_rotation() || brush.shape_has_per_dab_rotation(has_shape_image);
        // Accumulate OFF caps the stroke at Strength (per-pixel mask); skip the caches when Strength < 1.
        let accumulate_cap = !brush.accumulate
            && (brush.strength < 1.0
                // Film AA (BUGS #16): a fractional rim texel must CAP at its area fraction — the
                // old premise ("the cap is observable only under Strength < 1") is false once a
                // texel's target alpha is fractional at full strength, because overlapping dabs
                // would build the rim right back to a hard edge (measured: 0.64 -> 0.94).
                || brush.film_aa_wanted(brush.shape.is_active()));
        // A Colour Ramp owns the painted COLOUR (baked LUT): the **Shape** ramp (its B&W filter off →
        // colourise the silhouette) or the **Grain** ramp (indexed by the Grain pattern). With NO Grain
        // the COVERAGE (the Shape silhouette, OR the bare falloff when there's no Shape image) indexes the
        // ramp, and the StampMask already caches that coverage → blit the cached mask applying
        // `ramp[coverage]` (as cheap as a plain cached stamp, NOT a per-pixel coverage recompute per dab —
        // critical for the Line/Curve/Ellipse/Polygon fills, which re-stamp the WHOLE shape every move).
        // Per-pixel otherwise: a Grain to index, per-dab rotation, or the Accumulate cap (Enio 2026-06-26).
        let grain_active = brush.texture.is_active();
        let owner = self.color_ramp_owner(grain_active);
        if owner != RampLutOwner::None {
            // `dab_mask_cacheable` already covers the no-Shape falloff case (a static View silhouette);
            // requiring `shape_silhouette_active` here needlessly forced a colour ramp on a plain brush
            // onto the per-pixel path — the cause of the slow ramped fills (Enio 2026-06-26).
            let static_ok =
                brush.dab_mask_cacheable(has_shape_image) && !per_dab_rotation && !accumulate_cap;
            // TextureAlpha mode punches the sprite alpha (a different blend) → keep it per-pixel; the
            // coloured stamp bakes coverage straight, so it serves None / Strength.
            let bakeable_alpha = !matches!(
                self.active_ramp_alpha_mode(owner),
                ph2d_painter_brush::RampAlphaMode::TextureAlpha
            );
            if static_ok && !grain_active {
                // No Grain: the silhouette/falloff coverage indexes the ramp → the cached grayscale
                // StampMask + `ramp[coverage]` at blit (cheap as a plain cached stamp).
                self.stamp_dabs_cached_ramped(dabs, &brush, alpha_locked, w, h);
            } else if static_ok && grain_active && bakeable_alpha {
                // Grain + ramp: the Grain VALUE indexes the ramp → bake the grain×ramp colour ONCE into
                // a coloured stamp + scale-blit (vs the per-pixel Grain+ramp recompute — the slow fills).
                self.stamp_dabs_cached_ramp_color(dabs, &brush, alpha_locked, w, h);
            } else {
                self.stamp_dabs_ramped(dabs, &brush, alpha_locked, w, h);
            }
            return;
        }
        // Whether either slot genuinely shapes the dab beyond the falloff: a Shape image (silhouette) or
        // an active Grain. A bare falloff brush (neither) stays on the per-pixel path, as before.
        let want_cache =
            brush.shape_silhouette_active(has_shape_image) || brush.texture.is_active();
        // Both slots static & View ⇒ bake silhouette × Grain once into the scale-invariant stamp (Blender).
        if want_cache
            && brush.dab_mask_cacheable(has_shape_image)
            && !per_dab_rotation
            && !accumulate_cap
        {
            self.stamp_dabs_cached(dabs, &brush, alpha_locked, w, h);
            return;
        }
        // Grain Tiled / Stencil is canvas-fixed but dab-independent — cache each canvas pixel's Grain
        // once per stroke + look it up per dab; the silhouette (Shape or falloff) stays per-pixel.
        if brush.texture.is_canvas_cacheable() && !per_dab_rotation && !accumulate_cap {
            self.stamp_dabs_canvas_cached(dabs, &brush, alpha_locked, w, h);
            return;
        }
        // Per-pixel path (no cache applies, or the Accumulate cap): resolves the texture frame +
        // Randomize-Color colour per dab and threads the per-stroke coverage mask.
        self.stamp_dabs_per_pixel(dabs, &brush, alpha_locked, w, h);
    }
}
