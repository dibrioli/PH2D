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
/// ⚠️ **O número NÃO é escolhido pelo interior sozinho — isso foi a 1ª rodada, e ela regrediu a
/// cena do Enio.** Com a transição CONTÍNUA (ver [`aa_coverage`]) o limiar é uma **curva de troca**
/// entre duas cenas que querem coisas opostas, e as duas estão medidas (traço vertical r=26,
/// Dilution 0,45; *seco* = um 2º traço atravessando uma faixa já commitada, *interior* = degraus no
/// miolo sobre papel):
///
/// ```text
///     lei                 seco (pico)   interior (degraus)   gates de AA
///   sem portão                32,7            42              todos verdes
///   suave 0,20                32,7            17              9 de 10
///   suave 0,35                60,7             0              todos verdes
///   suave 0,50                83,7             0              todos verdes
///   DURO  0,20 (1ª rodada)    81,7             0              todos verdes
///   AA desligado             112,7             0              (o pior dos dois)
/// ```
///
/// **Nenhum limiar entrega as duas colunas** — é o que prova que o VÃO não é a variável que separa
/// os dois casos (sobre tinta seca o AA é preciso no mesmo vão em que sobre papel ele é nocivo; o
/// que difere é a BASE contra a qual o composite mistura, não a vizinhança de cobertura).
///
/// **0,20 é escolhido por ser a única linha que não é PIOR que o mundo pré-cura em nenhum dos dois
/// eixos** (seco empata em 32,7 · interior cai de 42 para 17). O `0` do interior que a 1ª rodada
/// comprava custava exatamente o pico de 81,7 na cena reportada, e *um zero pago com a outra metade
/// da tela não é um zero*. `LITERAL-PX-OK`: fração de cobertura endurecida, não um comprimento.
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
/// (`grad == 0`: open paper) or one whose hardened coverage does not vary at all across the
/// footprint (the wash's own body) — takes the single sample and is byte-identical, and **entre os
/// dois a resposta é INTERPOLADA** pelo vão medido em [`AA_SPAN_MIN`].
/// ⚠️ **Esta segunda metade chegou em 2026-08-11 e a frase anterior era o defeito:** ela dizia
/// *"only a genuinely FLAT neighbourhood"*, e o parêntese no fim deste doc admite que o *scallop*
/// mantém `grad > 0` **em toda a lavagem** — ou seja o portão nunca fechava, e sobre uma lavagem
/// DILUÍDA (cujo corpo pousa dentro de `[e0, e1]` em vez de saturado) as duas estatísticas abaixo
/// degrauavam o miolo. ⚠️ **E a PRIMEIRA tentativa de fechá-lo era um `if`, que o Enio reprovou no
/// mesmo dia** (*"diminuiu a qualidade do AA e não resolveu"*): um limiar duro sobre uma grandeza
/// contínua faz dois texels vizinhos serem calculados por leis diferentes e **fabrica** o contorno
/// que deveria remover. Ver [`AA_SPAN_MIN`] para a curva de troca medida e as tentativas mortas.
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
    //
    // ⚠️ **A transição é CONTÍNUA, e a versão binária disto era o próprio defeito** (Enio
    // 2026-08-11, 2ª rodada: *"diminuiu a qualidade do AA e não resolveu"*, pintando sobre pigmento
    // já seco). Um `if` sobre uma grandeza contínua faz dois texels vizinhos serem calculados por
    // LEIS diferentes, e o salto entre elas é um degrau que o portão **fabrica** — medido sobre
    // tinta seca a Dilution 0,45, o pico saltava de 32,7 (sem portão) para **81,7** (portão duro),
    // com a contagem de degraus praticamente igual: o que o portão duro movia era a ALTURA deles.
    // Interpolar as duas respostas remove a lei-fronteira: em vão zero é `single` exato (o interior
    // da lavagem, byte-idêntico), em vão cheio é a reconstrução inteira (o rim), e no meio nenhum
    // par de vizinhos discorda sobre qual lei os governa.
    let t = smoothstep(0.0, AA_SPAN_MIN, mx - mn);
    if t <= 0.0 {
        return (single, 1.0);
    }
    if t < 1.0 {
        return (single + (mx - single) * t, 1.0 + (ss / mx - 1.0) * t);
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
