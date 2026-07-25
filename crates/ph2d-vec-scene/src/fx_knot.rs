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

use crate::arclen::{Cubic, arclen, arclen_to, point_at};
use crate::corner_live::segment;
use crate::effect::FxCtx;
use crate::fx_trim::{pieces_between, rebuild};
use crate::{Contour, VecPath, VecVertex};

/// Abaixo deste vão (fração) o efeito é o ponto neutro.
const EPS: f64 = 1e-9;

/// Amostras por segmento na poligonal de detecção. O recurso é o custo de `cooked()` (`O(E²)` nas
/// arestas); 16 dá precisão de arco melhor que 1/16 de segmento, e o vão tem largura de qualquer
/// forma. Guardado pelo teto de amostras.
const SAMPLES_PER_SEG: usize = 16;

/// Teto de amostras na poligonal inteira — guarda o `O(E²)` contra um caminho patológico.
const MAX_SAMPLES: usize = 4096;

/// Dois cruzamentos mais próximos do que isto (em unidades de MUNDO, relativo à referência) são o
/// mesmo — a poligonal densa pode reportar a mesma travessia por duas arestas vizinhas.
const MERGE_FRAC: f64 = 1e-3;

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

/// A geometria de um contorno, pronta para cortar por arco.
struct Geom {
    verts: Vec<VecVertex>,
    closed: bool,
    n: usize,
    segs: Vec<Cubic>,
    lens: Vec<f64>,
    total: f64,
}

impl Geom {
    fn of(verts: &[VecVertex], closed: bool) -> Option<Self> {
        let n = verts.len();
        if n < 2 {
            return None;
        }
        let seg_count = if closed { n } else { n - 1 };
        let segs: Vec<Cubic> = (0..seg_count).map(|i| segment(verts, i, n)).collect();
        let lens: Vec<f64> = segs.iter().map(arclen).collect();
        let total: f64 = lens.iter().sum();
        (total > EPS).then_some(Self {
            verts: verts.to_vec(),
            closed,
            n,
            segs,
            lens,
            total,
        })
    }

    /// A poligonal de detecção: arestas `(p0, p1, f0, f1)` com as frações de arco de cada ponta.
    /// A aresta de EMENDA de um fechado vai de `f_last` a `1.0` (não a 0), para o `lerp` da fração
    /// ser monótono na travessia perto da costura.
    fn edges(&self) -> Vec<Edge> {
        let mut pts: Vec<([f64; 2], f64)> = Vec::new();
        let mut walked = 0.0;
        for (i, seg) in self.segs.iter().enumerate() {
            for j in 0..SAMPLES_PER_SEG {
                #[allow(clippy::cast_precision_loss)]
                let t = j as f64 / SAMPLES_PER_SEG as f64;
                let f = (walked + arclen_to(seg, t)) / self.total;
                pts.push((point_at(seg, t), f));
            }
            walked += self.lens[i];
        }
        let m = pts.len();
        let mut out = Vec::with_capacity(m + 1);
        for i in 0..m - 1 {
            out.push(Edge::new(pts[i], pts[i + 1]));
        }
        if self.closed {
            // A emenda: última amostra -> a 1ª (mesmo ponto), fração de `f_last` a 1.0.
            out.push(Edge::new(pts[m - 1], (pts[0].0, 1.0)));
        } else {
            // Aberto: acrescenta a ponta final (t=1 do último segmento) e fecha a poligonal nela.
            let last = self.segs.len() - 1;
            let endp = point_at(&self.segs[last], 1.0);
            out.push(Edge::new(pts[m - 1], (endp, 1.0)));
        }
        out
    }
}

/// Uma aresta da poligonal de detecção.
#[derive(Copy, Clone)]
struct Edge {
    p0: [f64; 2],
    p1: [f64; 2],
    f0: f64,
    f1: f64,
}

impl Edge {
    fn new(a: ([f64; 2], f64), b: ([f64; 2], f64)) -> Self {
        Self {
            p0: a.0,
            p1: b.0,
            f0: a.1,
            f1: b.1,
        }
    }
}

/// Uma travessia: as duas passagens `(contorno, fração)` e o ponto onde ela cai.
struct Crossing {
    a: (usize, f64),
    b: (usize, f64),
    at: [f64; 2],
}

/// Interseção de dois segmentos de reta. `Some((ta, tb))` com os dois parâmetros ESTRITAMENTE
/// dentro (as pontas coincidentes não são travessia). Sem `atan2` nem transcendental (HR-5).
fn seg_cross(a0: [f64; 2], a1: [f64; 2], b0: [f64; 2], b1: [f64; 2]) -> Option<(f64, f64)> {
    let d1 = [a1[0] - a0[0], a1[1] - a0[1]];
    let d2 = [b1[0] - b0[0], b1[1] - b0[1]];
    let denom = d1[0] * d2[1] - d1[1] * d2[0];
    if denom.abs() < 1e-14 {
        return None;
    }
    let diff = [b0[0] - a0[0], b0[1] - a0[1]];
    let ta = (diff[0] * d2[1] - diff[1] * d2[0]) / denom;
    let tb = (diff[0] * d1[1] - diff[1] * d1[0]) / denom;
    const M: f64 = 1e-6;
    (ta > M && ta < 1.0 - M && tb > M && tb < 1.0 - M).then_some((ta, tb))
}

