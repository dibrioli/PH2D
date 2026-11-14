//! O TEMPO pela lei única da tinta (F4-2 E8, ADR-0197; `docs/Painter/Arquitetura/E8_tempo.md`): com
//! [`crate::sim::LeiDaTinta::Pigmento`] assentar move a fracção da SOMA que a massa move (sem a inversa de
//! Newton: o ecrã junta filme e aberto, logo não muda), re-molhar devolve-a pela abertura `o`, e o aberto
//! cura `o ← o·e^{−Δt/τ(m)}` até FECHAR sobre `r_g`. Invariante: `filme.v·M_UM == susp`, `aberto.v·M_UM == sett`.

use super::edge_window::EdgePlane;
use super::{DryConst, dry_cell};
use crate::grid::Grid;
use crate::grid::faixa_tiles::{span_runs, tiles_row_of};
use crate::grid::tinta::CelulaTinta;
use crate::grid::tinta::pigmento::optica_celula;
use crate::grid::tinta::pigmento::{ABERTO, FILME, LadoEspectral, srgb_0_255};
use ph2d_det_math::DetMath as _;
use ph2d_paint_tinta::{Espectro, Soma, transfere};
use rayon::prelude::*;

/// A cura do óleo puro (`m = 1`), em segundos do relógio da simulação (F2 §2: «τ(1) ≈ horas», escala artística).
pub const TAU_OLEO_S: f32 = 3600.0;

/// Abaixo desta abertura o aberto FECHA (`e^{−4,6}`: o pick-up `k_u·o` já não leva 1 %).
pub const O_FECHA: f32 = 0.01;

/// De quantos passos (40 Hz) em quantos o relógio da cura anda: a cura varre a grade INTEIRA (a tinta seca fora
/// da caixa molhada também cura) e `e^{−a}·e^{−b} = e^{−(a+b)}` deixa juntar o tempo.
pub const PASSOS_DA_CURA: u64 = 40;

/// `τ(m)`: só o veículo que OXIDA (a fracção `m` de óleo) cura, à taxa `m/τ_óleo`. A goma da aguarela re-dissolve
/// sempre ⇒ `τ(0) = ∞` (a aguarela não fecha; levantá-la seca é a lei dela).
#[must_use]
pub fn tau(m: f32) -> f32 {
    if m > 0.0 {
        TAU_OLEO_S / m
    } else {
        f32::INFINITY
    }
}

/// A abertura do aberto que recebe `chega` de volume fresco (`o = 1`): média pelo volume (`o = 1` fica `1` ao bit).
#[inline]
fn abre_pelo_volume(c: &mut CelulaTinta, antes: f32, chega: f32) {
    let v = antes + chega;
    if v > 0.0 {
        c.o += (1.0 - c.o) * (chega / v);
    }
}

/// A célula da lei da tinta que a secagem move: as somas e, com o Pigment mix, o lado espectral (E10c).
pub(crate) type Celula<'a> = (&'a mut CelulaTinta, Option<&'a mut LadoEspectral>);

/// Assentar: a fracção `f` do filme passa ao aberto (a mesma que `susp → sett`); o espelho `sett_rgb` segue.
#[inline]
pub(crate) fn assenta(
    c: &mut CelulaTinta,
    mut l: Option<&mut LadoEspectral>,
    f: f64,
    sett_rgb: &mut [f32; 3],
) {
    let antes = c.aberto.v;
    let movido = transfere(&mut c.filme, &mut c.aberto, f as f32);
    abre_pelo_volume(c, antes, movido.v);
    if let Some([a, b]) = l.as_deref_mut() {
        let _ = Espectro::transfere(a, b, f as f32);
    }
    if let Some(rgb) = srgb_0_255(&c.aberto, l.map(|l| &l[ABERTO])) {
        *sett_rgb = rgb;
    }
}

