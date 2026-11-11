//! **Impasto** — the light pass: the relief made visible.
//!
//! The height field ([`super::impasto`]) is the *material*; this is the *look*. A surface normal is
//! taken from the height by central differences and shaded (Lambert + a Blinn-Phong highlight), and
//! the result modulates the composited pixels.
//!
//! ## The one property everything hangs on: flat paint is UNTOUCHED
//!
//! The shading is **relative**, not absolute: a pixel's diffuse response is divided by the response
//! of a *flat* surface, and the specular has the flat surface's own highlight subtracted. So where
//! the canvas has no relief the pass multiplies by exactly 1 and adds exactly 0 — the pixels come out
//! **byte-identical**.
//!
//! That is not a nicety, it is the whole contract. An absolute shading model (the naive
//! `rgb × (N·L)`) would darken the entire painting the moment the light came on, because a flat
//! surface lit from 45° returns 0.7, not 1. Half the emboss filters ever written have that bug. It
//! also gives the gates their teeth: "no relief ⇒ not one byte moves" is checkable, and it is checked.
//!
//! ## Why the pass runs on a freshly-composited region
//!
//! Lighting is spatially non-local (a pixel's normal reads its neighbours), and it is **not**
//! idempotent — lighting an already-lit pixel lights it twice. The dirty-rect fast lane re-composites
//! its region from the layers (un-lit) and lights *that*, so the cached composite holds lit pixels but
//! every one of them was lit exactly once, from scratch. Applying the pass in place on the cache would
//! have been the obvious thing, and it would have compounded the shading a little more every frame.

use super::Region;
use super::impasto_luz_dobra::{PorBytes, dobra_da_faixa, sombreia_faixa};
use super::impasto_shade::Rig;
#[cfg(doc)]
use super::relief_fields::ReliefFields;
use crate::layers::ReliefComposite;
use crate::tool::PainterTool;
use ph2d_painter_brush::material::MaterialBytes;
use rayon::prelude::*;

/// Linhas por faixa do passe de luz ([`PainterTool::apply_impasto_light`]): 2 linhas de moldura por faixa (+6 %),
/// 68 faixas numa tela 4K para o pool.
pub(super) const FAIXA_LINHAS: usize = 32;

/// How many CANVAS PIXELS of physical paint height `h = 1.0` (one full-Depth stroke) represents.
///
/// This is the industry's decomposition of the height-to-slope gain — *a physical height over the
/// texel size*, the Blender Bump node's `Distance`, Substance's "Height Depth (cm)", HDRP's
/// amplitude-in-centimetres (`docs/Painter/17_impasto_deposito_pesquisa2.md` §4) — instead of a
/// unitless fudge. The slope is then geometry: `∇h × DEPTH_UNIT_PX` height-pixels per pixel, no
/// gain knob multiplying it (Substance Painter exposes none either; the deposit value is the knob).
///
/// The number itself is perceptual and MEASURED on the probe (`probe`-family tests): the default
/// soft brush at r=40 lays its wall over the falloff's `W_TAIL..W_SOLID` band, ~11 px — so at 16,
/// a Depth 0.7 stroke is ~11 px of paint falling over ~11 px of wall: the classic 45° bevel, and
/// the probe's shading peak sits ON that wall (42% → edge band) instead of inside the stroke. It
/// has a way to be wrong now: if strokes read taller or flatter than `DEPTH_UNIT_PX` pixels of
/// real paint would, this constant is lying. // CLAMP-OK
pub(super) const DEPTH_UNIT_PX: f32 = 16.0;

// (The Blinn-Phong exponent used to be `const SHININESS: f32 = 24.0` right here. It is now the paint's
// **Roughness** — `ph2d_painter_brush::material` — because how BROAD a highlight is, is a property of
// the surface, not of the renderer. The old constant survives exactly, as the neutral material: the
// roughness→exponent map is geometric between 6 and 96 and `√(6·96) = 24`, so a default brush shades
// byte-identically to the build that had the constant. `Material::NEUTRAL` and its gates pin that.)

/// **Ambient** floor of the diffuse term: what a face turned fully AWAY from the light still returns.
///
/// ⚠️ O número mora em [`ph2d_light`] junto com o rig: ele é **lei do modelo relativo**, não
/// material. O porquê do valor está lá.
pub(super) use ph2d_light::AMBIENT;

