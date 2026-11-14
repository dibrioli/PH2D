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
use crate::grid::faixa_tiles::{span_runs, tiles_row_of};
use crate::jsmath::clamp01;
use crate::opacity::alpha_of_mass;
use crate::sim::Params;
use crate::solver::rebuild_active_region;
use crate::tuning::Knob;

#[path = "drying/edge_window.rs"]
mod edge_window;
#[cfg(test)]
mod evaporacao_base_tests;
use edge_window::EdgeWindow;
#[path = "drying/por_tile.rs"]
pub mod por_tile;
#[path = "drying/tinta.rs"]
pub mod tinta;
pub use por_tile::SecaPorTile;
pub use tinta::{PASSOS_DA_CURA, cura};

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
    glaze: f64,
    ecra: crate::assenta::Ecra,
    out: &mut [f64; 3],
) {
    let st = *sett as f64;
    let lift = st * b;
    if ecra != crate::assenta::Ecra::Porto && lift > 0.0 {
        let sf = *susp as f64;
        // Arrays escritos à mão, não `.map`: a régua da rodada 5 mediu ~12 ms por passe de secagem a 4K nas cópias.
        let r = crate::assenta::levanta(
            ecra,
            [sett_rgb[0] as f64, sett_rgb[1] as f64, sett_rgb[2] as f64],
            [susp_rgb[0] as f64, susp_rgb[1] as f64, susp_rgb[2] as f64],
            alpha_of_mass(st),
            alpha_of_mass(sf),
            alpha_of_mass(st - lift),
            alpha_of_mass(sf + lift),
        );
        *susp_rgb = [r[0] as f32, r[1] as f32, r[2] as f32];
        *susp = (sf + lift) as f32;
        *sett = (st - lift) as f32;
        return;
    }
    let a_in = alpha_of_mass(st) * b;
    let a_susp = alpha_of_mass(*susp as f64);
    let u = a_susp * (1.0 - a_in);
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
        crate::colorops::velatura(
            out,
            w,
            glaze,
            a_susp,
            susp_rgb.map(f64::from),
            sett_rgb.map(f64::from),
        );
        *susp_rgb = [out[0] as f32, out[1] as f32, out[2] as f32];
    } else {
        *susp_rgb = sett_rgb;
    }
    *susp = (*susp as f64 + lift) as f32;
    *sett = (st - lift) as f32;
}

/// A evaporação de base da lei (B35.10): o knob Base evaporation fundiu-se no Dry Speed (`Evaporation`), que já
/// escala o termo inteiro; fica a constante de fábrica, `2`.
pub const EVAPORACAO_BASE: f64 = 2.0;

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
    pub glaze: f64,
    pub ecra: crate::assenta::Ecra,
}