/// Re-molhar (a porta de [`super::lift_settled`] na lei nova): a fracção `b·o` do aberto volta ao filme, na MASSA
/// com a aritmética de `lift_settled` e nas somas por [`transfere`]. Com `o = 1` a massa é a da lei velha ao bit.
#[inline]
pub(crate) fn levanta(
    c: &mut CelulaTinta,
    mut l: Option<&mut LadoEspectral>,
    b: f64,
    susp: &mut f32,
    sett: &mut f32,
    susp_rgb: &mut [f32; 3],
) {
    let b = b * f64::from(c.o);
    let st = f64::from(*sett);
    let lift = st * b;
    *susp = (f64::from(*susp) + lift) as f32;
    *sett = (st - lift) as f32;
    transfere(&mut c.aberto, &mut c.filme, b as f32);
    if let Some([fi, ab]) = l.as_deref_mut() {
        let _ = Espectro::transfere(ab, fi, b as f32);
    }
    if let Some(rgb) = srgb_0_255(&c.filme, l.map(|l| &l[FILME])) {
        *susp_rgb = rgb;
    }
}

/// A secagem independente de ordem com as somas: o MESMO [`dry_cell`] de [`super::dry_jacobi`], com a 6.ª faixa
/// (`Grid::tinta`) — o `walk_rows5` não a leva.
pub(super) fn dry_jacobi<P: EdgePlane>(
    g: &mut Grid,
    c: &DryConst,
    esc: Option<&super::por_tile::Escolha<'_>>,
    mode: crate::par::Rows,
    soft: &[f64],
) {
    let s = g.s;
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        scratch,
        film,
        susp,
        sett,
        susp_rgb,
        sett_rgb,
        tinta,
        pigmento,
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
    let (paper, vela): (&[f32], &[f32]) = (paper, vela);
    let aparencia: &crate::grid::aparencia::CampoAparencia = aparencia;
    let oleo: &crate::grid::oleo_celula::CampoOleo = oleo;
    let (row_lo, row_hi): (&[i32], &[i32]) = (row_lo, row_hi);
    let band = by0 as usize * s..(by1 as usize + 1) * s;
    let corpo = |ri: usize,
                 fr: &mut [f32],
                 sr: &mut [f32],
                 tr: &mut [f32],
                 cr: &mut [[f32; 3]],
                 dr: &mut [[f32; 3]],
                 kr: &mut [CelulaTinta],
                 mut lr: Option<&mut [LadoEspectral]>| {
        let y = by0 + ri as i32;
        let (lo, hi) = crate::grid::span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y);
        let mut out = [0.0f64; 3];
        let base = y as usize * s;
        let runs = span_runs(tiles_row_of(row_tiles, tw, tiles_on, y), lo, hi);
        for x in runs.flat_map(|(a, b)| a as usize..=b as usize) {
            if fr[x] <= 0.0 && sr[x] <= 0.0 {
                continue;
            }
            let Some(c) = super::por_tile::escolhe(c, esc, x, y as usize) else {
                continue;
            };
            let i = base + x;
            dry_cell(
                c,
                paper[i],
                P::nb(edge, soft, i),
                vela[i],
                crate::grid::aparencia::vidrado(aparencia, i),
                crate::grid::aparencia::opaca(aparencia, i),
                crate::grid::aparencia::borda(aparencia, i),
                crate::grid::oleo_celula::da_celula(oleo, i),
                &mut fr[x],
                &mut sr[x],
                &mut tr[x],
                &mut cr[x],
                &mut dr[x],
                &mut out,
                Some((&mut kr[x], lr.as_deref_mut().map(|l| &mut l[x]))),
            );
        }
    };
    let linhas: Vec<_> = film[band.clone()]
        .chunks_mut(s)
        .zip(susp[band.clone()].chunks_mut(s))
        .zip(sett[band.clone()].chunks_mut(s))
        .zip(susp_rgb[band.clone()].chunks_mut(s))
        .zip(sett_rgb[band.clone()].chunks_mut(s))
        .zip(tinta[band.clone()].chunks_mut(s))
        .zip(lados_por_linha(pigmento, band, s))
        .collect();
    let serial = matches!(mode, crate::par::Rows::Serial);
    let roda = |(ri, ((((((fr, sr), tr), cr), dr), kr), lr)): (usize, _)| {
        corpo(ri, fr, sr, tr, cr, dr, kr, lr);
    };
    if serial {
        linhas.into_iter().enumerate().for_each(roda);
    } else {
        linhas.into_par_iter().enumerate().for_each(roda);
    }
}

