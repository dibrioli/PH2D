//! **A RECONSTRUÇÃO DA SILHUETA** — como um texel de borda da lavagem vira uma fração de cobertura.
//!
//! Saiu do [`super::watercolor_field`] pelo teto de LOC, e o corte é por ASSUNTO: aquele arquivo
//! responde *"como se amostra e se borra um campo"* (samplers, blur, ruído, estado de sessão); este
//! responde *"onde há uma silhueta, e que fração dela este texel contém"*. O irmão de testes
//! [`super::watercolor_aa_tests`] já morava separado pela mesma razão.
//!
//! Tudo aqui é re-exportado pelo `watercolor_field`, então nenhum caminho de chamador muda.

use super::watercolor_field::{sample_bilinear, smoothstep};

/// Bilinear sample **plus** the field's screen-space gradient magnitude at `(fx, fy)`, in field units
/// per texel — the `fwidth` estimate `|∂/∂x| + |∂/∂y|` of the same bilinear patch (reads the SAME four
/// texels as [`sample_bilinear`], value bit-identical to it). [`aa_coverage`] uses the gradient to
/// decide whether the neighbourhood is a transition at all (flat ⇒ single sample, byte-identical).
#[inline]
pub(super) fn sample_bilinear_grad(
    src: &[f32],
    w: usize,
    h: usize,
    fx: f32,
    fy: f32,
) -> (f32, f32) {
    let fx = fx.clamp(0.0, (w - 1) as f32);
    let fy = fy.clamp(0.0, (h - 1) as f32);
    let x0 = fx.floor() as usize;
    let y0 = fy.floor() as usize;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;
    let a = src[y0 * w + x0];
    let b = src[y0 * w + x1];
    let c = src[y1 * w + x0];
    let d = src[y1 * w + x1];
    let top = a + (b - a) * tx;
    let bot = c + (d - c) * tx;
    let val = top + (bot - top) * ty;
    // Analytic gradient of the bilinear interpolant, per texel (∂x holds y, ∂y holds x).
    let dcdx = (b - a) * (1.0 - ty) + (d - c) * ty;
    let dcdy = (c - a) * (1.0 - tx) + (d - b) * tx;
    (val, dcdx.abs() + dcdy.abs())
}

/// Sub-texel offsets of the edge-reconstruction grid (3×3, spanning ±0.667 texel). Wider than the
/// unit ±0.5 box on purpose: the watercolor silhouette's HARDENED coverage crosses `[e0, e1]` in well
/// under one texel on a thin stroke (`feather` rim + `smoothstep`), so a unit box barely reaches across
/// it — a ~1.3-texel footprint reconstructs the sub-texel step as a soft ramp of about the plain
/// painter's edge width. `LITERAL-PX-OK`: AA reconstruction geometry.
const AA_SS: [f32; 3] = [-0.667, 0.0, 0.667]; // LITERAL-PX-OK