impl DryConst {
    pub fn new(g: &Grid, p: &Params, evap_base: f64, rewet_base: f64, ext_bypass: bool) -> Self {
        DryConst {
            retention: p.retencao(),
            edge_darkening: if g.borda_fundo < 0.0 {
                p.k(Knob::EdgeDarkening)
            } else {
                g.borda_fundo
            },
            base_evaporation: EVAPORACAO_BASE,
            ext_granulation: p.k(Knob::ExtGranulation),
            ext_staining: p.k(Knob::ExtStaining),
            evap_base,
            rewet_base,
            ext_bypass,
            mix: p.mix,
            glaze: p.glaze,
            ecra: p.ecra,
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
    nb: f64,
    vela: f32,
    vidrado: f32,
    opaca: f32,
    borda: f32,
    oleo: crate::grid::oleo_celula::OleoCelula,
    film: &mut f32,
    susp: &mut f32,
    sett: &mut f32,
    susp_rgb: &mut [f32; 3],
    sett_rgb: &mut [f32; 3],
    out: &mut [f64; 3],
    mut tinta: Option<tinta::Celula<'_>>,
) {
    let film0 = *film;
    let f = film0 as f64;
    let s_mass = *susp as f64;
    let mut susp_c = *susp;
    let mut sett_c = *sett;
    let mut susp_rgb_c = *susp_rgb;
    let mut sett_rgb_c = *sett_rgb;
    let mix = c.mix;
    let ecra = c
        .ecra
        .com_vela(vela)
        .com_vidrado(vidrado)
        .com_oleo(oleo)
        .com_opaca(opaca);

    // Edge factor: fraction of the 3x3 neighbourhood carrying pigment. A full
    // block (9/9) reads e=1 (no boost); a rim cell reads e<1.
    let mut e = 1.0;
    if film0 > 0.0 && sett_c < 1000.0 {
        e = nb;
    }

    // Evaporate: retention leak + edge-boosted linear loss — both slowed by the medium (`crate::meio`).
    let m = f64::from(oleo.meio);
    let borda = if borda < 0.0 {
        c.edge_darkening
    } else {
        f64::from(borda)
    };
    let mut new_film = f * crate::meio::retencao(c.retention, m)
        - c.evap_base * ((1.0 - e) * borda + c.base_evaporation) * crate::meio::evaporacao(m);
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
        if let Some((t, l)) = tinta.as_mut() {
            tinta::assenta(t, l.as_deref_mut(), dm / s_mass, &mut sett_rgb_c);
        } else {
            let a_sett = alpha_of_mass(sett_c as f64);
            let a_in = alpha_of_mass(dm);
            if ecra != crate::assenta::Ecra::Porto {
                sett_rgb_c = crate::assenta::assenta_memo_f32(
                    ecra,
                    sett_rgb_c,
                    susp_rgb_c,
                    a_sett,
                    alpha_of_mass(s_mass),
                    alpha_of_mass(sett_c as f64 + dm),
                    alpha_of_mass(s_mass - dm),
                );
            } else if a_sett > 0.0 {
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
                crate::colorops::velatura(
                    out,
                    w,
                    c.glaze,
                    a_sett,
                    sett_rgb_c.map(f64::from),
                    susp_rgb_c.map(f64::from),
                );
                sett_rgb_c = [out[0] as f32, out[1] as f32, out[2] as f32];
            } else {
                sett_rgb_c = susp_rgb_c;
            }
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
        if let (true, Some((t, l))) = (b > 0.0, tinta) {
            tinta::levanta(t, l, b, &mut susp_c, &mut sett_c, &mut susp_rgb_c);
        } else if b > 0.0 {
            lift_settled(
                b,
                &mut susp_c,
                &mut sett_c,
                &mut susp_rgb_c,
                sett_rgb_c,
                mix,
                c.glaze,
                ecra,
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
    drying_pass_por_tile(g, p, evap_base, rewet_base, ext_bypass, None);
}

/// [`drying_pass`] com a cadência por tile ([`SecaPorTile`], B13.4); `None` = tudo calmo, o de antes ao bit.
pub fn drying_pass_por_tile(
    g: &mut Grid,
    p: &Params,
    evap_base: f64,
    rewet_base: f64,
    ext_bypass: bool,
    t: Option<&SecaPorTile<'_>>,
) {
    match p.k(Knob::EdgeSoftMass) {
        k if k > 0.0 => {
            dry_gs::<edge_window::EdgeWindowSoft>(g, p, evap_base, rewet_base, ext_bypass, k, t)
        }
        _ => dry_gs::<EdgeWindow>(g, p, evap_base, rewet_base, ext_bypass, 0.0, t),
    }
}

fn dry_gs<W: edge_window::EdgeGather>(
    g: &mut Grid,
    p: &Params,
    evap_base: f64,
    rewet_base: f64,
    ext_bypass: bool,
    k: f64,
    t: Option<&SecaPorTile<'_>>,
) {
    let s = g.s;
    let mut out = [0.0f64; 3];
    let c = DryConst::new(g, p, evap_base, rewet_base, ext_bypass);
    let esc = por_tile::Escolha::new(t, g, p, ext_bypass);
    let pigmento = p.tinta == crate::sim::LeiDaTinta::Pigmento && !g.tinta.is_empty();
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
        let mut win = W::seed(&g.susp, i, s, edge_window::soft_den(k));
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
            let Some(c) = por_tile::escolhe(&c, esc.as_ref(), x as usize, y as usize) else {
                i += 1;
                continue;
            };
            let count = win.frac();
            let mut film = g.film[i];
            let mut susp = g.susp[i];
            let mut sett = g.sett[i];
            let mut srgb = g.susp_rgb[i];
            let mut trgb = g.sett_rgb[i];
            dry_cell(
                c,
                g.paper[i],
                count,
                g.vela[i],
                crate::grid::aparencia::vidrado(&g.aparencia, i),
                crate::grid::aparencia::opaca(&g.aparencia, i),
                crate::grid::aparencia::borda(&g.aparencia, i),
                crate::grid::oleo_celula::da_celula(&g.oleo, i),
                &mut film,
                &mut susp,
                &mut sett,
                &mut srgb,
                &mut trgb,
                &mut out,
                if pigmento {
                    Some((&mut g.tinta[i], g.pigmento.get_mut(i)))
                } else {
                    None
                },
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
        row_tiles,
        tile_words,
        tiles_enabled,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let (tw, tiles_on, row_tiles) = (*tile_words, spans_enabled && *tiles_enabled, &row_tiles[..]);
    let susp: &[f32] = susp;
    let band = by0 as usize * s..(by1 as usize + 1) * s;
    crate::par::walk_rows(mode, &mut scratch.edge[band], s, |ri, row| {
        let y = by0 + ri as i32;
        let (lo, hi) = crate::grid::span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y);
        for (lo, hi) in span_runs(tiles_row_of(row_tiles, tw, tiles_on, y), lo, hi) {
            let mut i = lo as usize + y as usize * s;
            let mut win = EdgeWindow::seed(susp, i, s);
            for x in lo..=hi {
                if x > lo {
                    win.advance(susp, i + 1, s);
                }
                row[x as usize] = win.count() as u8;
                i += 1;
            }
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
    let mode = crate::par::Rows::pick_medido(rows, span, crate::par::MIN_CELLS_DRYING);
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
    drying_pass_jacobi_por_tile(g, p, evap_base, rewet_base, ext_bypass, mode, None);
}

/// [`drying_pass_jacobi_rows`] com a cadência por tile ([`SecaPorTile`], B13.4); `None` = o de antes ao bit.
pub fn drying_pass_jacobi_por_tile(
    g: &mut Grid,
    p: &Params,
    evap_base: f64,
    rewet_base: f64,
    ext_bypass: bool,
    mode: crate::par::Rows,
    t: Option<&SecaPorTile<'_>>,
) {
    if g.by0 > g.by1 {
        return;
    }
    g.scratch.ensure(g.cells);
    let k = p.k(Knob::EdgeSoftMass);
    if k > 0.0 {
        return edge_window::soft_plane(g, mode, k, |g, soft| {
            dry_jacobi::<edge_window::SoftPlane>(
                g, p, evap_base, rewet_base, ext_bypass, mode, soft, t,
            )
        });
    }
    edge_rows(g, mode);
    dry_jacobi::<edge_window::HardPlane>(g, p, evap_base, rewet_base, ext_bypass, mode, &[], t);
}

fn dry_jacobi<P: edge_window::EdgePlane>(
    g: &mut Grid,
    p: &Params,
    evap_base: f64,
    rewet_base: f64,
    ext_bypass: bool,
    mode: crate::par::Rows,
    soft: &[f64],
    t: Option<&SecaPorTile<'_>>,
) {
    let s = g.s;
    let c = DryConst::new(g, p, evap_base, rewet_base, ext_bypass);
    let esc = por_tile::Escolha::new(t, g, p, ext_bypass);
    if p.tinta == crate::sim::LeiDaTinta::Pigmento && !g.tinta.is_empty() {
        return tinta::dry_jacobi::<P>(g, &c, esc.as_ref(), mode, soft);
    }
    let esc = esc.as_ref();
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        scratch,
        film,
        susp,
        sett,
        susp_rgb,
        sett_rgb,
        paper,
        vela,
        aparencia,
        oleo,
        row_lo,
        row_hi,
        row_tiles,
        tile_words,
        tiles_enabled,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let (tw, tiles_on, row_tiles) = (*tile_words, spans_enabled && *tiles_enabled, &row_tiles[..]);
    let edge: &[u8] = &scratch.edge;
    let paper: &[f32] = paper;
    let vela: &[f32] = vela;
    let aparencia: &crate::grid::aparencia::CampoAparencia = aparencia;
    let oleo: &crate::grid::oleo_celula::CampoOleo = oleo;
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
            let mut out = [0.0f64; 3];
            let base = y as usize * s;
            let runs = span_runs(tiles_row_of(row_tiles, tw, tiles_on, y), lo, hi);
            for x in runs.flat_map(|(a, b)| a as usize..=b as usize) {
                if fr[x] <= 0.0 && sr[x] <= 0.0 {
                    continue;
                }
                let Some(c) = por_tile::escolhe(&c, esc, x, y as usize) else {
                    continue;
                };
                let i = base + x;
                let (mut a, mut b, mut d) = (fr[x], sr[x], tr[x]);
                let (mut e, mut f) = (cr[x], dr[x]);
                dry_cell(
                    c,
                    paper[i],
                    P::nb(edge, soft, i),
                    vela[i],
                    crate::grid::aparencia::vidrado(aparencia, i),
                    crate::grid::aparencia::opaca(aparencia, i),
                    crate::grid::aparencia::borda(aparencia, i),
                    crate::grid::oleo_celula::da_celula(oleo, i),
                    &mut a,
                    &mut b,
                    &mut d,
                    &mut e,
                    &mut f,
                    &mut out,
                    None,
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
