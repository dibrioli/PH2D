//! **FASE 2 — os NÓS.** Com a entrada saneada, o predicado exacto decide **sem
//! ambiguidade** onde cada ponto de grade inteira intersecta a malha
//! parametrizada, e a varredura é literalmente três laços.
//!
//! | espécie | de onde vem | como se acha |
//! |---|---|---|
//! | **de vértice** | coincide com um vértice de entrada | a imagem do vértice **é** inteira |
//! | **de aresta** | cai sobre uma aresta, e não é o caso acima | os pontos inteiros no interior do segmento |
//! | **de face** | cai no interior de um triângulo | os pontos inteiros no interior do triângulo-imagem |
//!
//! ⛔⛔ **NÃO há correspondência 1:1 entre pontos inteiros e nós.** Cartas podem
//! **sobrepor-se** — o mesmo ponto inteiro pode gerar vários nós. Isso é esperado,
//! e quem trata é a fusão ([`crate::cells`]), pelas **coordenadas locais** dentro de
//! uma célula. *Deduplicar aqui, pela coordenada inteira, colaria dois nós que
//! vivem em folhas diferentes da mesma dobra.*
//!
//! ⭐ **A posição em `R³` é a MESMA combinação convexa que localizou o nó no
//! domínio**, aplicada aos cantos — e é por isso que a geometria da superfície
//! sobrevive a um colapso no domínio: o canto lembra-se de onde estava.

use crate::exact::{P, orient, strictly_between};
use crate::fan::{fan_of, seed_corners};
use crate::ingest::Topo;

/// Onde o nó vive na malha parametrizada.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Site {
    /// A classe de vértice.
    Vertex(u32),
    /// A face canónica e o lado dela; a imagem vive na carta dessa face.
    Edge { face: u32, side: u8 },
    /// A face.
    Face { face: u32 },
}

/// Um vértice da malha de saída, ainda por fundir.
#[derive(Clone, Debug)]
pub(crate) struct Node {
    pub site: Site,
    /// A imagem do nó na carta da face canónica do [`Site`].
    pub at: P,
    /// A posição em `R³`.
    pub pos: [f64; 3],
}

/// O que a varredura mediu.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct NodeStats {
    pub vertex: usize,
    pub edge: usize,
    pub face: usize,
    /// ⚠️ Faces cuja imagem tem área **zero** — não geram nó de face, e são saltadas
    /// na emissão de saídas.
    pub degenerate_faces: usize,
    /// Faces cuja imagem tem área **negativa** — dobras. ⭐ Não são um erro: o
    /// método aceita-as e extrai à mesma.
    pub folded_faces: usize,
}

/// ⭐ **A VARREDURA DOS TRÊS LAÇOS.**
pub(crate) fn find_nodes(topo: &Topo) -> (Vec<Node>, NodeStats) {
    let one = topo.one;
    let mut st = NodeStats::default();
    let mut nodes: Vec<Node> = Vec::new();

    // ── 1. NÓS DE VÉRTICE: a imagem do vértice É inteira.
    let seeds = seed_corners(topo);
    for (v, seed) in seeds.iter().enumerate() {
        let Some(seed) = *seed else { continue };
        let at = topo.uv[seed.f()][seed.kk()];
        if at[0].rem_euclid(one) != 0 || at[1].rem_euclid(one) != 0 {
            continue;
        }
        let fan = fan_of(topo, seed);
        let pos = corner_centroid(topo, &fan.corners);
        #[allow(clippy::cast_possible_truncation)]
        nodes.push(Node {
            site: Site::Vertex(v as u32),
            at: topo.uv[fan.corners[0].f()][fan.corners[0].kk()],
            pos,
        });
        st.vertex += 1;
    }

    // ── 2. NÓS DE ARESTA: os pontos inteiros no INTERIOR do segmento.
    //
    // ⚠️ **Um lado canónico por aresta.** A aresta interior é vista por duas faces;
    // enumerá-la duas vezes daria dois nós no mesmo sítio da mesma folha, e a fusão
    // teria de os desfazer sem nunca saber que eram o mesmo.
    for f in 0..topo.tris.len() {
        for k in 0..3usize {
            if !is_canonical(topo, f, k) {
                continue;
            }
            let a = topo.uv[f][k];
            let b = topo.uv[f][(k + 1) % 3];
            for q in lattice_in_box(a, b, a, one) {
                if orient(a, b, q) != 0 || !strictly_between(a, b, q) {
                    continue;
                }
                if by_vertex_hit(topo, f, k, q) {
                    continue;
                }
                let pos = lerp3(topo.p3[f][k], topo.p3[f][(k + 1) % 3], param(a, b, q));
                #[allow(clippy::cast_possible_truncation)]
                nodes.push(Node {
                    site: Site::Edge {
                        face: f as u32,
                        side: k as u8,
                    },
                    at: q,
                    pos,
                });
                st.edge += 1;
            }
        }
    }

    // ── 3. NÓS DE FACE: os pontos inteiros no INTERIOR do triângulo-imagem.
    for f in 0..topo.tris.len() {
        let [a, b, c] = topo.uv[f];
        let s = orient(a, b, c);
        if s == 0 {
            st.degenerate_faces += 1;
            continue;
        }
        if s < 0 {
            st.folded_faces += 1;
        }
        for q in lattice_in_box(a, b, c, one) {
            if orient(a, b, q) != s || orient(b, c, q) != s || orient(c, a, q) != s {
                continue;
            }
            let w = barycentric(a, b, c, q);
            let pos = mix3(topo.p3[f], w);
            #[allow(clippy::cast_possible_truncation)]
            nodes.push(Node {
                site: Site::Face { face: f as u32 },
                at: q,
                pos,
            });
            st.face += 1;
        }
    }

    (nodes, st)
}

