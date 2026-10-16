//! As duas rotas da **rampa de cor** (cortadas de [`super::stamp_cache`] pelo tecto de LOC, prompt 36):
//! a do carimbo em cache sem Grain ([`PainterTool::stamp_dabs_cached_ramped`]) e a por pixel
//! ([`PainterTool::stamp_dabs_ramped`]). Quem INDEXA a rampa é a dona dela — a lei vive numa porta só,
//! [`ph2d_painter_brush::RampPaint`].

use super::stamp_cache::{mask_size_for, prepare_stroke_mask};
use super::{Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::{BrushSpec, Dab, blit_stamp_ramped};

impl PainterTool {
    /// Like [`Self::stamp_dabs_cached`] but the per-texel colour comes from the **Shape Colour Ramp**
    /// indexed by the cached coverage — the cacheable fast path for a no-Grain colour-ramp stroke whose
    /// layers all read the coverage (no Shape, an Image tip, or Falloff Color alone). The
    /// silhouette is rendered once into the [`ph2d_painter_brush::StampMask`] (the no-Grain gate already
    /// suppresses the B&W tone ramp, so the mask is the raw silhouette × falloff coverage) and the ramp
    /// is a 256-entry LUT lookup at blit time — vs the per-pixel [`Self::stamp_dabs_ramped`] recomputing
    /// the silhouette every pixel every dab (Enio 2026-06-26).
    pub(super) fn stamp_dabs_cached_ramped(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        let owner = self.color_ramp_owner(brush.texture.is_active());
        self.ensure_ramp_lut(owner);
        let max_r = dabs.iter().map(|d| d.radius_px).fold(0.0_f32, f32::max);
        self.ensure_stamp_cache(
            brush,
            mask_size_for(max_r * brush.stamp_domain(self.paint.shape_image.is_some())),
        );
        let Some((mask, _)) = self.paint.stamp_cache.as_ref() else {
            return;
        };
        // Owner Shape: its two layers composed per index — both indices ARE the coverage here (the router
        // sends the procedural Texture Color elsewhere: [`Self::padrao_do_shape_vira_cor`]).
        let ramp = if owner == super::ramp_lut::RampLutOwner::Shape {
            self.paint.shape_camadas.lut_combinada.as_slice()
        } else {
            self.paint.texture_ramp_lut.as_slice()
        };
        let alpha_mode = super::ramp_lut::BRUSH_RAMP_ALPHA;
        let buf = super::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            super::region::dabs_bounds(dabs, w, h, brush.tip_reach()),
        );
        let mut touched: Option<Region> = None;
        for d in dabs {
            let spec = BrushSpec {
                radius_px: d.radius_px,
                color: d.color,
                ..*brush
            };
            if let Some(r) = blit_stamp_ramped(
                buf,
                w,
                h,
                d.center,
                d.radius_px,
                mask,
                &spec,
                d.coverage,
                alpha_locked,
                ramp,
                alpha_mode,
            ) {
                let rect = Region {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                };
                touched = Some(touched.map_or(rect, |acc| union_region(acc, rect)));
            }
        }
        self.declare_wrote(touched);
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }

    /// Stamp each dab through the **Color Ramp** colour path — the texture's scalar indexes the baked
    /// ramp LUT for the per-texel colour (the scalar-only caches can't carry colour, so this is
    /// per-pixel; the 256-entry LUT keeps the lookup cheap). Mirrors the per-pixel branch of
    /// [`Self::stamp_dabs`], calling [`ph2d_painter_brush::stamp_dab_ramped`].
    pub(super) fn stamp_dabs_ramped(
        &mut self,
        dabs: &[Dab],
        brush: &BrushSpec,
        alpha_locked: bool,
        w: u32,
        h: u32,
    ) {
        let owner = self.color_ramp_owner(brush.texture.is_active());
        self.ensure_ramp_lut(owner);
        self.ensure_shape_ramp_lut();
        let textured = brush.texture.is_active();
        // Disjoint borrows: the imported image + the ramp LUT (both `self.paint` sub-fields) and the
        // canvas (`self.canvas_rgba`) are held at once; the texture RNG is copied out + written back.
        let papel = self.papel_como_grain(); // a imagem do slot que faz de Grain (`dab_spec`)
        let image = if papel {
            &self.paint.paper_image
        } else {
            &self.paint.texture_image
        }
        .as_ref()
        .map(|i| i.as_mask());
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        let shape_active = brush.shape_silhouette_active(shape_image.is_some());
        // Shape **tone** ramp applies when the Shape ramp's B&W filter is on (then the Grain / brush
        // owns colour and the Shape remaps the silhouette tone); off ⇒ the Shape ramp is the colour
        // owner, so no tone remap (Enio 2026-06-26).
        let shape_ramp_lut = (self.paint.shape_color_ramp_enabled
            && self.paint.shape_color_ramp_bw)
            .then_some(self.paint.shape_ramp_lut.as_slice());
        // [`Self::rampa_de_cor`], field by field: the canvas is borrowed mutably below.
        let rampa = ph2d_painter_brush::RampPaint {
            lut: &self.paint.texture_ramp_lut,
            alpha_mode: super::ramp_lut::BRUSH_RAMP_ALPHA,
            index: owner.index(),
            shape: if owner == super::ramp_lut::RampLutOwner::Shape {
                self.paint.shape_camadas.motor(shape_active)
            } else {
                ph2d_painter_brush::ShapeRampLayers::default()
            },
        };
        // A dab's wrapped Tiling copies must share ONE random frame (see `tiling::DabRng`).
        let groups = self.paint.dab_groups.clone();
        let mut dab_rng = super::tiling::DabRng::new(self.paint.tex_rng);
        // Does this stroke track its own coverage? ONE door ([`Self::stroke_cover_wanted`]) — so a
        // Color-Ramp stroke honours Accumulate (Enio 2026-06-25) without this route holding its own
        // opinion about the rule.
        let accumulate_cap = self.stroke_cover_wanted(brush);
        if accumulate_cap {
            prepare_stroke_mask(
                &mut self.paint.stroke_mask,
                (w as usize) * (h as usize),
                brush.stroke_method,
            );
        }
        let buf = super::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            super::region::dabs_bounds(dabs, w, h, brush.tip_reach()),
        );
        let mut mask: Option<&mut [u8]> =
            accumulate_cap.then_some(self.paint.stroke_mask.as_mut_slice());
        let mut touched: Option<Region> = None;
        for (di, d) in dabs.iter().enumerate() {
            let tex_rng = dab_rng.enter(&groups, di);
            let spec = BrushSpec {
                radius_px: d.radius_px,
                color: d.color,
                ..*brush
            };
            // The Rake heading is the dab's own smoothed path direction `d.dir` (computed once in the
            // engine, where the path geometry is known). Both slots read the same heading; `dab_basis`
            // ignores it unless that slot's Rake is on, and falls back to the base Angle for `[0, 0]`.
            // Jitter Rotate spins the whole footprint (falloff + Shape + View-Grain); per-slot Random Angle
            // stays the basis `extra_rot = [1, 0]` so the pattern-within isn't double-rotated.
            let rotor = spec.dab_rotor(d);
            let fp = spec.dab_footprint(rotor);
            let shape_basis = shape_active.then(|| {
                ph2d_painter_brush::texture::shape_basis(
                    &spec.shape,
                    &mut *tex_rng,
                    [w as f32, h as f32],
                    fp,
                    ph2d_painter_brush::texture::ShapeFrame::Stroke {
                        arc_len: d.arc_len,
                        unit_px: d.stroke_radius_px,
                    },
                )
            });
            let shape_in = shape_basis
                .as_ref()
                .map(|sb| ph2d_painter_brush::ShapeInput {
                    basis: sb,
                    image: shape_image.as_ref(),
                    ramp_lut: shape_ramp_lut,
                });
            let basis = textured.then(|| {
                ph2d_painter_brush::texture::dab_basis(
                    &spec.texture,
                    &mut *tex_rng,
                    [w as f32, h as f32],
                    fp,
                )
            });
            if let Some(r) = ph2d_painter_brush::stamp_dab_ramped(
                buf,
                w,
                h,
                d.center,
                &spec,
                d.coverage,
                alpha_locked,
                basis.as_ref(),
                image.as_ref(),
                shape_in,
                rampa,
                mask.as_deref_mut(),
                rotor,
                d.capsula(&spec),
            ) {
                let rect = Region {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                };
                touched = Some(touched.map_or(rect, |acc| union_region(acc, rect)));
            }
        }
        self.paint.tex_rng = dab_rng.finish();
        self.declare_wrote(touched);
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }
}