/// A CURA de `dt_s` segundos em toda a grade: `o ← o·e^{−dt/τ(m)}` onde há aberto; `o < O_FECHA` com a célula SECA
/// (sem água e sem filme — o fecho é então invisível ao bit, `transfere.rs fecha`) ⇒ o aberto vira `r_g`, o volume
/// dele vai para `altura_fechada` e a massa `sett` sai da grade (o Óleo Real soma-a de volta pela
/// `render::tinta::massa_da_altura`).
pub fn cura(g: &mut Grid, dt_s: f32) {
    if g.tinta.is_empty() {
        return;
    }
    if !g.pigmento.is_empty() {
        return cura_com_pigmento(g, dt_s);
    }
    let s = g.s;
    let Grid {
        tinta, sett, film, ..
    } = g;
    let film: &[f32] = film;
    let rows = tinta.len() / s;
    let mode = crate::par::Rows::pick_medido(rows, s, crate::par::MIN_CELLS_DRYING);
    crate::par::walk_rows2(mode, tinta, sett, s, |ri, kr, tr| {
        for (x, c) in kr.iter_mut().enumerate() {
            let m = c.aberto.meio();
            if c.aberto.v <= 0.0 || m <= 0.0 {
                continue;
            }
            c.o *= (-dt_s / tau(m)).det_exp();
            let i = ri * s + x;
            if c.o < O_FECHA && film[i] <= 0.0 && c.filme.v <= 0.0 {
                fecha_celula(c, None, &mut tr[x]);
            }
        }
    });
}

/// O fecho de UMA célula (sem as condições de [`cura`]): o aberto passa a fundo e a massa dele sai de `sett`.
pub(crate) fn fecha_celula(c: &mut CelulaTinta, l: Option<&mut LadoEspectral>, sett: &mut f32) {
    let m = c.aberto.meio();
    c.altura_fechada += c.aberto.v;
    // A óptica do aberto SECO (a de `ph2d_paint_tinta::fecha`, com a cor do Pigment mix quando há lado); o fundo
    // sobre o preto fecha com a MESMA óptica (a cobertura do fechado = a de antes, ao bit).
    let so_aberto = l.as_deref().map(|l| [Espectro::default(), l[ABERTO]]);
    let o = optica_celula(&Soma::default(), &c.aberto, so_aberto.as_ref(), m, false);
    // A camada fina prende o matiz (E10d1, `docs/Painter/Arquitetura/E10d1_camada_fina.md`).
    (c.r_g, c.r_g0) = crate::render::tinta::matiz::pilha_presa(&o, c, l.as_deref());
    c.aberto = Soma::default();
    if let Some(l) = l {
        l[ABERTO] = Espectro::default();
    }
    c.o = 1.0;
    *sett = 0.0;
}

/// As linhas do lado espectral da faixa (`None` por linha sem o campo).
fn lados_por_linha(
    l: &mut [LadoEspectral],
    band: core::ops::Range<usize>,
    s: usize,
) -> Vec<Option<&mut [LadoEspectral]>> {
    if l.is_empty() {
        (0..band.len() / s).map(|_| None).collect()
    } else {
        l[band].chunks_mut(s).map(Some).collect()
    }
}

/// A [`cura`] com o lado espectral (série: o fecho zera o `Espectro` do aberto com a soma).
fn cura_com_pigmento(g: &mut Grid, dt_s: f32) {
    let Grid {
        tinta,
        pigmento,
        sett,
        film,
        ..
    } = g;
    for (i, (c, l)) in tinta.iter_mut().zip(pigmento.iter_mut()).enumerate() {
        let m = c.aberto.meio();
        if c.aberto.v <= 0.0 || m <= 0.0 {
            continue;
        }
        c.o *= (-dt_s / tau(m)).det_exp();
        if c.o < O_FECHA && film[i] <= 0.0 && c.filme.v <= 0.0 {
            fecha_celula(c, Some(l), &mut sett[i]);
        }
    }
}

#[cfg(test)]
#[path = "tinta_tests.rs"]
mod tests;