/// How the relief's effect scales with how much PAINT is at a pixel.
///
/// The pass multiplies the composited pixel — and at a stroke's translucent edge that pixel is mostly
/// **paper showing through the paint**. Shading it in full means shading the paper: on a white canvas
/// that bleach was the halo Enio photographed (2026-07-12; 81 levels at the 20–60%-ink edge vs 55 at
/// the core, gone on black canvas). So the effect is weighted by the paint's own coverage, and bare
/// paper gets exactly none — the pass stays a strict no-op there.
///
/// ## The weight IS the coverage — because the coverage is now the SOLID paint
///
/// It used to be `body_profile(cover)` over a `cover` that held the RAW paint (silhouette × dynamics),
/// and that put the dynamics INSIDE the body curve, where they could starve it: at
/// Flow × Strength × pressure ≈ `W_TAIL` the argument falls under the tail for every texel and the light
/// models **nothing anywhere on the stroke** — while the pigment, cut on the silhouette
/// ([`ph2d_painter_brush::height::film_coverage`]), is still perfectly there. Enio's haze, hiding behind
/// the mouse (which always presses at 1.0).
///
/// The layers now store the **solid paint** itself
/// ([`ph2d_painter_brush::height::solid_paint`] — `dynamics × body_profile(silhouette)`, the film's own
/// alpha), so the curve is already in the number and the weight is the number. At full dynamics the two
/// are the same value, so every stroke a mouse ever drew — and every gate that pins the look — is
/// unchanged; under a light touch the light now models a thinner film instead of refusing to see it.
///
/// One definition of "solid paint", still, on both sides of the pipeline: nothing over the stain
/// (`≤ W_TAIL` of silhouette — where the wall does not stand, the light does not push, and the brush no
/// longer lays pigment either), full over solid paint (`≥ W_SOLID`), the ramp between.
#[inline]
pub(super) fn paint_body(cover: f32) -> f32 {
    cover
}

/// How the SPECULAR scales with the paint.
///
/// It is the **same body curve** the diffuse uses ([`paint_body`]) — and that is not laziness, it is
/// the only place the glint can live. The relief's slope exists exactly where the wall is, i.e. over
/// the coverage band `W_TAIL..W_SOLID`; gating the highlight *above* `W_SOLID` (the first cut, to kill
/// the halo) therefore allowed it only on the plateau, which is FLAT — the pass early-outs there and
/// adds nothing. Measured: Shine 0 → 1 moved the brightest pixel by **1 level**. A knob that does
/// nothing (Enio: *"shine não funciona"*), and the exact species the 2026-07-12 sweep exterminated.
///
/// The halo the gate is guarding against is a bleach of the PAPER seen through translucent paint, and
/// the body curve already refuses that: it is zero over the stain and only reaches full on solid
/// paint. So the glint climbs the wall with the body and peaks on the crest — where oil paint glints —
/// while the rim, which has barely any body, gets barely any highlight.
///
/// (`impasto_shine_glints_on_the_wall_without_bleaching_the_rim` pins BOTH halves: the glint must be
/// visible, and it must not bleach. Fixing one by breaking the other is how this knob died the first
/// time.)
#[inline]
pub(super) fn gloss_body(cover: f32) -> f32 {
    paint_body(cover)
}

/// One layer's contribution to the composed relief, in z-order.
pub(super) struct ReliefLayer<'a> {
    /// A camada — a chave da dobra através dos ajustes.
    pub(super) id: crate::layers::LayerId,
    /// The committed height plane. `None` for the active layer when it has only a live stroke on it.
    pub(super) height: Option<&'a [f32]>,
    /// Its paint coverage, when it has one.
    pub(super) cover: Option<&'a [u8]>,
    /// Its MATERIAL plane, when it has one. `None` = a layer sculpted before materials existed; it
    /// reads as `Material::NEUTRAL`, which is the pass exactly as it shaded then.
    pub(super) mat: Option<&'a [MaterialBytes]>,
    /// The layer's own **Impasto depth** (`Layer::impasto_depth`) — `1` composites as sculpted, `0`
    /// mutes, negative inverts the relief.
    pub(super) depth: f32,
    /// How this layer meets the relief below it.
    pub(super) composite: ReliefComposite,
    /// This is the ACTIVE layer, so the open stroke rides on it (`live_h`/`live_c` fold in here — at
    /// this layer's z, under this layer's depth, not on top of the whole pile).
    pub(super) active: bool,
}

