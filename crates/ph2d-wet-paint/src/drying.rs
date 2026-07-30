//! Drying / settling / re-wetting (port of `drying.js`, SPEC §6.2) + fast dry
//! (SPEC §12).
//!
//! This pass is where the classic watercolor rim comes from: cells at the
//! edge of a wash (few suspended neighbours => low edge factor e) evaporate
//! up to ~26x faster than the interior, so pigment carried there by the flow
//! settles first and darkens the boundary. Once a cell has settled >= 1000
//! the edge boost stops — saturated washes stop over-darkening.

use crate::colorops::ColorMix;
use crate::grid::Grid;
use crate::jsmath::clamp01;
use crate::opacity::alpha_of_mass;
use crate::sim::Params;
use crate::solver::rebuild_active_region;
use crate::tuning::Knob;

#[path = "drying/edge_window.rs"]
mod edge_window;
use edge_window::EdgeWindow;

/// Staining extension: bidirectional lift multiplier, exactly 1 at 0.5.
/// `pub(crate)`: the doc-23 active lifts (Wet/Smear/Blend tools) apply the
/// SAME law, so staining means one thing everywhere.
#[inline]
pub(crate) fn staining_multiplier(s: f64) -> f64 {
    if s < 0.5 {
        1.0 + (0.5 - s) * 14.0 // 0 -> 8x
    } else {
        (1.0 - s) * 2.0 // 1 -> 0x
    }
}

/// The LIFT DOOR (SPEC §6.2 re-wet arithmetic, extracted verbatim): move
/// `sett · b` back into suspension with the opacity-composited color. ONE
/// arithmetic for every rewetter — the passive drying pass below and the
/// doc-23 active tools (Wet/Smear/Blend) — so passive and active lift can
/// never disagree about what "dissolving" means. Callers own `b` (clamped
/// 0..1) and the staining multiplier.
#[inline]
pub(crate) fn lift_settled(
    b: f64,
    susp: &mut f32,
    sett: &mut f32,
    susp_rgb: &mut [f32; 3],
    sett_rgb: [f32; 3],
    mix: ColorMix,
    out: &mut [f64; 3],
) {
    let st = *sett as f64;
    let lift = st * b;
    let a_in = alpha_of_mass(st) * b;
    let u = alpha_of_mass(*susp as f64) * (1.0 - a_in);
    if u + a_in > 0.0 {
        let w = a_in / (u + a_in);
        mix.mix(
            susp_rgb[0] as f64,
            susp_rgb[1] as f64,
            susp_rgb[2] as f64,
            sett_rgb[0] as f64,
            sett_rgb[1] as f64,
            sett_rgb[2] as f64,
            w,
            out,
        );
        *susp_rgb = [out[0] as f32, out[1] as f32, out[2] as f32];
    } else {
        *susp_rgb = sett_rgb;
    }
    *susp = (*susp as f64 + lift) as f32;
    *sett = (st - lift) as f32;
}

/// As constantes do passe, colhidas UMA vez — o que o laço quente não pode
/// re-perguntar por célula.
pub(crate) struct DryConst {
    pub retention: f64,
    pub edge_darkening: f64,
    pub base_evaporation: f64,
    pub ext_granulation: f64,
    pub ext_staining: f64,
    pub evap_base: f64,
    pub rewet_base: f64,
    pub ext_bypass: bool,
    pub mix: ColorMix,
}

impl DryConst {
    pub fn new(p: &Params, evap_base: f64, rewet_base: f64, ext_bypass: bool) -> Self {
        DryConst {
            retention: p.k(Knob::Retention),
            edge_darkening: p.k(Knob::EdgeDarkening),
            base_evaporation: p.k(Knob::BaseEvaporation),
            ext_granulation: p.k(Knob::ExtGranulation),
            ext_staining: p.k(Knob::ExtStaining),
            evap_base,
            rewet_base,
            ext_bypass,
            mix: p.mix,
        }
    }
}

