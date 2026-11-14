//! **A caixa que a óptica fora recompõe é a que MUDOU** (B14.1, `docs/Painter/missao_4k/BACKLOG.md`): o passo da sim
//! suja a caixa do FLUIDO inteira (a de antes e a de depois) mesmo quando nenhuma célula mudou — medido a 1440²
//! células, 2 passos por rodada a ~130 mil células com 0 mudadas. O `dirty` da tela continua a união; à parte, o sujo
//! do PASSO e o das outras portas. Com o espelho da óptica na mão (a entrada da imagem anterior, logo a grade de
//! então), [`Engine::take_dirty_pela_otica`] troca a caixa do passo pela das células que diferem AO BIT.

use super::espelho::Espelho;
use super::{Dirty, Engine, conta_sujo, nova_revisao};
use crate::grid::Grid;

/// O `dirty` partido: o que o passo da sim marcou × o das outras portas.
#[derive(Clone, Debug)]
pub(super) struct SujoDaOtica {
    pub(super) passo: Dirty,
    /// As outras portas em PEDAÇOS (B14.1, a última envolvente): a sua união é a caixa de sempre.
    pub(super) alheio: Pedacos,
}

impl Default for SujoDaOtica {
    fn default() -> Self {
        Self {
            passo: Dirty::Clean,
            alheio: Pedacos::default(),
        }
    }
}

/// Retângulos sujos que não se tocam (B14.1): um novo junta-se aos que toca (até nenhum tocar), senão entra à parte.
/// `[Full]` = a tela inteira; vazio = limpo. Cobre exactamente o que a união cobria menos os vãos entre pedaços.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pedacos(Vec<Dirty>);

impl Pedacos {
    /// A porta de [`Dirty::une`] (com a folga de uma célula), em pedaços.
    pub(super) fn une(&mut self, r: Caixa, w: usize, h: usize) {
        let mut d = Dirty::Clean;
        d.une(r, w, h);
        self.junta(d);
    }

    /// Junta um sujo já com a folga.
    pub fn junta(&mut self, d: Dirty) {
        let Dirty::Rect { x0, y0, x1, y1 } = d else {
            if d == Dirty::Full {
                self.0 = vec![Dirty::Full];
            }
            return;
        };
        if self.cheio() {
            return;
        }
        let mut c = (x0, y0, x1, y1);
        while let Some(k) = self.0.iter().position(|p| match *p {
            Dirty::Rect { x0, y0, x1, y1 } => x0 <= c.2 && c.0 <= x1 && y0 <= c.3 && c.1 <= y1,
            _ => false,
        }) {
            if let Dirty::Rect { x0, y0, x1, y1 } = self.0.swap_remove(k) {
                c = (c.0.min(x0), c.1.min(y0), c.2.max(x1), c.3.max(y1));
            }
        }
        self.0.push(Dirty::Rect {
            x0: c.0,
            y0: c.1,
            x1: c.2,
            y1: c.3,
        });
    }

    /// A tela inteira.
    #[must_use]
    pub fn cheio(&self) -> bool {
        self.0.first() == Some(&Dirty::Full)
    }

    /// A envolvente (o `alheio` de antes do B14.1).
    #[must_use]
    pub fn uniao(&self) -> Dirty {
        self.0.iter().fold(Dirty::Clean, |mut u, p| {
            match (*p, &mut u) {
                (Dirty::Full, _) => u = Dirty::Full,
                (_, Dirty::Full) | (Dirty::Clean, _) => {}
                (r, Dirty::Clean) => u = r,
                (
                    Dirty::Rect { x0, y0, x1, y1 },
                    Dirty::Rect {
                        x0: a,
                        y0: b,
                        x1: c,
                        y1: e,
                    },
                ) => {
                    (*a, *b, *c, *e) = ((*a).min(x0), (*b).min(y0), (*c).max(x1), (*e).max(y1));
                }
            }
            u
        })
    }

    /// Os pedaços (`[Full]` = a tela inteira).
    #[must_use]
    pub fn pedacos(&self) -> &[Dirty] {
        &self.0
    }