/// Acha todas as travessias entre as poligonais dos contornos (auto-interseções e cruzamentos
/// entre contornos). Salta pares de arestas ADJACENTES no MESMO contorno (partilham um vértice, não
/// são travessia).
fn crossings(geoms: &[Geom], edges: &[Vec<Edge>], span: f64) -> Vec<Crossing> {
    let merge = span * MERGE_FRAC;
    let mut out: Vec<Crossing> = Vec::new();
    for (ca, ea) in edges.iter().enumerate() {
        for (cb, eb) in edges.iter().enumerate().skip(ca) {
            let na = ea.len();
            for (i, &u) in ea.iter().enumerate() {
                let jstart = if ca == cb { i + 1 } else { 0 };
                for (j, &v) in eb.iter().enumerate().skip(jstart) {
                    // Arestas vizinhas (partilham vértice) não são travessia; a emenda de um fechado
                    // torna 0 e na-1 vizinhas também.
                    if ca == cb && (j == i + 1 || (geoms[ca].closed && i == 0 && j == na - 1)) {
                        continue;
                    }
                    let Some((ta, tb)) = seg_cross(u.p0, u.p1, v.p0, v.p1) else {
                        continue;
                    };
                    let at = [
                        u.p0[0] + ta * (u.p1[0] - u.p0[0]),
                        u.p0[1] + ta * (u.p1[1] - u.p0[1]),
                    ];
                    if out
                        .iter()
                        .any(|c| (c.at[0] - at[0]).hypot(c.at[1] - at[1]) < merge)
                    {
                        continue; // a mesma travessia por duas arestas vizinhas
                    }
                    let fa = u.f0 + ta * (u.f1 - u.f0);
                    let fb = v.f0 + tb * (v.f1 - v.f0);
                    out.push(Crossing {
                        a: (ca, fa % 1.0),
                        b: (cb, fb % 1.0),
                        at,
                    });
                }
            }
        }
    }
    out
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

/// As faixas a MANTER num contorno fechado = o círculo `[0,1)` menos os vãos. Cada faixa vira
/// `(lo, hi)` para o [`pieces_between`] (fechado: `hi` pode passar de 1, dá a volta pela emenda).
fn keep_ranges_closed(gaps_norm: &[(f64, f64)]) -> Vec<(f64, f64)> {
    // `gaps_norm`: (lo em [0,1), largura). Um vão que cobre a volta inteira apaga o contorno.
    if gaps_norm.iter().any(|&(_, w)| w >= 1.0 - EPS) {
        return Vec::new();
    }
    let inside = |x: f64| gaps_norm.iter().any(|&(l, w)| (x - l).rem_euclid(1.0) < w);
    let mut cuts: Vec<f64> = gaps_norm
        .iter()
        .flat_map(|&(l, w)| [l, (l + w).rem_euclid(1.0)])
        .collect();
    cuts.sort_by(f64::total_cmp);
    cuts.dedup_by(|a, b| (*a - *b).abs() < EPS);
    let mut keeps = Vec::new();
    let m = cuts.len();
    for i in 0..m {
        let a = cuts[i];
        let b = cuts[(i + 1) % m];
        let span = if b > a { b - a } else { b + 1.0 - a };
        let mid = (a + span * 0.5).rem_euclid(1.0);
        if !inside(mid) {
            keeps.push((a, a + span));
        }
    }
    keeps
}

/// As faixas a MANTER num contorno ABERTO = `[0,1]` menos os vãos (sem dar a volta).
fn keep_ranges_open(mut gaps: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    // `gaps`: (lo, hi) recortados a [0,1]. Funde os que se sobrepõem, devolve o complemento.
    gaps.retain(|&(lo, hi)| hi > lo + EPS);
    gaps.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut merged: Vec<(f64, f64)> = Vec::new();
    for g in gaps {
        match merged.last_mut() {
            Some(last) if g.0 <= last.1 + EPS => last.1 = last.1.max(g.1),
            _ => merged.push(g),
        }
    }
    let mut keeps = Vec::new();
    let mut cursor = 0.0;
    for (lo, hi) in merged {
        if lo > cursor + EPS {
            keeps.push((cursor, lo));
        }
        cursor = hi;
    }
    if cursor < 1.0 - EPS {
        keeps.push((cursor, 1.0));
    }
    keeps
}

/// Recorta um contorno nas faixas a manter, devolvendo as fitas (contornos ABERTOS). Um contorno
/// SEM vão sai inteiro (fechado como era) — não há travessia de baixo nele.
fn strands(g: &Geom, gap_centers: &[f64], gap_frac: f64) -> Vec<(Vec<VecVertex>, bool)> {
    if gap_centers.is_empty() {
        return vec![(g.verts.clone(), g.closed)];
    }
    let half = gap_frac * 0.5;
    let keeps = if g.closed {
        let norm: Vec<(f64, f64)> = gap_centers
            .iter()
            .map(|&c| ((c - half).rem_euclid(1.0), gap_frac))
            .collect();
        keep_ranges_closed(&norm)
    } else {
        let ranges: Vec<(f64, f64)> = gap_centers
            .iter()
            .map(|&c| ((c - half).max(0.0), (c + half).min(1.0)))
            .collect();
        keep_ranges_open(ranges)
    };
    keeps
        .into_iter()
        .filter_map(|(lo, hi)| {
            let pieces = pieces_between(&g.segs, &g.lens, g.total, lo, hi, g.closed);
            let v = rebuild(&g.verts, &g.segs, &pieces, g.n);
            (!v.is_empty()).then_some((v, false))
        })
        .collect()
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
    let xings = crossings(&geoms, &edges, ctx.ref_size);
    if xings.is_empty() {
        return out; // nada se cruza — nada a tecer
    }
    let gaps = dive_gaps(&xings, geoms.len(), spec.swap);
    let gap_len = ctx.ref_size * (spec.gap / 100.0);

    let mut contours: Vec<(Vec<VecVertex>, bool)> = Vec::new();
    for (i, g) in geoms.iter().enumerate() {
        let gap_frac = (gap_len / g.total).min(0.999);
        contours.extend(strands(g, &gaps[i], gap_frac));
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
