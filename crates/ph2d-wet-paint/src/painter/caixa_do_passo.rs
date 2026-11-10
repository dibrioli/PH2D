//! **A caixa que a óptica fora recompõe é a que MUDOU** (B14.1, `docs/Painter/missao_4k/BACKLOG.md`): o passo da sim
//! suja a caixa do FLUIDO inteira (a de antes e a de depois) mesmo quando nenhuma célula mudou — medido a 1440²
//! células, 2 passos por rodada a ~130 mil células com 0 mudadas. O `dirty` da tela continua a união; à parte, o sujo
//! do PASSO e o das outras portas. Com o espelho da óptica na mão (a entrada da imagem anterior, logo a grade de
//! então), [`Engine::take_dirty_pela_otica`] troca a caixa do passo pela das células que diferem AO BIT.

use super::espelho::Espelho;
use super::{Dirty, Engine, conta_sujo, nova_revisao};
use crate::grid::Grid;

/// O `dirty` partido: o que o passo da sim marcou × o das outras portas.
#[derive(Clone, Copy, Debug)]
pub(super) struct SujoDaOtica {
    pub(super) passo: Dirty,
    pub(super) alheio: Dirty,
}

impl Default for SujoDaOtica {
    fn default() -> Self {
        Self {
            passo: Dirty::Clean,
            alheio: Dirty::Clean,
        }
    }
}

type Caixa = (i32, i32, i32, i32);

impl Engine {
    /// A porta do PASSO da sim: a de [`Engine::merge_dirty`], com a caixa no sujo do passo.
    #[track_caller]
    pub(super) fn merge_dirty_do_passo(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        self.revisao = nova_revisao();
        self.suja_o_espelho((x0, y0, x1, y1));
        conta_sujo::regista((i64::from(x1 - x0 + 1).max(0) * i64::from(y1 - y0 + 1).max(0)) as u64);
        self.dirty.une((x0, y0, x1, y1), self.width, self.height);
        self.sujo_da_otica
            .passo
            .une((x0, y0, x1, y1), self.width, self.height);
    }

    /// O [`Engine::take_dirty`] da óptica fora, ANTES de [`Engine::copia_para`] sobre o mesmo `espelho`: as outras
    /// portas ∪ a caixa (com uma célula de folga) das células da camada activa cujos planos da óptica diferem ao bit
    /// entre a grade e o espelho dentro do sujo do passo. Sem espelho de confiança (outra forma, um plano de outro
    /// tamanho, um escalar mudado, o espelho por copiar inteiro), a união de sempre.
    pub fn take_dirty_pela_otica(&mut self, espelho: &Espelho) -> Dirty {
        let uniao = std::mem::replace(&mut self.dirty, Dirty::Clean);
        let SujoDaOtica { passo, alheio } = std::mem::take(&mut self.sujo_da_otica);
        let Dirty::Rect { x0, y0, x1, y1 } = passo else {
            return uniao;
        };
        if matches!(alheio, Dirty::Full) || matches!(self.espelho_sujo, Dirty::Full) {
            return uniao;
        }
        let mesma_forma = espelho.camadas.len() == self.layers.len()
            && espelho.camadas.iter().zip(&self.layers).all(|(c, l)| {
                (c.grid.w, c.grid.h, c.grid.s, c.grid.rows)
                    == (l.grid.w, l.grid.h, l.grid.s, l.grid.rows)
            });
        if !mesma_forma {
            return uniao;
        }
        let (g, e) = (
            &self.layers[self.active_layer].grid,
            &espelho.camadas[self.active_layer].grid,
        );
        let Some(mudou) = caixa_que_mudou(g, e, (x0, y0, x1, y1)) else {
            return uniao;
        };
        let mut d = alheio;
        if let Some(r) = mudou {
            d.une(r, self.width, self.height);
        }
        d
    }
}

