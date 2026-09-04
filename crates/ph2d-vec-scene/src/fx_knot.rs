//! **Knot** — o entrelace celta: onde o caminho se cruza, uma fita passa por CIMA e a outra por
//! BAIXO, e a de baixo ganha um VÃO na travessia. É o *Knot* LPE do Inkscape, e o idioma da
//! trança / do nó celta que nenhum editor livre entrega bem.
//!
//! # O mecanismo, e por que ele não precisa de z-buffer
//!
//! Um cruzamento é um ponto por onde o caminho passa DUAS vezes (auto-interseção), ou onde dois
//! contornos se cruzam. Em cada cruzamento uma passagem é a de CIMA (fica inteira) e a outra é a
//! de BAIXO (perde um pedaço de arco — o VÃO). Como as duas fitas têm o mesmo traço e a de cima
//! tem tinta onde a de baixo tem o vão, o resultado LÊ como *"uma passa sobre a outra"* sem
//! nenhuma ordenação de profundidade: o vão É a sombra. É o método do Inkscape.
//!
//! # Alternância — a propriedade que faz o nó parecer tecido
//!
//! Seguindo UMA fita, ela passa por cima, por baixo, por cima, por baixo... Isso é a alternância,
//! e produz-se percorrendo o caminho e trocando o lado a cada travessia (`over` que vira a cada
//! ponta de cruzamento em ordem de arco). Quando a projeção não permite alternância perfeita, a
//! regra garante **exatamente um vão por cruzamento** (nunca dois — que apagaria a fita — nem
//! zero — que a deixaria sólida): em caso de empate de paridade, a passagem de arco maior mergulha.
//! O `swap` inverte todos.
//!
//! # Detecta na poligonal, corta na curva
//!
//! As travessias são achadas na POLIGONAL densa (interseção reta-reta, com a posição de arco de
//! cada passagem); o VÃO é cortado na CURVA de Bézier pela MESMA máquina de arco do Trim
//! ([`crate::fx_trim::pieces_between`]/[`rebuild`](crate::fx_trim::rebuild)) — as fitas saem lisas.

use crate::arc_cut::{Crossing, EPS, Edge, Geom, MAX_SAMPLES, Merge, crossings, strands_uniform};
use crate::effect::FxCtx;
use crate::{Contour, VecPath, VecVertex};

/// **Os parâmetros de um Knot.** Neutro em `gap == 0`.
#[derive(Copy, Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct KnotSpec {
    /// A largura do VÃO na fita de baixo, em **percentagem** da referência da forma (`100` = a
    /// média das dimensões). É a "espessura aparente" do entrelace.
    pub gap: f64,
    /// Inverte quem passa por cima em TODOS os cruzamentos.
    pub swap: bool,
}

impl KnotSpec {
    /// Um Knot novo, no ponto NEUTRO.
    #[must_use]
    pub fn new() -> Self {
        Self {
            gap: 0.0,
            swap: false,
        }
    }

    /// Sem vão não há entrelace — e o neutro tem de ser no-op byte-idêntico (ADR-0132), o que
    /// mantém o `Cow::Borrowed` do `cooked()` vivo.
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.gap.abs() <= EPS
    }
}

/// Para cada travessia, decide qual passagem MERGULHA (ganha o vão). Devolve, por contorno, as
/// frações centrais dos vãos. Alternância por paridade de arco global, com garantia de exatamente
/// um vão por travessia; `swap` inverte todos.
fn dive_gaps(crossings: &[Crossing], num_contours: usize, swap: bool) -> Vec<Vec<f64>> {
    // Chave de arco global: contorno + fração (contorno 0 em [0,1), 1 em [1,2), ...).
    let key = |p: (usize, f64)| p.0 as f64 + p.1;
    // Pontas ordenadas por arco; `over` vira a cada ponta -> a fita alterna cima/baixo.
    let mut ends: Vec<(f64, usize, u8)> = Vec::with_capacity(crossings.len() * 2);
    for (k, c) in crossings.iter().enumerate() {
        ends.push((key(c.a), k, 0));
        ends.push((key(c.b), k, 1));
    }
    ends.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut over = vec![[false; 2]; crossings.len()];
    let mut flag = true;
    for (_, k, side) in &ends {
        over[*k][*side as usize] = flag;
        flag = !flag;
    }
    let mut gaps: Vec<Vec<f64>> = vec![Vec::new(); num_contours];
    for (k, c) in crossings.iter().enumerate() {
        let (a_over, b_over) = (over[k][0], over[k][1]);
        // Exatamente UM vão: se a paridade empatar, mergulha a passagem de arco MAIOR (determinista).
        let mut dive_a = if a_over == b_over {
            key(c.a) > key(c.b)
        } else {
            b_over // se B está por cima, A mergulha
        };
        if swap {
            dive_a = !dive_a;
        }
        let (c_idx, f) = if dive_a { c.a } else { c.b };
        gaps[c_idx].push(f);
    }
    gaps
}

/// **Aplica o Knot ao caminho inteiro.** Whole-path (não por-contorno) porque uma travessia pode
/// ser entre dois contornos — e uma tela sem travessia sai clonada, sem tecer nada.
#[must_use]
pub fn knot_path(path: &VecPath, spec: &KnotSpec, ctx: &FxCtx) -> VecPath {
    let mut out = path.clone();
    if spec.is_neutral() || ctx.ref_size <= EPS {
        return out;
    }
    // Todos os contornos: o primário + os subpaths.
    let mut geoms: Vec<Geom> = Vec::new();
    if let Some(g) = Geom::of(&path.verts, path.closed) {
        geoms.push(g);
    }
    for c in &path.subpaths {
        if let Some(g) = Geom::of(&c.verts, c.closed) {
            geoms.push(g);
        }
    }
    if geoms.is_empty() {
        return out;
    }
    let edges: Vec<Vec<Edge>> = geoms.iter().map(Geom::edges).collect();
    if edges.iter().map(Vec::len).sum::<usize>() > MAX_SAMPLES {
        return out; // caminho patológico: não teço, devolvo intacto
    }
    let xings = crossings(&geoms, &edges, Merge::do_corte(ctx.ref_size));
    if xings.is_empty() {
        return out; // nada se cruza — nada a tecer
    }
    let gaps = dive_gaps(&xings, geoms.len(), spec.swap);
    let gap_len = ctx.ref_size * (spec.gap / 100.0);

    let mut contours: Vec<(Vec<VecVertex>, bool)> = Vec::new();
    for (i, g) in geoms.iter().enumerate() {
        let gap_frac = (gap_len / g.total).min(0.999);
        contours.extend(strands_uniform(g, &gaps[i], gap_frac));
    }
    if contours.is_empty() {
        return out;
    }
    let (v0, c0) = contours.remove(0);
    out.verts = v0;
    out.closed = c0;
    out.subpaths = contours
        .into_iter()
        .map(|(verts, closed)| Contour { verts, closed })
        .collect();
    out
}

#[cfg(test)]
#[path = "fx_knot_tests.rs"]
mod tests;