/// One resolved lamp: its direction, its half-vector, and what a FLAT surface returns to it. The whole
/// pass is relative to that flat response — see [`Rig`].
impl PainterTool {
    /// Whether the light pass has anything to do: it is switched on and some layer carries relief.
    /// Cheap enough to call per frame — the height map is empty for every document nobody has sculpted.
    #[must_use]
    pub fn impasto_visible(&self) -> bool {
        self.substrate().is_some()
            || (self.paint.impasto_show
                && (!self.heights.is_empty() || !self.paint.relief.stroke_height.is_empty()))
    }

    /// Light `rgba` — the pixels of `region`, freshly composited and NOT yet lit (`rgba` is
    /// `region.w × region.h × 4`, straight sRGB8). No-op when the pass is off or nothing has relief.
    ///
    /// **This is the canonical pass**, and it is public because it is what the GPU port is reconciled
    /// against ([`super::impasto_gpu`] + `ph2d_render::ImpastoLightPass`). A parity gate that compared the
    /// shader to a re-implementation of this would agree with the bug; it compares against THIS.
    pub fn apply_impasto_light(&self, rgba: &mut [u8], region: Region) {
        self.acende_em_faixas(rgba, region, FAIXA_LINHAS);
    }

    /// [`Self::apply_impasto_light`] com `linhas` por faixa (a sonda mede 32 × 64 no MESMO binário).
    pub(super) fn acende_em_faixas(&self, rgba: &mut [u8], region: Region, linhas: usize) {
        self.acende_contando(rgba, region, linhas, true);
    }

    /// O passe por faixas; devolve os texels que DOBROU (o contador é o gate do B15.2). `podar`: sem papel nem filtro
    /// no relevo, cada faixa só dobra as colunas onde a diferença central pode não ser zero ([`colunas_com_relevo`]) —
    /// fora delas a altura composta é `±0` nos três vizinhos, o `shade` devolve `(1, 0)` e o píxel não muda: o mesmo bit.
    pub(super) fn acende_contando(
        &self,
        rgba: &mut [u8],
        region: Region,
        linhas: usize,
        podar: bool,
    ) -> usize {
        if !self.impasto_visible() {
            return 0;
        }
        let Some(fields) = self.impasto_fields_para(region) else {
            return 0;
        };
        let (w, h) = (region.w as usize, region.h as usize);
        if w == 0 || h == 0 || rgba.len() < w * h * 4 {
            return 0; // shape guard — bail rather than index out of a mis-sized buffer
        }
        let Some(light) = Rig::new(&self.paint.impasto_rig) else {
            return 0; // every lamp switched off: the canvas comes back unlit, to the byte
        };
        // B15.2: os planos que fazem a altura composta; a poda só vale quando eles são TUDO o que ela lê.
        let planos: Option<Vec<&[f32]>> =
            (podar && fields.substrate.is_none() && fields.folded.is_none()).then(|| {
                fields
                    .layers
                    .iter()
                    .filter_map(|l| l.height)
                    .chain(fields.live_h)
                    .collect()
            });
        // ⭐ **Por FAIXAS de linhas, no pool** (dono 10/11: o pen-up a 4K travava 1,6 s — o `height_at` com o dente
        // do papel, amostrado 4× por píxel pelas diferenças centrais, num fio só). Cada faixa lê a altura UMA vez por
        // píxel numa moldura de 1 px e difere dentro dela: o mesmo `height_at` nos mesmos píxeis, logo o mesmo bit
        // (`as_faixas_dao_o_passe_por_pixel_ao_bit`). Píxeis independentes; a cache do material é por faixa.
        // B15.6: as linhas da faixa vêm da MESMA dobra que os planos da placa ([`ReliefFields::dobra_da_linha`]):
        // altura, corpo e material num passeio só; as duas linhas de moldura só precisam da altura.
        let paper_body = fields.paper_body();
        let (x0, y0) = (i64::from(region.x), i64::from(region.y));
        let tela = (fields.width, fields.height);
        rgba[..w * h * 4]
            .par_chunks_mut(w * 4 * linhas)
            .enumerate()
            .map(|(b, faixa)| {
                let n = faixa.len() / (w * 4);
                let gy0 = y0 + (b * linhas) as i64;
                let (a, z) = match &planos {
                    None => (0, w),
                    Some(p) => match colunas_com_relevo(p, tela, gy0, n, region.x as usize, w) {
                        Some(c) => c,
                        None => return 0, // a faixa inteira é plana: nada a dobrar
                    },
                };
                let sw = z - a;
                let fw = sw + 2;
                let mut alt = vec![0.0f32; fw * (n + 2)];
                let mut cob = vec![0.0f32; fw * n];
                let mut mats: Vec<MaterialBytes> = vec![[0; 7]; fw * n];
                let mut pap = super::impasto_luz_dobra::separa_o_papel(&fields)
                    .then(|| vec![0.0f32; alt.len()]);
                dobra_da_faixa(
                    &fields,
                    gy0,
                    x0 + a as i64,
                    fw,
                    &mut alt,
                    pap.as_deref_mut(),
                    &mut cob,
                    &mut mats,
                );
                let cache =
                    light.resolve(ph2d_painter_brush::material::Material::NEUTRAL.to_bytes());
                let fonte = PorBytes { mats: &mats, cache };
                if sw == w {
                    sombreia_faixa(
                        &light,
                        paper_body,
                        &alt,
                        pap.as_deref(),
                        &cob,
                        faixa,
                        w,
                        fonte,
                    );
                } else {
                    let mut sub: Vec<u8> = faixa
                        .chunks_exact(w * 4)
                        .flat_map(|l| l[a * 4..z * 4].iter().copied())
                        .collect();
                    sombreia_faixa(
                        &light,
                        paper_body,
                        &alt,
                        pap.as_deref(),
                        &cob,
                        &mut sub,
                        sw,
                        fonte,
                    );
                    for (l, s) in faixa.chunks_exact_mut(w * 4).zip(sub.chunks_exact(sw * 4)) {
                        l[a * 4..z * 4].copy_from_slice(s);
                    }
                }
                n * sw
            })
            .sum()
    }