/// **O VÃO mínimo que faz de uma vizinhança uma SILHUETA** — o quanto a cobertura ENDURECIDA precisa
/// variar através do footprint [`AA_SS`] para que exista uma fronteira a reconstruir.
///
/// ⚠️ **Ele existe porque o portão anterior era `grad > 0`, e o doc do [`aa_coverage`] admite que
/// esse dispara em toda parte** (*"the feather's plateau scallop keeps `grad > 0` across virtually
/// the whole wash"*) — a frase estava escrita como tranquilizante e era o defeito (Enio 2026-08-11:
/// *"Smooth Edges + Dilution > 0 empurra pixels duros para além das bordas do traço"*). Sobre o CORPO
/// da lavagem, `mx` é uma **DILATAÇÃO 3×3** (um filtro de máximo), cujo contorno segue a grade
/// discreta ⇒ escadinha por construção; e `ss / mx` é a razão de duas estatísticas da mesma
/// vizinhança, que degrau junto.
///
/// ⚠️ **A `Dilution` decide se isso é VISÍVEL, e é por isso que o report a nomeia:**
/// `flow = 1 − dilution`, então a 0,45 o corpo da lavagem pousa em ~0,55 — **dentro** da janela
/// `[SS0 = 0,12 · SS1 = 0,60]` em vez de saturado acima dela —, e o que era `1` em toda parte passa a
/// variar. A varredura confirma o ARCO que a mecânica prevê: poucos degraus em Dilution 0 (corpo
/// acima da janela), pico em 0,45-0,60 (dentro), zero em 0,90 (abaixo de `SS0`, nada renderiza).
///
/// ⚠️ **E o portão NÃO pode ser *"o footprint toca o lado de fora"*, que foi a primeira tentativa e
/// MORREU medida:** num rim SUAVE nenhuma das nove amostras sai da lavagem, e o AA é preciso lá
/// mesmo assim — porque quem endurece a borda grossa é o modelo ÓPTICO, não a cobertura (o doc acima
/// já dizia isto). Com aquele portão os dois modos rendiam **byte-idênticos** e quatro gates caíam.
///
/// **O número é o joelho MEDIDO das duas restrições** (cliffs no INTERIOR de uma lavagem, traço
/// vertical r=26; o AA-off é `D0: 5` e `D0,45: 0`):
///
/// ```text
///     T      gates de AA      D 0,00      D 0,45
///   0,02      9 verdes        13           42
///   0,08      9 verdes        11           25
///   0,12      9 verdes        10            7
///   0,16      9 verdes         9            1
///   0,20      9 verdes         6            0     <- o interior iguala o AA-off
///   0,30      1 VERMELHO       -            -
///   0,45      2 VERMELHOS      -            -
/// ```
///
/// Abaixo de 0,20 o corpo ainda degraus; a partir de 0,30 o portão começa a comer rim de verdade.
/// `LITERAL-PX-OK`: fração de cobertura endurecida, não um comprimento.
pub(super) const AA_SPAN_MIN: f32 = 0.20; // LITERAL-PX-OK