/// **A ARITMÉTICA DE UMA CÉLULA — a porta única das duas rotas.**
///
/// ⚠️ Ela recebe o `count` do fator de borda em vez de o buscar, e é isso que
/// permite ao passe existir nas duas formas sem duas cópias da física: o
/// Gauss-Seidel passa a janela deslizante VIVA (que enxerga o `susp` já
/// reescrito à esquerda), o Jacobi passa o plano materializado ANTES do passe.
/// *Uma segunda cópia deste corpo é como "a secagem paralela dourou a borda
/// diferente" nasce daqui a seis meses.*
#[inline]
#[allow(clippy::too_many_arguments)]
pub(crate) fn dry_cell(
    c: &DryConst,
    paper: f32,
    count: u32,
    film: &mut f32,
    susp: &mut f32,
    sett: &mut f32,
    susp_rgb: &mut [f32; 3],
    sett_rgb: &mut [f32; 3],
    out: &mut [f64; 3],
) {
    let film0 = *film;
    let f = film0 as f64;
    let s_mass = *susp as f64;
    let mut susp_c = *susp;
    let mut sett_c = *sett;
    let mut susp_rgb_c = *susp_rgb;
    let mut sett_rgb_c = *sett_rgb;
    let mix = c.mix;

    // Edge factor: fraction of the 3x3 neighbourhood carrying pigment. A full
    // block (9/9) reads e=1 (no boost); a rim cell reads e<1.
    let mut e = 1.0;
    if film0 > 0.0 && sett_c < 1000.0 {
        e = if count == 9 { 1.0 } else { count as f64 / 9.0 };
    }

    // Evaporate: retention leak + edge-boosted linear loss.
    let mut new_film =
        f * c.retention - c.evap_base * ((1.0 - e) * c.edge_darkening + c.base_evaporation);
    if new_film < 0.0001 {
        new_film = 0.0;
    }
    let lost = if f > 0.0 {
        1.0 - clamp01(new_film / f)
    } else {
        1.0
    };
    let film_c: f32 = new_film as f32;

    // Settle: the fraction of water lost carries the same fraction of
    // suspended pigment onto the paper (opacity-composited color).
    if lost > 0.0 && s_mass > 0.0 {
        let mut dm = s_mass * lost;
        if !c.ext_bypass {
            // Physical granulation (extension): settle biased toward valleys.
            let mut bias = 1.0 + c.ext_granulation * 0.6 * (0.5 - paper as f64) * 2.0;
            if bias < 0.3 {
                bias = 0.3;
            } else if bias > 1.7 {
                bias = 1.7;
            }
            dm *= bias;
            if dm > s_mass {
                dm = s_mass;
            }
        }
        // SPEC §6.2 step 3 (the shared settle composite, on locals).
        let a_sett = alpha_of_mass(sett_c as f64);
        let a_in = alpha_of_mass(dm);
        if a_sett > 0.0 {
            let u = a_sett * (1.0 - a_in);
            let w = a_in / (u + a_in);
            mix.mix(
                sett_rgb_c[0] as f64,
                sett_rgb_c[1] as f64,
                sett_rgb_c[2] as f64,
                susp_rgb_c[0] as f64,
                susp_rgb_c[1] as f64,
                susp_rgb_c[2] as f64,
                w,
                out,
            );
            sett_rgb_c = [out[0] as f32, out[1] as f32, out[2] as f32];
        } else {
            sett_rgb_c = susp_rgb_c;
        }
        sett_c = (sett_c as f64 + dm) as f32;
        let rem = s_mass - dm;
        susp_c = if rem < 0.0 { 0.0 } else { rem as f32 };
    }

    // Re-wet: standing water lifts a little settled pigment back into
    // suspension; the lift grows with the EXCESS water over what the
    // suspended pigment already occupies.
    let st = sett_c as f64;
    if film_c > 0.0 && st > 0.0 {
        let excess = (film_c as f64 - alpha_of_mass(susp_c as f64)).max(0.0);
        let mut b = c.rewet_base * (1.0 + excess * 50.0);
        if !c.ext_bypass {
            b *= staining_multiplier(c.ext_staining);
        }
        b = clamp01(b);
        if b > 0.0 {
            lift_settled(
                b,
                &mut susp_c,
                &mut sett_c,
                &mut susp_rgb_c,
                sett_rgb_c,
                mix,
                out,
            );
        }
    }

    // Write the cell back once.
    *film = film_c;
    *susp = susp_c;
    *sett = sett_c;
    *susp_rgb = susp_rgb_c;
    *sett_rgb = sett_rgb_c;
}

