//! **A água parada DORME** (B13.3, decisão 1 do dono). Por tile de 64×64 (as colunas da
//! [`super::faixa_tiles`], as linhas de 64 em 64), conta-se os passos seguidos com `|vel| ≤ ε`; um tile
//! cuja ilha 3×3 está toda quieta há [`PASSOS_PARA_DORMIR`] passos sai da máscara nos estágios do
//! fluxo (2..5). A secagem (estágio 1) e o rebuild (0) veem a máscara inteira: dormindo, só seca.
//!
//! Acorda (contador a zero): água nova pela [`Grid::note_tiles`] (pincel), restauro de histórico
//! ([`Grid::fill_tiles_all`]), troca de gravidade ou de knob ([`inicio`]). Sonda e medição: BACKLOG B13.3.

use super::Grid;
use super::faixa_tiles::TILE_FAIXA;

/// Passos seguidos quieto antes de dormir.
pub const PASSOS_PARA_DORMIR: u8 = 20;

/// O estado do sono de uma grade.
#[derive(Clone, Debug, Default)]
pub struct Sono {
    /// DESLIGADO por omissão: com ε = 0 o salto NÃO é ao bit na tela do produto (sonda
    /// `sonda_sono_ligado_e_desligado`: 365/400 passos diferem a 4K) — BACKLOG B13.3.
    pub ligado: bool,
    /// `|vel|` máximo que ainda conta como quieto.
    pub eps: f32,
    /// Passos seguidos quieto, por tile (`ntx × nty`).
    pub quietos: Vec<u8>,
    pub ntx: usize,
    pub nty: usize,
    /// A máscara de tiles antes de o sono a recortar (reposta no fim do passo).
    cheia: Vec<u64>,
    aplicado: bool,
    chave: u64,
    /// Tiles-linha saltados desde o início (medida).
    pub saltados: u64,
}

impl Sono {
    #[must_use]
    pub fn novo() -> Self {
        Sono::default()
    }
}

/// A chave do que, ao mudar, acorda toda a grade: os knobs e a gravidade do passo.
#[must_use]
pub fn chave(vals: &[f64], grav: [f64; 2]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for v in vals.iter().chain(&grav) {
        h = (h ^ v.to_bits()).wrapping_mul(0x0100_0000_01b3);
    }
    h
}

fn dimensiona(g: &mut Grid) {
    let (ntx, nty) = (
        g.s.div_ceil(TILE_FAIXA as usize),
        g.rows.div_ceil(TILE_FAIXA as usize),
    );
    if g.sono.ntx != ntx || g.sono.nty != nty {
        g.sono.ntx = ntx;
        g.sono.nty = nty;
        g.sono.quietos = vec![0; ntx * nty];
    }
}

fn ativo(g: &Grid) -> bool {
    g.sono.ligado && g.tiles_enabled && g.spans_enabled && !g.row_tiles.is_empty()
}

/// Começo do passo: knob ou gravidade mudou ⇒ acorda tudo.
pub(crate) fn inicio(g: &mut Grid, k: u64) {
    if !ativo(g) {
        return;
    }
    dimensiona(g);
    if g.sono.chave != k {
        g.sono.chave = k;
        g.sono.quietos.fill(0);
    }
}