    /// Os pedaços, consumidos.
    #[must_use]
    pub fn em_vec(self) -> Vec<Dirty> {
        self.0
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

    /// O [`Engine::take_dirty`] de casa partido em `(passo, alheio)`; a união deles é a de sempre. O do passo volta ao
    /// worker por [`Engine::devolve_sujo_do_passo`], onde o espelho o reduz ao que mudou (B14.1: a água herdada no
    /// pen-down era a caixa do fluido inteira, a envolvente de todas as pinceladas molhadas).
    pub fn take_dirty_partido(&mut self) -> (Dirty, Pedacos) {
        self.dirty = Dirty::Clean;
        let SujoDaOtica { passo, alheio } = std::mem::take(&mut self.sujo_da_otica);
        (passo, alheio)
    }

    /// Devolve ao motor um sujo do PASSO tirado por [`Engine::take_dirty_partido`].
    #[track_caller]
    pub fn devolve_sujo_do_passo(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        self.merge_dirty_do_passo(x0, y0, x1, y1);
    }

    /// O [`Engine::take_dirty`] da óptica fora, ANTES de [`Engine::copia_para`] sobre o mesmo `espelho`: as outras
    /// portas ∪ a caixa (com uma célula de folga) das células da camada activa cujos planos da óptica diferem ao bit
    /// entre a grade e o espelho dentro do sujo do passo. Sem espelho de confiança (outra forma, um plano de outro
    /// tamanho, um escalar mudado, o espelho por copiar inteiro), a união de sempre.
    pub fn take_dirty_pela_otica(&mut self, espelho: &Espelho) -> Dirty {
        match self.tira_pela_otica(espelho) {
            PelaOtica::Uniao(d) => d,
            PelaOtica::Faixas { alheio, faixas } => {
                let mut d = alheio.uniao();
                for r in faixas {
                    d.une(r, self.width, self.height);
                }
                d
            }
        }
    }

    /// O mesmo sujo em PEDAÇOS (B14.1): cada pedaço da porta alheia e cada faixa de linhas mudadas (separadas por ≥ 1
    /// linha sem mudança), cada uma com a folga de uma célula. A união deles é [`Engine::take_dirty_pela_otica`]; vazio = limpo.
    pub fn take_dirty_pela_otica_em_faixas(&mut self, espelho: &Espelho) -> Vec<Dirty> {
        match self.tira_pela_otica(espelho) {
            PelaOtica::Uniao(Dirty::Clean) => Vec::new(),
            PelaOtica::Uniao(d) => vec![d],
            PelaOtica::Faixas { alheio, faixas } => {
                let (w, h) = (self.width, self.height);
                alheio
                    .em_vec()
                    .into_iter()
                    .chain(faixas.into_iter().map(|r| {
                        let mut f = Dirty::Clean;
                        f.une(r, w, h);
                        f
                    }))
                    .collect()
            }
        }
    }

    fn tira_pela_otica(&mut self, espelho: &Espelho) -> PelaOtica {
        let uniao = std::mem::replace(&mut self.dirty, Dirty::Clean);
        let SujoDaOtica { passo, alheio } = std::mem::take(&mut self.sujo_da_otica);
        let Dirty::Rect { x0, y0, x1, y1 } = passo else {
            // Sem passo o espelho não reduz nada, mas a porta alheia continua em pedaços (B14.1).
            return match passo {
                Dirty::Clean if !alheio.cheio() => PelaOtica::Faixas {
                    alheio,
                    faixas: Vec::new(),
                },
                _ => PelaOtica::Uniao(uniao),
            };
        };
        if alheio.cheio() || matches!(self.espelho_sujo, Dirty::Full) {
            return PelaOtica::Uniao(uniao);
        }
        let mesma_forma = espelho.camadas.len() == self.layers.len()
            && espelho.camadas.iter().zip(&self.layers).all(|(c, l)| {
                (c.grid.w, c.grid.h, c.grid.s, c.grid.rows)
                    == (l.grid.w, l.grid.h, l.grid.s, l.grid.rows)
            });
        if !mesma_forma {
            return PelaOtica::Uniao(uniao);
        }
        let (g, e) = (
            &self.layers[self.active_layer].grid,
            &espelho.camadas[self.active_layer].grid,
        );
        match faixas_que_mudaram(g, e, (x0, y0, x1, y1)) {
            Some(faixas) => PelaOtica::Faixas { alheio, faixas },
            None => PelaOtica::Uniao(uniao),
        }
    }
}

/// O sujo tirado pela óptica: a união (sem espelho de confiança) ou a porta alheia e as faixas sem folga.
enum PelaOtica {
    Uniao(Dirty),
    Faixas { alheio: Pedacos, faixas: Vec<Caixa> },
}

/// As caixas de `caixa` (presa a `[1..=w]×[1..=h]`) onde algum plano da óptica difere ao bit entre `g` e o espelho `e`,
/// de cima para baixo: as faixas de linhas mudadas (uma linha sem diferença parte), cada uma partida em colunas (uma
/// coluna sem diferença parte; o anel que `solver::apply_boundaries` reescreve com água junto à borda muda uma coluna
/// em todas as linhas e ligava as faixas, B14-cont3) e de novo em linhas, até nenhuma se partir; cada caixa é a
/// envolvente justa das suas diferenças. Vazio = nenhuma; `None` = o espelho não serve (um plano, um escalar).
fn faixas_que_mudaram(g: &Grid, e: &Grid, (x0, y0, x1, y1): Caixa) -> Option<Vec<Caixa>> {
    if !mesmos_escalares(g, e) || !mesmos_tamanhos(g, e) {
        return None;
    }
    let (xa, xb) = (x0.max(1) as usize, (x1.max(0) as usize).min(g.w));
    let (ya, yb) = (y0.max(1) as usize, (y1.max(0) as usize).min(g.h));
    let mut faixas = Vec::new();
    if xa > xb || ya > yb {
        return Some(faixas);
    }
    let s = g.s;
    let difere = |x: usize, y: usize| difere_em(g, e, y * s + x);
    let i = |v: usize| i32::try_from(v).unwrap_or(i32::MAX);
    let mut linhas: Vec<(usize, usize, usize, usize)> = Vec::new();
    let mut aberta: Option<(usize, usize, usize, usize)> = None;
    for y in ya..=yb {
        let Some(l) = (xa..=xb).find(|&x| difere(x, y)) else {
            linhas.extend(aberta.take());
            continue;
        };
        let r = (l..=xb).rev().find(|&x| difere(x, y)).unwrap_or(l);
        aberta = Some(match aberta {
            Some((lo, topo, hi, _)) => (lo.min(l), topo, hi.max(r), y),
            None => (l, y, r, y),
        });
    }
    linhas.extend(aberta);
    for (lo, topo, hi, fundo) in linhas {
        let mw = hi + 1 - lo;
        let mascara: Vec<bool> = (topo..=fundo)
            .flat_map(|y| (lo..=hi).map(move |x| (x, y)))
            .map(|(x, y)| difere(x, y))
            .collect();
        let mut locais = Vec::new();
        parte(&mascara, mw, (0, 0, mw - 1, fundo - topo), &mut locais);
        faixas.extend(
            locais
                .into_iter()
                .map(|(a, b, c, d)| (i(lo + a), i(topo + b), i(lo + c), i(topo + d))),
        );
    }
    Some(faixas)
}

/// O corte XY da `mascara` (`mw` colunas) dentro de `(x0, y0, x1, y1)`: as corridas de linhas com alguma diferença; com
/// uma só, as de colunas; com uma só de cada, a caixa justa. Cada ramo encolhe estritamente: termina.
fn parte(
    m: &[bool],
    mw: usize,
    (x0, y0, x1, y1): (usize, usize, usize, usize),
    out: &mut Vec<(usize, usize, usize, usize)>,
) {
    let corridas = |n0: usize, n1: usize, tem: &dyn Fn(usize) -> bool| {
        let mut v: Vec<(usize, usize)> = Vec::new();
        for k in n0..=n1 {
            if !tem(k) {
                continue;
            }
            match v.last_mut() {
                Some((_, b)) if *b + 1 == k => *b = k,
                _ => v.push((k, k)),
            }
        }
        v
    };
    let ls = corridas(y0, y1, &|y| m[y * mw + x0..=y * mw + x1].contains(&true));
    match ls.as_slice() {
        [] => {}
        &[(ya, yb)] => {
            let cs = corridas(x0, x1, &|x| (ya..=yb).any(|y| m[y * mw + x]));
            match cs.as_slice() {
                [] => {}
                &[(xa, xb)] => out.push((xa, ya, xb, yb)),
                _ => cs
                    .iter()
                    .for_each(|&(xa, xb)| parte(m, mw, (xa, ya, xb, yb), out)),
            }
        }
        _ => ls
            .iter()
            .for_each(|&(ya, yb)| parte(m, mw, (x0, ya, x1, yb), out)),
    }
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
        || g.material.get(i) != e.material.get(i)
        || g.aparencia.get(i) != e.aparencia.get(i)
        || g.oleo.get(i) != e.oleo.get(i)
        || c(&g.tinta, &e.tinta, i)
        || c(&g.pigmento, &e.pigmento, i)
}

#[cfg(test)]
#[path = "caixa_do_passo_tests.rs"]
mod tests;