/// A caixa das células de `caixa` (presa a `[1..=w]×[1..=h]`) onde algum plano da óptica difere ao bit entre `g` e
/// o espelho `e`: `Some(None)` = nenhuma; `None` = o espelho não serve (um plano de outro tamanho, um escalar).
/// Linhas de cima e de baixo até à primeira diferença; no meio, só as colunas FORA das achadas.
fn caixa_que_mudou(g: &Grid, e: &Grid, (x0, y0, x1, y1): Caixa) -> Option<Option<Caixa>> {
    if !mesmos_escalares(g, e) || !mesmos_tamanhos(g, e) {
        return None;
    }
    let (xa, xb) = (x0.max(1) as usize, (x1.max(0) as usize).min(g.w));
    let (ya, yb) = (y0.max(1) as usize, (y1.max(0) as usize).min(g.h));
    if xa > xb || ya > yb {
        return Some(None);
    }
    let s = g.s;
    let difere = |x: usize, y: usize| difere_em(g, e, y * s + x);
    let linha = |y: usize| (xa..=xb).find(|&x| difere(x, y));
    let Some((topo, xt)) = (ya..=yb).find_map(|y| linha(y).map(|x| (y, x))) else {
        return Some(None);
    };
    let fundo = (topo..=yb)
        .rev()
        .find(|&y| linha(y).is_some())
        .unwrap_or(topo);
    let (mut lo, mut hi) = (xt, xt);
    for y in topo..=fundo {
        if let Some(x) = (xa..lo).find(|&x| difere(x, y)) {
            lo = x;
        }
        if let Some(x) = (hi + 1..=xb).rev().find(|&x| difere(x, y)) {
            hi = x;
        }
    }
    let i = |v: usize| i32::try_from(v).unwrap_or(i32::MAX);
    Some(Some((i(lo), i(topo), i(hi), i(fundo))))
}

/// Os escalares que a óptica lê (os de `espelho::escalares` sem a caixa do fluido, que ela não lê).
fn mesmos_escalares(g: &Grid, e: &Grid) -> bool {
    (g.oleo_real, g.flow, g.spans_enabled) == (e.oleo_real, e.flow, e.spans_enabled)
        && g.suave_teto.to_bits() == e.suave_teto.to_bits()
        && g.borda_fundo == e.borda_fundo
        && (g.paper_preset, g.paper_sheet) == (e.paper_preset, e.paper_sheet)
}

/// Os planos da óptica (os de `espelho::copia_faixas`) do mesmo tamanho.
fn mesmos_tamanhos(g: &Grid, e: &Grid) -> bool {
    [
        (g.film.len(), e.film.len()),
        (g.susp.len(), e.susp.len()),
        (g.susp_rgb.len(), e.susp_rgb.len()),
        (g.sett.len(), e.sett.len()),
        (g.sett_rgb.len(), e.sett_rgb.len()),
        (g.relevo.len(), e.relevo.len()),
        (g.altura.len(), e.altura.len()),
        (g.espessura.len(), e.espessura.len()),
        (g.crista.len(), e.crista.len()),
        (g.nivelado.len(), e.nivelado.len()),
        (g.vela.len(), e.vela.len()),
        (g.material.len(), e.material.len()),
        (g.aparencia.len(), e.aparencia.len()),
        (g.oleo.len(), e.oleo.len()),
        (g.tinta.len(), e.tinta.len()),
        (g.pigmento.len(), e.pigmento.len()),
        (g.wet.len(), e.wet.len()),
        (g.paper.len(), e.paper.len()),
    ]
    .into_iter()
    .all(|(a, b)| a == b)
}

/// A célula `i` difere em algum plano da óptica: `f32` ao bit; as células compostas pela igualdade delas.
fn difere_em(g: &Grid, e: &Grid, i: usize) -> bool {
    fn f(a: &[f32], b: &[f32], i: usize) -> bool {
        a.get(i).map(|v| v.to_bits()) != b.get(i).map(|v| v.to_bits())
    }
    fn rgb(a: &[[f32; 3]], b: &[[f32; 3]], i: usize) -> bool {
        a.get(i).map(|v| v.map(f32::to_bits)) != b.get(i).map(|v| v.map(f32::to_bits))
    }
    fn c<T: PartialEq>(a: &[T], b: &[T], i: usize) -> bool {
        a.get(i) != b.get(i)
    }
    f(&g.film, &e.film, i)
        || f(&g.susp, &e.susp, i)
        || f(&g.sett, &e.sett, i)
        || rgb(&g.susp_rgb, &e.susp_rgb, i)
        || rgb(&g.sett_rgb, &e.sett_rgb, i)
        || f(&g.relevo, &e.relevo, i)
        || f(&g.altura, &e.altura, i)
        || f(&g.espessura, &e.espessura, i)
        || f(&g.crista, &e.crista, i)
        || f(&g.nivelado, &e.nivelado, i)
        || f(&g.vela, &e.vela, i)
        || f(&g.paper, &e.paper, i)
        || g.wet.get(i) != e.wet.get(i)
        || c(&g.material, &e.material, i)
        || c(&g.aparencia, &e.aparencia, i)
        || c(&g.oleo, &e.oleo, i)
        || c(&g.tinta, &e.tinta, i)
        || c(&g.pigmento, &e.pigmento, i)
}

#[cfg(test)]
#[path = "caixa_do_passo_tests.rs"]
mod tests;