/// One drying pass over every bbox cell holding water or suspended pigment
/// (deliberately NOT filtered by the active mask — paint dries everywhere).
/// `evap_base` is the cadence-adaptive scale (or 1000 for a force-dry pass);
/// `rewet_base` the cadence-adaptive lift floor.
///
/// ⚠️ **Esta é a rota GAUSS-SEIDEL, congelada:** ela lê o `susp` que ela mesma
/// escreve (a janela deslizante enxerga a coluna à esquerda JÁ reescrita). É o
/// caminho de referência do ADR-0134; quem o produto roda é o
/// [`drying_pass_jacobi`].
pub fn drying_pass(g: &mut Grid, p: &Params, evap_base: f64, rewet_base: f64, ext_bypass: bool) {
    let s = g.s;
    let mut out = [0.0f64; 3];
    let c = DryConst::new(p, evap_base, rewet_base, ext_bypass);
    for y in g.by0..=g.by1 {
        // Faixa viva: ela é publicada a partir de `film > 0 || susp > 0` — o
        // MESMO predicado que gateia este laço —, então fora dela o corpo já
        // era um `continue`.
        let (bx0, bx1) = g.span_x(y);
        if bx0 > bx1 {
            continue;
        }
        let mut i = bx0 as usize + y as usize * s;
        // A vizinhança 3×3 do fator de borda, deslizante: só a coluna da
        // direita é nova a cada passo (`edge_window.rs`). Ela anda em TODA
        // célula — inclusive nas puladas, que são vizinhas das próximas.
        let mut win = EdgeWindow::seed(&g.susp, i, s);
        for x in bx0..=bx1 {
            // ⚠️ O avanço é no TOPO e carrega a coluna `i + 1`, nunca `i + 2`:
            // o pad da grade é de UMA coluna (`s = w + 2`), então `bx1 + 2`
            // cairia na linha seguinte.
            if x > bx0 {
                win.advance(&g.susp, i + 1, s);
            }
            if g.film[i] <= 0.0 && g.susp[i] <= 0.0 {
                i += 1;
                continue;
            }
            let count = win.count();
            let mut film = g.film[i];
            let mut susp = g.susp[i];
            let mut sett = g.sett[i];
            let mut srgb = g.susp_rgb[i];
            let mut trgb = g.sett_rgb[i];
            dry_cell(
                &c, g.paper[i], count, &mut film, &mut susp, &mut sett, &mut srgb, &mut trgb,
                &mut out,
            );
            g.film[i] = film;
            g.susp[i] = susp;
            g.sett[i] = sett;
            g.susp_rgb[i] = srgb;
            g.sett_rgb[i] = trgb;
            // A célula seguinte lê este `susp` JÁ ESCRITO (Gauss-Seidel).
            win.note_write(susp);
            i += 1;
        }
    }
}

/// Fast dry (SPEC §12): repeatedly halve the water and run force-dry passes
/// until no fluid remains (guard <= 40 loops). No advection — it cannot
/// stall; edge darkening still forms because the settle step runs each loop.
pub fn fast_dry(g: &mut Grid, p: &Params, rewet_base: f64, ext_bypass: bool) {
    let s = g.s;
    let mut guard = 0;
    while guard < 40 && g.has_fluid {
        for y in g.by0..=g.by1 {
            // Faixa viva: fora dela `film` é 0, e `0.5 * 0` é 0 — a escrita
            // que pulamos escrevia zero sobre zero.
            let (bx0, bx1) = g.span_x(y);
            if bx0 > bx1 {
                continue;
            }
            let mut i = bx0 as usize + y as usize * s;
            for _x in bx0..=bx1 {
                g.film[i] = (g.film[i] as f64 * 0.5) as f32;
                i += 1;
            }
        }
        drying_pass(g, p, 1000.0, rewet_base, ext_bypass);
        rebuild_active_region(g);
        guard += 1;
    }
    if !g.has_fluid {
        g.empty_bbox();
    }
}

// ---------------------------------------------------------------------------
// A SECAGEM INDEPENDENTE DE ORDEM (doc 28 §5.45)
// ---------------------------------------------------------------------------

