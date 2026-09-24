//! **O DEPÓSITO de um dab de tinta — a lei da célula, e o caminho do produto por LINHAS**
//! ([ADR-0175](../../../../docs/architecture/decisions/0175-o-deposito-do-dab-do-wet-paint-corre-em-linhas-disjuntas.md)).
//!
//! Filho de [`super`] por responsabilidade: o pai guarda *o que a janela do rasto É* (a âncora, o
//! crescimento, o transfer), aqui fica *o que um dab POUSA nela e no grid*.
//!
//! ## Porquê existe (medido 2026-09-24, `examples/mede_o_wet_paint`, tela 4096², release)
//!
//! Com a caneta em baixo a simulação está PARADA, e o quadro de pintar é quase só isto: com raio
//! 250 cada quadro pousa ~5 dabs e **cada dab custava `~5,7 ms`** (`~251 000` células), ou seja
//! `28` dos `31 ms` do quadro — o artista sentia `~33` FPS a pintar e `~22` com raio 400. E corria
//! numa thread só, com o pool do motor parado ao lado.
//!
//! ## Porquê é byte-idêntico (as três condições do [`crate::par`])
//!
//! 1. **cada linha escreve SÓ a própria linha** — o carimbo de umidade `wet[i]` na linha do grid e
//!    o pigmento/água na linha da janela do rasto; cada célula é visitada UMA vez por dab;
//! 2. **o que ela lê, ninguém escreve neste passe** — `susp`, `sett`, `paper` e `film` são só
//!    lidos, a silhueta e o grão são funções puras da célula ([`crate::brush::CellFn`]);
//! 3. **a única redução é de RETÂNGULOS** — a extensão da janela e o `wrote` são caixas envolventes,
//!    união de mínimos e máximos inteiros: associativa **e** comutativa.
//!
//! ⚠️ **E a lei da célula é UMA função** ([`DepositLaw::cell`]) com dois chamadores — o caminho do
//! produto por linhas e o do próprio motor em série ([`super::Trail::accumulate_paint`], que a
//! impressão digital da sessão fixa ao bit) —; as rotas só escolhem o caminhante, nunca a
//! aritmética. E o caminho do motor é o ORÁCULO independente da caminhada por linhas
//! (`tests/it/deposit_rows.rs`): com uma silhueta que reproduz a queda dele, os dois pousam o
//! mesmo traço ao bit.

use super::{Accumulated, TouchedRect, Trail, touch_ext};
use crate::brush::{CellFn, shaped_bounds, stamp_row_shaped};
use crate::grid::{Grid, wet_byte_from_paper};
use crate::par::{self, Rows};
use crate::sim::Params;
use crate::trail::Dab;
use crate::tuning::Knob;

/// A caixa vazia (`x1 < x0`), a sentinela que o [`touch_ext`] já usa.
const VAZIA: (i32, i32, i32, i32) = (0, 0, -1, -1);

/// A união de duas caixas envolventes, com [`VAZIA`] como identidade — a redução das linhas.
fn uniao(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> (i32, i32, i32, i32) {
    if a.2 < a.0 {
        return b;
    }
    if b.2 < b.0 {
        return a;
    }
    (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))
}

/// Os números de um dab que a lei da célula lê — tirados UMA vez por dab, fora do laço.
#[derive(Clone, Copy)]
pub(super) struct DepositLaw {
    gain: f64,
    gate: f64,
    pig_cap: f64,
    ext_dry_brush: f64,
    ext_wet_soften: f64,
    ext_bypass: bool,
    intensity: f64,
    dry_gate: f64,
    pub(super) water_amount: f64,
}

impl DepositLaw {
    pub(super) fn new(p: &Params, dab: &Dab, ext_bypass: bool) -> Self {
        Self {
            gain: p.k(Knob::PigmentPerDab),
            gate: p.k(Knob::PaperGate),
            pig_cap: p.k(Knob::GateSaturation),
            ext_dry_brush: p.k(Knob::ExtDryBrush),
            ext_wet_soften: p.k(Knob::ExtWetSoften),
            ext_bypass,
            intensity: dab.intensity,
            dry_gate: dab.dry_gate,
            water_amount: dab.water_amount,
        }
    }

    pub(super) fn gain(&self) -> f64 {
        self.gain
    }

    /// **A lei de UMA célula**: `Some((depósito, byte de umidade))`, ou `None` quando nada pousa —
    /// e então nem a umidade é carimbada.
    #[inline]
    pub(super) fn cell(
        &self,
        susp: f32,
        sett: f32,
        paper: f32,
        film: f32,
        fall: f64,
        texv: f64,
    ) -> Option<(f64, u8)> {
        let mut stamp = fall * texv * self.intensity;
        if stamp > 1.0 {
            stamp = 1.0;
        }
        if stamp <= 0.0 {
            return None;
        }
        // Paper gate: tooth peaks always take pigment, valleys reject
        // it — that per-pixel pass/reject IS the granulation. Heavily
        // loaded cells read a flat 0.45 tooth (the grain is buried).
        let tooth = if (susp as f64 + sett as f64) < self.pig_cap {
            paper as f64
        } else {
            0.45
        };
        let mut deposit = stamp - (1.0 - tooth) * self.gate;
        if !self.ext_bypass {
            // Dry-brush extension: raise the gate subtraction.
            deposit -= self.dry_gate * self.ext_dry_brush * 0.6;
        }
        if deposit <= 0.0 {
            return None;
        }
        if !self.ext_bypass {
            // Wet-edge softening extension: thin the rim on wet paper.
            let softness = (film as f64 / 3.0).min(1.0) * self.ext_wet_soften;
            deposit *= 1.0 - softness * (1.0 - fall);
        }
        // Wetness seed: OVERWRITE (not max) — repainting can dry the
        // byte back down.
        Some((deposit, wet_byte_from_paper(tooth)))
    }
}

