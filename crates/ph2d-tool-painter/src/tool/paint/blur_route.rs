//! The **Blur** (soften) route: for each dab, blur the canvas neighbourhood under its footprint and
//! blend it back, weighted by the brush's full dab coverage (Shape silhouette × Grain × flatten/rotate)
//! × Strength × pressure. The soften sibling of [`super::stamp_route`]'s `stamp_dabs_smear`.
//!
//! Unlike Smear, Blur is **stationary per dab** — it needs no motion (a single click softens the
//! footprint), so there is no lift-source to chain across pointer batches. Everything else mirrors the
//! Smear route: the cached [`ph2d_painter_brush::StampMask`] for a View-static dab; the per-pixel Grain
//! path ([`ph2d_painter_brush::blur_blit_grain`]) for a canvas-fixed Grain Mapping (Tiled/Stencil) or
//! per-dab rotation (Rake / Random / Jitter Rotate); the plain round falloff otherwise. **Tiling**
//! replicates each dab across the enabled sprite edges (the neighbourhood read wraps toroidally too).

use super::{Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::{BlurKernel, BrushSpec, Dab};

/// A região que o lote de Blur escreve, **com as cópias do Tiling** (o wrap é feito AQUI, não na lista):
/// a união das pegadas `[c ± r]` de cada dab em cada deslocamento. `r ≥ 0,5` porque o `blur_dab` sobe o
/// raio a esse piso (`clamped_radius`); o `dab_write_bounds` soma o pad, logo é superconjunto do que os
/// três núcleos escrevem (`footprint_bbox`).
fn blur_bounds(
    dabs: &[Dab],
    w: u32,
    h: u32,
    source_size: (u32, u32),
    tiling: [bool; 2],
) -> Option<Region> {
    lote_bounds(dabs, w, h, source_size, tiling, 1.0)
}

/// [`blur_bounds`] com o alcance da ponta `reach` (o Clone: `BrushSpec::tip_reach`).
pub(super) fn lote_bounds(
    dabs: &[Dab],
    w: u32,
    h: u32,
    source_size: (u32, u32),
    tiling: [bool; 2],
    reach: f32,
) -> Option<Region> {
    let mut acc: Option<Region> = None;
    let mut offs = [[0.0f32; 2]; 9];
    for d in dabs {
        let n = if tiling[0] || tiling[1] {
            super::tiling::tiled_offsets_into(d.center, d.radius_px, source_size, tiling, &mut offs)
        } else {
            offs[0] = [0.0, 0.0];
            1
        };
        for off in &offs[..n] {
            let c = [d.center[0] + off[0], d.center[1] + off[1]];
            let Some(r) =
                ph2d_painter_brush::dab_write_bounds(c, d.radius_px.max(0.5), [reach, reach], w, h)
            else {
                continue;
            };
            let r = Region {
                x: r.x,
                y: r.y,
                w: r.w,
                h: r.h,
            };
            acc = Some(acc.map_or(r, |a| union_region(a, r)));
        }
    }
    acc
}

/// A pegada de um lote que LEVANTA a tela (Blur, Clone) — uma decisão só para os dois (B25.6): a máscara em
/// cache, o grão por píxel ou o redondo.
pub(super) struct PegadaQueLevanta {
    pub(super) want_mask: bool,
    pub(super) want_grain: bool,
    /// A rampa do Grain como TOM de cobertura (sem cor pintada).
    pub(super) grain_tone: Option<Vec<f32>>,
}

impl PainterTool {
    /// A rota da pegada de um lote de Blur ou Clone sob o spec `base` (o grão que pinta). Per-dab-rotating
    /// Shape/Grain (Rake / Random), Jitter Rotate ou a caneta DEITADA não são invariantes à escala → o
    /// caminho do grão por píxel; View-static → a máscara em cache; sem textura → o redondo. A ponta deitada
    /// conta como textura: o redondo (`blur_dab`/`clone_dab`) não lê a pegada do dab, e o Blur pintava-a
    /// redonda onde o Clone a deitava (W6). As rampas de Shape/Grain são TONS aqui; a máscara em cache não
    /// os leva ⇒ por píxel.
    pub(super) fn pegada_que_levanta(
        &mut self,
        base: &BrushSpec,
        dabs: &[Dab],
    ) -> PegadaQueLevanta {
        let has_shape_image = self.paint.shape_image.is_some();
        let deitados = ph2d_painter_brush::pen::any_tilted(dabs);
        let textured = base.shape_silhouette_active(has_shape_image)
            || base.texture.is_active()
            || base.pegada_anisotropica()
            || deitados;
        let per_dab_rotation = base.has_per_dab_rotation()
            || base.shape_has_per_dab_rotation(has_shape_image)
            || deitados;
        self.ensure_shape_ramp_lut();
        let grain_tone = self.grain_tone_lut();
        let ramp_tone_active = self.shape_tone_lut_slice().is_some() || grain_tone.is_some();
        let want_mask = textured
            && base.dab_mask_cacheable(has_shape_image)
            && !per_dab_rotation
            && !ramp_tone_active;
        PegadaQueLevanta {
            want_mask,
            want_grain: textured && !want_mask,
            grain_tone,
        }
    }

    /// Blur every dab's footprint into `canvas_rgba`, routed like the paint path (see the module doc).
    /// Blur amount = brush Strength × per-dab coverage (pressure). The engine snapshots each dab's
    /// neighbourhood before writing, so overlapping read/write never feeds back within a dab.
    pub(super) fn stamp_dabs_blur(&mut self, dabs: &[Dab], w: u32, h: u32) {
        self.stamp_dabs_blur_com(dabs, w, h, BlurKernel::Binomial);
    }

    /// O mesmo passe com o NÚCLEO escolhido por quem chama.
    ///
    /// ⚠️ **Duas portas e não uma bandeira, e é ordem do dono** (2026-09-20: *«só no Blur do
    /// composite; o Blur como ferramenta isolada não deve ser modificado»*): a porta de cima é a da
    /// ferramenta isolada e crava o binomial, logo o caminho dela é **byte-idêntico por
    /// construção** — não por alguém se lembrar de passar o argumento certo. Um chamador novo tem
    /// de escolher, e o que ele escolher fica escrito no sítio da chamada.
    pub(super) fn stamp_dabs_blur_com(&mut self, dabs: &[Dab], w: u32, h: u32, nucleo: BlurKernel) {
        if dabs.is_empty() {
            return;
        }
        // O grão com que o motor PINTA (o «Same as Paper» troca-o pelo papel, como no traço) e a imagem dele.
        let (base, _) = self.grao_que_pinta();
        let papel = self.papel_como_grain();
        let has_shape_image = self.paint.shape_image.is_some();
        // A pegada do lote pela porta comum do Blur e do Clone ([`Self::pegada_que_levanta`]).
        let PegadaQueLevanta {
            want_mask,
            want_grain,
            grain_tone,
        } = self.pegada_que_levanta(&base, dabs);
        let grain_active = base.texture.is_active();
        let shape_active = base.shape_silhouette_active(has_shape_image);
        let tiling = self.paint.tiling;
        let tiled = tiling[0] || tiling[1];
        let source_size = self.source_size;

        // Gather the weight source BEFORE the buffer borrow (disjoint `self.paint` fields vs
        // `self.canvas_rgba`): the cached mask (View), or the Grain/Shape images for the per-pixel path.
        // The Shape tone LUT is cloned — it borrows `self`, which the `canvas_rgba` write can't co-hold.
        if want_mask {
            let max_r = dabs.iter().map(|d| d.radius_px).fold(0.0_f32, f32::max);
            self.ensure_stamp_cache(&base, super::stamp_cache::mask_size_for(max_r));
        }
        let mask = want_mask
            .then(|| self.paint.stamp_cache.as_ref().map(|(m, _)| m))
            .flatten();
        let grain_img = want_grain
            .then(|| {
                if papel {
                    &self.paint.paper_image
                } else {
                    &self.paint.texture_image
                }
                .as_ref()
                .map(|i| i.as_mask())
            })
            .flatten();
        let shape_img = want_grain
            .then(|| self.paint.shape_image.as_ref().map(|i| i.as_mask()))
            .flatten();
        let shape_lut: Option<Vec<f32>> = if want_grain {
            self.shape_tone_lut_slice().map(|s| s.to_vec())
        } else {
            None
        };

        let mut tex_rng = self.paint.tex_rng;
        let buf = crate::tool::paint::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            blur_bounds(dabs, w, h, source_size, tiling),
        );
        let mut touched: Option<Region> = None;
        for d in dabs {
            // The dab's coverage already carries the Strength, once (BUGS #41).
            let amount = d.coverage;
            // Per-dab frame for the Grain path: the Jitter-Rotate footprint + the Rake heading (`d.dir`)
            // + the Random draw (`tex_rng`), computed ONCE per dab so the wrapped Tiling copies share it.
            let fp = base.dab_footprint_of(d);
            let sbasis = (want_grain && shape_active).then(|| {
                ph2d_painter_brush::texture::shape_basis(
                    &base.shape,
                    &mut tex_rng,
                    [w as f32, h as f32],
                    fp,
                    ph2d_painter_brush::texture::ShapeFrame::Stroke {
                        arc_len: d.arc_len,
                        unit_px: d.stroke_radius_px,
                    },
                )
            });
            let gbasis = (want_grain && grain_active).then(|| {
                ph2d_painter_brush::texture::dab_basis(
                    &base.texture,
                    &mut tex_rng,
                    [w as f32, h as f32],
                    fp,
                )
            });
            // Tiling wrap offsets (applied to the dab centre); stack buffer, no alloc.
            let mut offs = [[0.0f32; 2]; 9];
            let n = if tiled {
                super::tiling::tiled_offsets_into(
                    d.center,
                    d.radius_px,
                    source_size,
                    tiling,
                    &mut offs,
                )
            } else {
                1
            };
            for &off in &offs[..n] {
                let c = [d.center[0] + off[0], d.center[1] + off[1]];
                let dirty = if let Some(m) = mask {
                    ph2d_painter_brush::blur_blit_stamp(
                        buf,
                        w,
                        h,
                        c,
                        d.radius_px,
                        m,
                        amount,
                        tiling,
                        nucleo,
                    )
                } else if want_grain {
                    ph2d_painter_brush::blur_blit_grain(
                        buf,
                        w,
                        h,
                        c,
                        d.radius_px,
                        &base,
                        fp,
                        gbasis.as_ref(),
                        sbasis.as_ref(),
                        grain_img.as_ref(),
                        shape_img.as_ref(),
                        shape_lut.as_deref(),
                        grain_tone.as_deref(),
                        amount,
                        tiling,
                        nucleo,
                    )
                } else {
                    ph2d_painter_brush::blur_dab(
                        buf,
                        w,
                        h,
                        c,
                        &BrushSpec {
                            radius_px: d.radius_px,
                            ..base
                        },
                        amount,
                        tiling,
                        nucleo,
                    )
                };
                if let Some(r) = dirty {
                    let rect = Region {
                        x: r.x,
                        y: r.y,
                        w: r.w,
                        h: r.h,
                    };
                    touched = Some(touched.map_or(rect, |acc| union_region(acc, rect)));
                }
            }
        }
        self.paint.tex_rng = tex_rng;
        self.declare_wrote(touched);
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }
}