/// **O fator de borda, materializado ANTES do passe** — gather puro.
///
/// A janela deslizante é a MESMA do laço Gauss-Seidel ([`EdgeWindow`]); o que
/// muda é que aqui ninguém escreve `susp` enquanto ela anda, então
/// [`EdgeWindow::note_write`] nunca é chamado e o resultado é função só do
/// estado de entrada.
fn edge_rows(g: &mut Grid, mode: crate::par::Rows) {
    let s = g.s;
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        scratch,
        susp,
        row_lo,
        row_hi,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let susp: &[f32] = susp;
    let band = by0 as usize * s..(by1 as usize + 1) * s;
    crate::par::walk_rows(mode, &mut scratch.edge[band], s, |ri, row| {
        let y = by0 + ri as i32;
        let (lo, hi) = crate::grid::span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y);
        if lo > hi {
            return;
        }
        let mut i = lo as usize + y as usize * s;
        let mut win = EdgeWindow::seed(susp, i, s);
        for x in lo..=hi {
            if x > lo {
                win.advance(susp, i + 1, s);
            }
            row[x as usize] = win.count() as u8;
            i += 1;
        }
    });
}

/// **A secagem que o produto roda** — a mesma física, sem depender de em que
/// ordem o laço anda.
///
/// ⚠️ **Isto é um SEGUNDO MODELO, como o [`crate::solver::advect_jacobi`]**, e
/// pelo mesmo motivo: o fator de borda lê a vizinhança 3×3 de `susp` que o
/// passe reescreve, então o Gauss-Seidel conta vizinhos **já secos** à esquerda
/// e vizinhos **ainda molhados** à direita. Isso não é física — é a direção da
/// varredura, e ela tem assinatura (`tests/drying_symmetry.rs`).
pub fn drying_pass_jacobi(
    g: &mut Grid,
    p: &Params,
    evap_base: f64,
    rewet_base: f64,
    ext_bypass: bool,
) {
    let rows = (g.by1 - g.by0 + 1).max(0) as usize;
    let span = (g.bx1 - g.bx0 + 1).max(0) as usize;
    let mode = crate::par::Rows::pick(rows, span, crate::par::MIN_CELLS_DRYING);
    drying_pass_jacobi_rows(g, p, evap_base, rewet_base, ext_bypass, mode);
}

/// [`drying_pass_jacobi`] com a rota forçada — a porta dos gates de identidade.
pub fn drying_pass_jacobi_rows(
    g: &mut Grid,
    p: &Params,
    evap_base: f64,
    rewet_base: f64,
    ext_bypass: bool,
    mode: crate::par::Rows,
) {
    if g.by0 > g.by1 {
        return;
    }
    g.scratch.ensure(g.cells);
    edge_rows(g, mode);

    let s = g.s;
    let c = DryConst::new(p, evap_base, rewet_base, ext_bypass);
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        scratch,
        film,
        susp,
        sett,
        susp_rgb,
        sett_rgb,
        paper,
        row_lo,
        row_hi,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let edge: &[u8] = &scratch.edge;
    let paper: &[f32] = paper;
    let band = by0 as usize * s..(by1 as usize + 1) * s;
    crate::par::walk_rows5(
        mode,
        &mut film[band.clone()],
        &mut susp[band.clone()],
        &mut sett[band.clone()],
        &mut susp_rgb[band.clone()],
        &mut sett_rgb[band],
        s,
        |ri, fr, sr, tr, cr, dr| {
            let y = by0 + ri as i32;
            let (lo, hi) = crate::grid::span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y);
            if lo > hi {
                return;
            }
            let mut out = [0.0f64; 3];
            let base = y as usize * s;
            for x in lo as usize..=hi as usize {
                if fr[x] <= 0.0 && sr[x] <= 0.0 {
                    continue;
                }
                let i = base + x;
                let (mut a, mut b, mut d) = (fr[x], sr[x], tr[x]);
                let (mut e, mut f) = (cr[x], dr[x]);
                dry_cell(
                    &c,
                    paper[i],
                    u32::from(edge[i]),
                    &mut a,
                    &mut b,
                    &mut d,
                    &mut e,
                    &mut f,
                    &mut out,
                );
                fr[x] = a;
                sr[x] = b;
                tr[x] = d;
                cr[x] = e;
                dr[x] = f;
            }
        },
    );
}