/// Uma linha do depósito: o `y`, a linha do grid (`wet`) e — se a janela a cobre — a linha do
/// pigmento e a da água do rasto.
type Linha<'a> = (i32, &'a mut [u8], Option<(&'a mut [f32], &'a mut [f32])>);

impl Trail {
    /// O depósito de um dab com a silhueta do HOSPEDEIRO, caminhado por linhas na rota `mode`.
    ///
    /// Devolve o que a porta pública devolvia (a janela encheu? · onde o dab escreveu no grid) e
    /// actualiza a extensão da janela. `mode = None` é o produto: a rota sai de [`Rows::pick`]
    /// com o piso MEDIDO [`par::MIN_CELLS_DEPOSIT`].
    #[allow(clippy::too_many_arguments)]
    pub(super) fn deposit_shaped(
        &mut self,
        g: &mut Grid,
        tex: &[f32],
        dab: &Dab,
        law: DepositLaw,
        sil: CellFn<'_>,
        grain: Option<CellFn<'_>>,
        mode: Option<Rows>,
    ) -> (i32, i32, i32, i32) {
        let Some((x0, y0, x1, y1)) = shaped_bounds(g.w, g.h, dab.x, dab.y, dab.r) else {
            return VAZIA;
        };
        let mode = mode.unwrap_or_else(|| {
            Rows::pick(
                (y1 - y0 + 1) as usize,
                (x1 - x0 + 1) as usize,
                par::MIN_CELLS_DEPOSIT,
            )
        });
        let (s, size, half) = (g.s, self.size, self.half);
        let (ax, ay) = (self.anchor_x, self.anchor_y);
        let gain = law.gain();
        let Grid {
            susp,
            sett,
            paper,
            film,
            wet,
            ..
        } = g;
        let (susp, sett, paper, film) = (&*susp, &*sett, &*paper, &*film);
        // As linhas da janela que caem dentro da caixa do dab — cada uma é a MESMA linha `y` do
        // grid, com a âncora da janela a ligá-las.
        let n_linhas = (y1 - y0 + 1) as usize;
        let mut janela: Vec<Option<(&mut [f32], &mut [f32])>> =
            (0..n_linhas).map(|_| None).collect();
        for (ly, par_de_linhas) in self
            .pig
            .chunks_mut(size as usize)
            .zip(self.water.chunks_mut(size as usize))
            .enumerate()
        {
            let y = ly as i32 - half + ay;
            if (y0..=y1).contains(&y) {
                janela[(y - y0) as usize] = Some(par_de_linhas);
            }
        }
        let faixa = &mut wet[y0 as usize * s..(y1 as usize + 1) * s];
        let mut linhas: Vec<Linha<'_>> = faixa
            .chunks_mut(s)
            .zip(janela)
            .enumerate()
            .map(|(k, (w, j))| (y0 + k as i32, w, j))
            .collect();
        let linha = |it: &mut Linha<'_>| -> ((i32, i32, i32, i32), (i32, i32, i32, i32)) {
            let (y, wet_row, win) = it;
            let y = *y;
            let base = y as usize * s;
            let ly = y - ay + half;
            let mut ext = VAZIA;
            let mut wrote = VAZIA;
            stamp_row_shaped(y, x0, x1, tex, dab.x, dab.y, sil, grain, |x, fall, texv| {
                let i = base + x as usize;
                let Some((deposit, wb)) = law.cell(susp[i], sett[i], paper[i], film[i], fall, texv)
                else {
                    return;
                };
                // ⚠️ **Esta é uma escrita no DOCUMENTO** — ver o doc do pai: é dela que o véu do
                // Show Wet se alimenta, e acontece mesmo fora da janela do rasto.
                wet_row[x as usize] = wb;
                touch_ext(&mut wrote, x, y);
                let lx = x - ax + half;
                if !(0..size).contains(&lx) {
                    return;
                }
                let Some((pig_row, water_row)) = win.as_mut() else {
                    return;
                };
                let l = lx as usize;
                pig_row[l] = (pig_row[l] as f64 + deposit * gain) as f32;
                water_row[l] = (water_row[l] as f64 + deposit * law.water_amount) as f32;
                touch_ext(&mut ext, lx, ly);
            });
            (ext, wrote)
        };
        let (ext, wrote) =
            par::walk_items_reduce(mode, &mut linhas, (VAZIA, VAZIA), linha, |a, b| {
                (uniao(a.0, b.0), uniao(a.1, b.1))
            });
        let janela_antes = (self.lx0, self.ly0, self.lx1, self.ly1);
        (self.lx0, self.ly0, self.lx1, self.ly1) = uniao(janela_antes, ext);
        wrote
    }

    /// Fecha um dab: conta-o e diz ao chamador se a janela encheu e onde o grid foi escrito.
    pub(super) fn accumulated(&mut self, wrote: (i32, i32, i32, i32)) -> Accumulated {
        self.dab_count += 1;
        Accumulated {
            window_full: self.dab_count > self.window_size,
            wrote: (wrote.2 >= wrote.0).then_some(TouchedRect {
                x0: wrote.0,
                y0: wrote.1,
                x1: wrote.2,
                y1: wrote.3,
            }),
        }
    }
}