    /// O passe POR PÍXEL de antes das faixas (10/11), verbatim: o oráculo ao bit de [`Self::apply_impasto_light`].
    #[cfg(test)]
    pub(super) fn apply_impasto_light_por_pixel(&self, rgba: &mut [u8], region: Region) {
        if !self.impasto_visible() {
            return;
        }
        let Some(fields) = self.impasto_fields() else {
            return;
        };
        if rgba.len() < (region.w as usize) * (region.h as usize) * 4 {
            return; // shape guard — bail rather than index out of a mis-sized buffer
        }
        let Some(light) = Rig::new(&self.paint.impasto_rig) else {
            return; // every lamp switched off: the canvas comes back unlit, to the byte
        };
        let at = |x: i64, y: i64| fields.height_at(x, y);
        let cover_at = |x: i64, y: i64| fields.cover_at(x, y);
        // O SUBSTRATO: resolvido UMA vez (a base de rotação e o snap de Size não são baratos por-texel),
        // e `None` quando desligado — o que deixa cada linha abaixo cair no valor que já era o dela.
        // Materials are piecewise-constant across a canvas (one per stroke), so a ONE-entry cache turns
        // the per-material resolve — which now owns the flat divisor, and so is not free — into a few
        // calls per pass instead of one per texel.
        let mut mat = light.resolve(ph2d_painter_brush::material::Material::NEUTRAL.to_bytes());
        for ry in 0..region.h {
            let gy = i64::from(region.y + ry);
            for rx in 0..region.w {
                let gx = i64::from(region.x + rx);
                // Central differences — the normal reads across the region's edge into the canvas, so a
                // dirty-rect update lights its border exactly as a full recompose would.
                // ⚠️ O dente do papel JÁ está dentro do `height_at` (porta única), então a diferença
                // central o pega de graça — e o produtor de GPU, que dobra os MESMOS campos, também.
                let mut dhx = (at(gx + 1, gy) - at(gx - 1, gy)) * 0.5;
                let mut dhy = (at(gx, gy + 1) - at(gx, gy - 1)) * 0.5;
                let cover = cover_at(gx, gy);
                if super::impasto_luz_dobra::separa_o_papel(&fields) {
                    let p = |x: i64, y: i64| fields.altura_e_papel_at(x, y).1;
                    let px = (p(gx + 1, gy) - p(gx - 1, gy)) * 0.5;
                    let py = (p(gx, gy + 1) - p(gx, gy - 1)) * 0.5;
                    (dhx, dhy) =
                        super::impasto_luz_dobra::inclinacao_com_papel((dhx, dhy), (px, py), cover);
                }
                let i = ((ry as usize) * (region.w as usize) + rx as usize) * 4;
                // The pixel's own colour IS a metal's highlight, so it is an INPUT to the shade, not
                // only the thing the shade multiplies.
                let albedo = [
                    f32::from(rgba[i]) / 255.0,
                    f32::from(rgba[i + 1]) / 255.0,
                    f32::from(rgba[i + 2]) / 255.0,
                ];
                let key = fields.material_at(gx, gy);
                if key != mat.key {
                    mat = light.resolve(key);
                }
                // ⚠️ **A presença do PAPEL é `1` em toda parte, e é a única exceção honesta à regra
                // *"relevo sob cobertura zero não acende"*.** A regra existe porque relevo de TINTA sem
                // tinta é relevo de nada; um papel, ao contrário, está lá — a cobertura dele é 1 por
                // definição (doc 19 §1.3 antecipou exatamente isto). Sem esta linha o dente acenderia
                // só onde já houvesse tinta, e o Digital — que não tem `covers` nenhum — não veria nada.
                let paper_body = fields.paper_body();
                let body = paint_body(cover).max(paper_body);
                // ⚠️ **O `gloss` NÃO ganha a presença do papel, e isso é medição e não descuido:** um
                // papel não tem realce especular observável (ver o ⛔ em `substrate_relief` — o realce
                // plano é subtraído e clampado, e num dente de ~1 px ele é nulo em qualquer expoente).
                // Dar-lhe presença aqui seria escrever uma linha que a mutação prova inerte.
                let (mul, add) = light.shade(&mat, body, gloss_body(cover), dhx, dhy, albedo);
                if mul == [1.0; 3] && add == [0.0; 3] {
                    continue; // flat: byte-identical, and not even a rounding trip through f32
                }
                for c in 0..3 {
                    let lit = light_pixel(albedo[c], mul[c], add[c]);
                    rgba[i + c] = (lit * 255.0 + 0.5) as u8;
                }
            }
        }
    }
}