/// O lado `(f, k)` é o representante canónico da sua aresta?
pub(crate) fn is_canonical(topo: &Topo, f: usize, k: usize) -> bool {
    match topo.twin[f][k] {
        None => true,
        #[allow(clippy::cast_possible_truncation)]
        Some((g, j)) => (f as u32, k as u8) <= (g, j),
    }
}

/// O ponto `q` é a imagem de um dos dois vértices desta aresta?
fn by_vertex_hit(topo: &Topo, f: usize, k: usize, q: P) -> bool {
    topo.uv[f][k] == q || topo.uv[f][(k + 1) % 3] == q
}

/// Os pontos de grade inteiros na caixa dos três pontos dados.
fn lattice_in_box(a: P, b: P, c: P, one: i64) -> impl Iterator<Item = P> {
    let lo = [a[0].min(b[0]).min(c[0]), a[1].min(b[1]).min(c[1])];
    let hi = [a[0].max(b[0]).max(c[0]), a[1].max(b[1]).max(c[1])];
    let nx = ceil_div(lo[0], one)..=hi[0].div_euclid(one);
    let ny0 = ceil_div(lo[1], one);
    let ny1 = hi[1].div_euclid(one);
    nx.flat_map(move |i| (ny0..=ny1).map(move |j| [i * one, j * one]))
}

fn ceil_div(a: i64, b: i64) -> i64 {
    -((-a).div_euclid(b))
}

/// O parâmetro de `q` no segmento `[a, b]`, em `[0, 1]`.
fn param(a: P, b: P, q: P) -> f64 {
    let (num, den) = if a[0] != b[0] {
        (q[0] - a[0], b[0] - a[0])
    } else {
        (q[1] - a[1], b[1] - a[1])
    };
    if den == 0 {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let v = num as f64 / den as f64;
    v
}

/// Os pesos baricêntricos de `q` no triângulo `(a, b, c)`.
///
/// ⚠️ **A divisão é aqui e só aqui.** Os pesos alimentam uma posição em `R³`, que é
/// geometria e não decisão; nenhuma decisão discreta desta crate os lê.
fn barycentric(a: P, b: P, c: P, q: P) -> [f64; 3] {
    let total = crate::exact::area2(a, b, c);
    if total == 0 {
        return [1.0, 0.0, 0.0];
    }
    #[allow(clippy::cast_precision_loss)]
    let inv = 1.0 / total as f64;
    #[allow(clippy::cast_precision_loss)]
    let wa = crate::exact::area2(q, b, c) as f64 * inv;
    #[allow(clippy::cast_precision_loss)]
    let wb = crate::exact::area2(a, q, c) as f64 * inv;
    [wa, wb, 1.0 - wa - wb]
}

fn lerp3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn mix3(p: [[f64; 3]; 3], w: [f64; 3]) -> [f64; 3] {
    let mut out = [0.0f64; 3];
    for i in 0..3 {
        out[i] = p[0][i].mul_add(w[0], p[1][i].mul_add(w[1], p[2][i] * w[2]));
    }
    out
}

/// A posição de um nó de vértice.
///
/// ⚠️ **Cantos idênticos devolvem o valor idêntico, e não a média deles** — somar
/// `n` cópias do mesmo `f64` e dividir por `n` **não** devolve o original, e o caso
/// de longe mais comum é a classe de um vértice só.
fn corner_centroid(topo: &Topo, corners: &[crate::fan::Corner]) -> [f64; 3] {
    let first = topo.p3[corners[0].f()][corners[0].kk()];
    if corners.iter().all(|c| topo.p3[c.f()][c.kk()] == first) {
        return first;
    }
    let mut acc = [0.0f64; 3];
    for c in corners {
        let p = topo.p3[c.f()][c.kk()];
        for i in 0..3 {
            acc[i] += p[i];
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let n = corners.len() as f64;
    [acc[0] / n, acc[1] / n, acc[2] / n]
}