/// The hardened silhouette coverage `smoothstep(e0, e1, coverage)` at `(sx, sy)` **plus the screen-space
/// AA alpha the composite must apply to the finished pixel** (Enio 2026-07-20, "borda dura pixelada" em
/// traço fino). Returns `(cw, aa_alpha)`.
///
/// Two findings shaped this (both measured on rendered pixels):
/// - A thin stroke's silhouette crosses the hardening window `[e0, e1]` in ~a texel, and the OPTICAL
///   model downstream is exponential — the edge-darkening fringe + Beer–Lambert saturate to full dark
///   at small `cw`, so even a 2-texel coverage ramp renders as a binary cliff (radius 10: `255, 190, 1`).
///   Feeding an anti-aliased `cw` into the density is therefore NOT anti-aliasing: the exponential eats
///   the fraction. The fraction must be applied as **linear alpha on the finished pixel** — shading may
///   saturate all it wants; the blend against the paper is linear in coverage (the classic rasterizer
///   split of shape × shading).
/// - The fraction itself needs sub-texel reconstruction ([`AA_SS`] supersampling): on very thin strokes
///   the hardened coverage jumps 0→1 inside one texel, so a single sample has no fraction to offer.
///
/// The treatment applies to **every transition** (thick strokes included — the second smoke's order:
/// the saturation steepens the thick rim's perceived edge too, and the AA'd thin strokes came out
/// "melhores que traços grossos"); a neighbourhood that carries no SILHOUETTE — a flat field
/// (`grad == 0`: open paper) or one whose hardened coverage varies by less than [`AA_SPAN_MIN`]
/// across the footprint (the wash's own body) — takes the single sample and is byte-identical.
/// ⚠️ **Esta segunda metade chegou em 2026-08-11 e a frase anterior era o defeito:** ela dizia
/// *"only a genuinely FLAT neighbourhood"*, e o parêntese no fim deste doc admite que o *scallop*
/// mantém `grad > 0` **em toda a lavagem** — ou seja o portão nunca fechava, e sobre uma lavagem
/// DILUÍDA (cujo corpo pousa dentro de `[e0, e1]` em vez de saturado) as duas estatísticas abaixo
/// degrauavam o miolo. Ver [`AA_SPAN_MIN`] para o mecanismo, a varredura e a tentativa que morreu.
/// It is halo-free: nothing widens the window, so a fully-outside texel stays exactly `(0, …)` = paper.
/// `pos(ox, oy)` maps a sub-texel OUTPUT offset to the coverage-space sample position — the caller
/// routes it through the full Ragged-Edge warp (`pos(0,0)` must be the pixel's own warped centre), so
/// the supersamples span the output texel's TRUE footprint. Offsetting in warped space instead reads
/// a footprint far too small under a strong warp (adjacent output texels' warped positions sit up to
/// `1 + amp·0.19` texels apart — measured over a 300² sweep of `warp_offset`), and the serrated edge
/// stayed binary: warp 48 posted 226 cliffs, warp 32 posted 75, with the flat fixtures all green.
/// Routing the taps through the warp took both to zero. (A four-probe "rescue" for centres landing on
/// flat spots was built alongside and MEASURED DEAD — the feather's plateau scallop keeps `grad > 0`
/// across virtually the whole wash, so the gradient gate already fires everywhere it matters.)
#[inline]
pub(super) fn aa_coverage(
    src: &[f32],
    w: usize,
    h: usize,
    pos: impl Fn(f32, f32) -> (f32, f32),
    e0: f32,
    e1: f32,
) -> (f32, f32) {
    let (sx, sy) = pos(0.0, 0.0);
    let (val, grad) = sample_bilinear_grad(src, w, h, sx, sy);
    let single = smoothstep(e0, e1, val);
    // Flat field (the wash interior's plateau, open paper): one sample, no alpha — byte-identical.
    // Every TRANSITION gets the treatment (Enio 2026-07-20 pós-smoke: os finos ficaram "melhores que
    // traços grossos" — the optical saturation steepens the thick rim too, so the AA is for every
    // stroke, not a thin-stroke rescue; the original steepness gate was retired on that order).
    if grad <= 0.0 {
        return (single, 1.0);
    }
    let mut acc = 0.0;
    let mut mx = single;
    let mut mn = single;
    for &oy in &AA_SS {
        for &ox in &AA_SS {
            let (tx, ty) = pos(ox, oy);
            let c = smoothstep(e0, e1, sample_bilinear(src, w, h, tx, ty));
            acc += c;
            mx = mx.max(c);
            mn = mn.min(c);
        }
    }
    let ss = acc * (1.0 / (AA_SS.len() * AA_SS.len()) as f32);
    if mx <= 0.0 {
        // Wholly outside the silhouette: nothing to fade (cw = 0 early-outs downstream anyway).
        return (single, 1.0);
    }
    // ⚠️ **Uma vizinhança sem VÃO não é uma silhueta** — e sobre o corpo de uma lavagem DILUÍDA as
    // duas estatísticas abaixo (uma dilatação 3×3 e a razão de duas estatísticas dela) viram degraus
    // de tinta. O porquê, a varredura e a tentativa que morreu medida estão no [`AA_SPAN_MIN`].
    if mx - mn < AA_SPAN_MIN {
        return (single, 1.0);
    }
    // The rasterizer split, shape × shading: the SHADING is what the covered fraction of the texel
    // contains — the wash a little deeper in (the MAX subsample; using the centre sample double-fades:
    // a rim texel then renders the diluted light wash AND gets alpha-faded, while its inner neighbour
    // is already optically saturated — the cliff just moves over by one texel). The SHAPE is the
    // fractional area **relative to the wash level present** (mean ÷ max): a diluted wash's body sits
    // mid-band, where the feather's plateau scallop keeps a tiny gradient alive — the raw mean would
    // alpha-fade the whole interior (~0.8) and stair-step the owner junction (the cross gate caught
    // it); against the local max the interior ratios to ~1 while a true silhouette edge stays the
    // honest fraction. At a full-strength rim `mx ≈ 1`, so this is the approved thin-stroke fade.
    (mx, ss / mx)
}