/// As colunas `[a, z)` da faixa (relativas à região `x0..x0 + w`, linhas `gy0 .. gy0 + n`) cuja diferença central
/// PODE não ser zero: as de algum texel não nulo dos `planos` nas linhas `gy0 - 1 ..= gy0 + n` (a moldura), dilatadas
/// de 1 coluna. `None` = nenhum: a faixa é plana. Conservador por construção (a altura composta de planos nulos é `±0`).
pub(super) fn colunas_com_relevo(
    planos: &[&[f32]],
    (cw, ch): (usize, usize),
    gy0: i64,
    n: usize,
    x0: usize,
    w: usize,
) -> Option<(usize, usize)> {
    if cw == 0 || ch == 0 {
        return None;
    }
    let (c0, c1) = (x0.saturating_sub(1), (x0 + w + 1).min(cw));
    let (y_lo, y_hi) = (
        (gy0 - 1).max(0) as usize,
        ((gy0 + n as i64) as usize).min(ch - 1),
    );
    let (mut lo, mut hi) = (usize::MAX, 0usize);
    for p in planos {
        for y in y_lo..=y_hi {
            let linha = &p[y * cw + c0..y * cw + c1];
            if let Some(i) = linha.iter().position(|&v| v != 0.0) {
                lo = lo.min(c0 + i);
                hi = hi.max(c0 + linha.iter().rposition(|&v| v != 0.0).unwrap_or(i));
            }
        }
    }
    if lo == usize::MAX {
        return None;
    }
    let a = lo.saturating_sub(1).max(x0) - x0;
    let z = (hi + 2).min(x0 + w) - x0;
    (a < z).then_some((a, z))
}