/// Antes do estágio 2: tira da máscara os tiles cuja ilha 3×3 dorme.
pub(crate) fn adormece(g: &mut Grid) {
    if !ativo(g) || g.sono.quietos.is_empty() {
        return;
    }
    let (ntx, nty, tw) = (g.sono.ntx, g.sono.nty, g.tile_words);
    let dorme = |tx: usize, ty: usize, q: &[u8]| {
        let (x0, x1) = (tx.saturating_sub(1), (tx + 1).min(ntx - 1));
        let (y0, y1) = (ty.saturating_sub(1), (ty + 1).min(nty - 1));
        (y0..=y1).all(|y| (x0..=x1).all(|x| q[y * ntx + x] >= PASSOS_PARA_DORMIR))
    };
    let mut corte = vec![0u64; tw];
    let mut algum = false;
    let cheia = std::mem::take(&mut g.sono.cheia);
    let mut cheia = cheia;
    cheia.clear();
    cheia.extend_from_slice(&g.row_tiles);
    for ty in 0..nty {
        corte.fill(0);
        let mut n = 0u64;
        for tx in 0..ntx {
            if dorme(tx, ty, &g.sono.quietos) {
                corte[tx >> 6] |= 1 << (tx & 63);
                n += 1;
            }
        }
        if n == 0 {
            continue;
        }
        algum = true;
        let y1 = ((ty + 1) * TILE_FAIXA as usize).min(g.rows);
        for y in ty * TILE_FAIXA as usize..y1 {
            for (w, c) in g.row_tiles[y * tw..(y + 1) * tw].iter_mut().zip(&corte) {
                g.sono.saltados += u64::from((*w & c).count_ones());
                *w &= !c;
            }
        }
    }
    g.sono.cheia = cheia;
    g.sono.aplicado = algum;
}

/// Fim do passo: repõe a máscara e conta quem ficou quieto (só a velocidade: a secagem muda o filme).
pub(crate) fn conta(g: &mut Grid) {
    if g.sono.aplicado {
        for (w, c) in g.row_tiles.iter_mut().zip(&g.sono.cheia) {
            *w |= c;
        }
        g.sono.aplicado = false;
    }
    if !ativo(g) || g.sono.quietos.is_empty() {
        return;
    }
    let (ntx, eps) = (g.sono.ntx, g.sono.eps);
    let mut mexe = vec![false; g.sono.quietos.len()];
    let fg = g.flow;
    let rf = fg.rf as i32;
    for fy in 1..=fg.h {
        let (lo, hi) = (g.frow_lo[fy], g.frow_hi[fy]);
        if lo > hi {
            continue;
        }
        let ty0 = (((fy as i32 - 1) * rf).max(0) as usize) / TILE_FAIXA as usize;
        let ty1 = ((((fy as i32 + 1) * rf) as usize).min(g.rows - 1)) / TILE_FAIXA as usize;
        let lo = lo.max(1) as usize;
        let hi = (hi.max(0) as usize).min(fg.w);
        for fx in lo..=hi {
            let i = fy * fg.s + fx;
            if g.vel_x[i].abs() <= eps && g.vel_y[i].abs() <= eps {
                continue;
            }
            let fx = fx as i32;
            let tx0 = (((fx - 1) * rf).max(0) as usize) / TILE_FAIXA as usize;
            let tx1 = ((((fx + 1) * rf) as usize).min(g.s - 1)) / TILE_FAIXA as usize;
            for ty in ty0..=ty1 {
                for tx in tx0..=tx1.min(ntx - 1) {
                    mexe[ty * ntx + tx] = true;
                }
            }
        }
    }
    for (q, m) in g.sono.quietos.iter_mut().zip(mexe) {
        *q = if m { 0 } else { q.saturating_add(1) };
    }
}

/// Acorda os tiles de `x0..=x1` × `y0..=y1` (água nova).
pub(crate) fn acorda(g: &mut Grid, x0: i32, y0: usize, x1: i32, y1: usize) {
    if g.sono.quietos.is_empty() {
        return;
    }
    let ntx = g.sono.ntx;
    let t = TILE_FAIXA as usize;
    let tx0 = ((x0 - TILE_FAIXA).max(0) as usize) / t;
    let tx1 = (((x1 + TILE_FAIXA).max(0) as usize) / t).min(ntx - 1);
    let ty1 = (y1 / t).min(g.sono.nty - 1);
    for ty in y0 / t..=ty1 {
        g.sono.quietos[ty * ntx + tx0..=ty * ntx + tx1].fill(0);
    }
}

#[cfg(test)]
#[path = "sono_tests.rs"]
mod tests;
