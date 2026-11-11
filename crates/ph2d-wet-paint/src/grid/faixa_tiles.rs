//! **A faixa por TILES de 64 colunas** (B13.2) — a faixa viva é UM intervalo por linha, e duas manchas nas
//! mesmas linhas varrem o seco entre elas (sonda `measure_the_dry_gap_between_two_blobs_on_the_same_rows`:
//! 5,9 ms/quadro, 5,9× as células). Por linha, uma máscara de tiles diz onde, DENTRO do intervalo, há o que
//! varrer; [`span_runs`] parte o intervalo nos troços marcados.
//!
//! Invariante (o mesmo da faixa, no eixo x): fora dos tiles marcados `active == 0`, sem água, sem
//! pigmento em suspensão, sem velocidade. Publica-se do vivo com o pad de linhas da faixa e UM tile de
//! vizinho de cada lado — o alcance horizontal da faixa (pad + janela) é < 64 (`debug_assert`).
//! Quem não sabe (restauro de histórico) marca tudo: superconjunto, nunca sub.

use super::{Grid, SPAN_PAD};

/// Largura do tile da máscara, em células.
pub const TILE_FAIXA: i32 = 64;
const _: () = assert!(
    SPAN_PAD + 2 < TILE_FAIXA,
    "o alcance da faixa passa de um tile"
);

/// Palavras de máscara por linha numa grade de passo `s`.
#[must_use]
pub fn tile_words_for(s: usize) -> usize {
    s.div_ceil(TILE_FAIXA as usize).div_ceil(64).max(1)
}

#[inline]
fn bit(words: &[u64], t: i32) -> bool {
    let t = t as usize;
    words.get(t >> 6).is_some_and(|w| w >> (t & 63) & 1 == 1)
}

/// Os troços de `[lo, hi]` que caem em tiles marcados (`None` = o intervalo inteiro, a faixa de um troço).
pub struct Runs<'a> {
    words: Option<&'a [u64]>,
    lo: i32,
    hi: i32,
    t: i32,
}

impl Iterator for Runs<'_> {
    type Item = (i32, i32);
    fn next(&mut self) -> Option<(i32, i32)> {
        if self.lo > self.hi {
            return None;
        }
        let Some(words) = self.words else {
            let run = (self.lo, self.hi);
            self.lo = self.hi + 1;
            return Some(run);
        };
        let last = self.hi >> 6;
        while self.t <= last && !bit(words, self.t) {
            self.t += 1;
        }
        if self.t > last {
            self.lo = self.hi + 1;
            return None;
        }
        let t0 = self.t;
        while self.t <= last && bit(words, self.t) {
            self.t += 1;
        }
        let a = self.lo.max(t0 * TILE_FAIXA);
        let b = self.hi.min(self.t * TILE_FAIXA - 1);
        Some((a, b))
    }
}

/// Parte `[lo, hi]` pelos tiles da linha (`words`); `None` devolve o intervalo inteiro.
#[must_use]
pub fn span_runs(words: Option<&[u64]>, lo: i32, hi: i32) -> Runs<'_> {
    let lo = if words.is_some() { lo.max(0) } else { lo };
    Runs {
        words,
        lo,
        hi,
        t: lo.max(0) >> 6,
    }
}

/// A máscara da linha `y`, em forma livre (split-borrow); `None` com a faixa ou os tiles desligados.
#[inline]
#[must_use]
pub fn tiles_row_of(row_tiles: &[u64], tw: usize, on: bool, y: i32) -> Option<&[u64]> {
    if !on {
        return None;
    }
    let r = y as usize * tw;
    row_tiles.get(r..r + tw)
}

impl Grid {
    /// A máscara da linha `y` (`None` = varrer o intervalo inteiro).
    #[inline]
    #[must_use]
    pub fn tiles_row(&self, y: i32) -> Option<&[u64]> {
        tiles_row_of(
            &self.row_tiles,
            self.tile_words,
            self.spans_enabled && self.tiles_enabled,
            y,
        )
    }

    /// A máscara da linha de FLUXO `y` (projetada no [`Grid::refresh_flow_spans`]); `None` = o intervalo inteiro.
    #[inline]
    #[must_use]
    pub fn ftiles_row(&self, y: i32) -> Option<&[u64]> {
        tiles_row_of(
            &self.frow_tiles,
            tile_words_for(self.flow.s),
            self.spans_enabled && self.tiles_enabled,
            y,
        )
    }

    /// Marca todo tile (quem não sabe o que há na linha: o restauro de histórico).
    pub fn fill_tiles_all(&mut self) {
        self.row_tiles.fill(u64::MAX);
    }

    /// Declara `x0..=x1` vivo nas linhas `y0..=y1` (com o tile vizinho), como o [`Grid::note_live`].
    pub(crate) fn note_tiles(&mut self, x0: i32, y0: usize, x1: i32, y1: usize) {
        if self.row_tiles.is_empty() {
            return;
        }
        let tw = self.tile_words;
        let t0 = ((x0 - TILE_FAIXA).max(0)) >> 6;
        let t1 = (x1 + TILE_FAIXA) >> 6;
        for r in y0..=y1 {
            let row = &mut self.row_tiles[r * tw..(r + 1) * tw];
            for t in t0..=t1 {
                let t = t as usize;
                if let Some(w) = row.get_mut(t >> 6) {
                    *w |= 1 << (t & 63);
                }
            }
        }
    }

    /// Publica a máscara a partir do rascunho `live_tiles` (cabeçalho + palavras por linha): OR das linhas
    /// `y ± SPAN_PAD` e um tile de vizinho de cada lado.
    pub(crate) fn publish_tiles_from_live(&mut self) {
        if self.row_tiles.is_empty() || self.live_tiles.is_empty() {
            return;
        }
        let tw = self.tile_words;
        let st = tw + 1;
        let rows = self.rows as i32;
        let mut acc = vec![0u64; tw];
        for y in 0..rows {
            acc.fill(0);
            for r in (y - SPAN_PAD).max(0)..=(y + SPAN_PAD).min(rows - 1) {
                let src = &self.live_tiles[r as usize * st + 1..(r as usize + 1) * st];
                for (a, s) in acc.iter_mut().zip(src) {
                    *a |= s;
                }
            }
            let out = &mut self.row_tiles[y as usize * tw..(y as usize + 1) * tw];
            for k in 0..tw {
                let lo_carry = if k > 0 { acc[k - 1] >> 63 } else { 0 };
                let hi_carry = if k + 1 < tw { acc[k + 1] << 63 } else { 0 };
                out[k] = acc[k] | (acc[k] << 1) | lo_carry | (acc[k] >> 1) | hi_carry;
            }
        }
    }
}

/// Empacota a extensão viva `(lo, hi)` no cabeçalho de uma linha do `live_tiles`.
#[inline]
#[must_use]
pub(crate) fn pack_live(lo: i32, hi: i32) -> u64 {
    u64::from(lo as u32) | (u64::from(hi as u32) << 32)
}

/// O inverso de [`pack_live`].
#[inline]
#[must_use]
pub(crate) fn unpack_live(w: u64) -> (i32, i32) {
    (w as u32 as i32, (w >> 32) as u32 as i32)
}

#[cfg(test)]
#[path = "faixa_tiles_tests.rs"]
mod tests;