/// Apply the shading to ONE channel: the diffuse **modulates**, the highlight is light **added
/// against the headroom that is left** (a screen), never a flat `+ add`.
///
/// The screen is not a taste call, it is the only form that keeps the paint's colour. Write it out:
/// `screen(v) = v·(1 − add) + add`, so for two channels of the same pixel
/// `screen(R) − screen(G) = (R − G)·(1 − add)` — the hue is EXACTLY preserved and the chroma merely
/// scales, for every `add < 1`. A flat `v + add` instead clamps: on a translucent rim the pigment's own
/// channel is already at the ceiling, so only the OTHER channels rise and the colour collapses to white
/// — the halo Enio photographed, arriving through the specular door. (It is also what a real highlight
/// does: approach white, never overshoot it.)
#[inline]
pub(super) fn light_pixel(v: f32, mul: f32, add: f32) -> f32 {
    let lit = (v * mul).clamp(0.0, 1.0);
    (lit + add * (1.0 - lit)).clamp(0.0, 1.0)
}

#[cfg(test)]
impl PainterTool {
    /// The **composed** relief at a canvas pixel — depth-scaled, mode-folded, ceiling-clamped: exactly
    /// the number the light reads.
    ///
    /// Deliberately the light's OWN sampler, not a re-implementation of the fold for the gates to
    /// compare against. An oracle that re-derives the thing it is testing agrees with the bug
    /// ([[feedback_oracle_must_model_appearance_not_implementation]]); this one asks the pass what it
    /// sees. (`layer_height_view` is a different question — one layer's raw plane, before any of this.)
    pub(crate) fn composed_relief_at(&self, x: u32, y: u32) -> f32 {
        self.impasto_fields()
            .map_or(0.0, |f| f.height_at(i64::from(x), i64::from(y)))
    }

    /// O relevo composto da tela inteira e o corpo que a luz lê — o mesmo amostrador, uma só vez.
    pub(crate) fn composed_relief_plane(&self) -> (Vec<f32>, Vec<f32>) {
        let (w, h) = self.source_size;
        let Some(f) = self.impasto_fields() else {
            let n = (w as usize) * (h as usize);
            return (vec![0.0; n], vec![0.0; n]);
        };
        (0..i64::from(h))
            .flat_map(|y| (0..i64::from(w)).map(move |x| (x, y)))
            .map(|(x, y)| (f.height_at(x, y), f.cover_at(x, y)))
            .unzip()
    }
}

#[cfg(test)]
#[path = "impasto_luz_tique_tests.rs"]
mod impasto_luz_tique_tests; // B15.3: o tique da lâmpada = a recomposição inteira ao bit

#[cfg(test)]
#[path = "dobra_da_linha_tests.rs"]
mod dobra_da_linha_tests; // B15.6: a dobra de uma linha das duas pistas, ao bit com os laços de antes

#[cfg(test)]
#[path = "impasto_aro_tests.rs"]
mod impasto_aro_tests; // B31.1: a sonda do aro branco do Impasto no papel

#[cfg(test)]
#[path = "impasto_light_poda_tests.rs"]
mod impasto_light_poda_tests; // B15.2: a luz só dobra as colunas com relevo, ao bit com o passe inteiro

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_highlight_scales_chroma_and_never_annihilates_it() {
        // The algebraic heart of the specular blend. A flat additive highlight (`v + add`) destroys the
        // paint's colour on any pixel whose pigment channel is already at the ceiling — which is every
        // pixel of red paint on white paper. The screen cannot: it scales both channels by `(1 − add)`
        // and lifts both by `add`, so the DIFFERENCE between them — the pigment — survives in exact
        // proportion. RED with `let lit = v * mul + add`: the chroma below goes to 0.
        let pigment = |r: f32, g: f32| (r - g).max(0.0);
        for &add in &[0.0f32, 0.25, 0.5, 0.9] {
            // A rim pixel: red at the ceiling, green half-way (pale pink over white paper).
            let (r, g) = (1.0f32, 0.6f32);
            let (lr, lg) = (light_pixel(r, 1.0, add), light_pixel(g, 1.0, add));
            let before = pigment(r, g);
            let after = pigment(lr, lg);
            let expected = before * (1.0 - add);
            assert!(
                (after - expected).abs() < 1e-6,
                "the highlight scales the pigment by (1 − add): add {add} ⇒ {after} vs {expected}"
            );
            assert!(
                add >= 1.0 || after > 0.0,
                "…and never annihilates it (add {add} left {after})"
            );
        }
        // The other half of the contract: no glint, no light ⇒ the pixel is untouched, to the float.
        assert_eq!(light_pixel(0.42, 1.0, 0.0), 0.42);
        // And the highlight approaches white without overshooting it.
        assert!(light_pixel(0.9, 1.0, 1.0) <= 1.0);
    }
}
