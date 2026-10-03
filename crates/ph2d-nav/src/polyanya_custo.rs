//! ⭐⭐ (W7) **A REFRACÇÃO** — o Polyanya quando o custo muda numa aresta (plano 30 §2.5, a medição
//! que abriu a W7: `examples/medir_custo.rs` da `ph2d-navmesh`).
//!
//! # Porque o Polyanya puro não chega, e o que se lhe acrescenta
//!
//! O Polyanya supõe custo UNIFORME: um troço recto da raiz até onde ela vê. Entre áreas de custos
//! diferentes o caminho óptimo continua RECTO dentro de cada área, mas DOBRA na fronteira (a lei de
//! Snell das regiões pesadas, Mitchell & Papadimitriou 1991) — num ponto que nenhum vértice marca.
//! Três movimentos novos, todos numa aresta onde o custo muda:
//!
//! 1. **Atravessar** — um intervalo que chega a ela não continua: nascem RAÍZES nela, no pedaço que a
//!    raiz vê — as duas pontas do intervalo (onde um caminho que roça um canto atravessa) e os pontos
//!    de uma grelha FIXA da aresta, de [`STEINER_M`] em [`STEINER_M`] metros. Cada uma vê o polígono
//!    do outro lado inteiro (convexo); numa ponta que é VÉRTICE, o leque inteiro.
//! 2. **Deslizar** — andar EM CIMA da aresta custa o menor dos dois lados; de uma raiz nela, os outros
//!    pontos da grelha nascem a esse custo, e emitem para o lado de TRÁS quando ele é o caro. É o que
//!    faz o óptimo dentro da lama: sair, correr encostado à fronteira, e voltar a entrar (medido na
//!    cena `custo::dentro_da_lama…`). Num vértice, ao longo de cada aresta de fronteira que lhe toca.
//! 3. **Dobrar** no vértice de uma fronteira — é a volta num canto, com o vértice marcado como canto
//!    quando o custo muda à volta dele (`Polyanya::is_corner`).
//!
//! No fim, o **polimento de Snell** desliza cada raiz de fronteira no seu pedaço até ao mínimo de
//! `w₁·|x − P| + w₂·|N − x|` (convexo na posição ⇒ bissecção na derivada), aceitando só o que o custo
//! REAL (a caminhada de [`crate::cost::segment_cost`]) confirma — a grelha escolhe o corredor, o
//! polimento tira-lhe o erro da grelha.
//!
//! # ⭐ A poda: uma raiz por PONTO e por LADO para onde emite
//!
//! Uma raiz de fronteira emite um polígono INTEIRO (não depende de onde se veio), logo duas chegadas
//! ao mesmo ponto para o mesmo lado são o mesmo nó: fica a mais barata, e o empate também se poda
//! (medido: com `<` estrito, 35 milhões de nós numa cena de 200 polígonos). Uma ponta de intervalo
//! (fora da grelha) é inútil se um ponto da grelha vizinho `y` tem `g_y + w·|x − y| ≤ g` com `w` o
//! custo do LADO para onde ela emite. ⚠️ Com o menor dos dois lados a regra era falsa para o lado caro
//! — exactamente o caso do deslize.
//!
//! ⚠️ Onde o custo não muda nada disto corre: sem áreas é o Polyanya de sempre, AO BIT (gate).

use super::{Kind, Node, Polyanya, Root};
use crate::cost::segment_cost;
use crate::geom::{EPS, V2, dist, dot, lerp, same, sub};
use crate::mesh::NavMesh;

/// O passo da grelha de raízes numa fronteira de custo, em metros. O recurso é o TEMPO por consulta
/// (medido, `medir_custo` §3, 8 cenas `30 × 20` m com 4 lamas, custo / oráculo · µs):
///
/// | passo | peso 2: média · máx · µs | peso 10: média · máx · µs |
/// |---|---|---|
/// | `0,5` | `1,0001 · 1,0107 · 74` | `1,0005 · 1,0938 · 245` |
/// | **`0,25`** | `1,0000 · 1,0107 · 100` | `0,9997 · 1,0045 · 351` |
/// | `0,1` | `0,9999 · 1,0000 · 218` | `0,9997 · 1,0000 · 803` |
///
/// ⇒ `0,25`: o mesmo caminho em média, ~1 % no pior, a 2,3× menos que `0,1` (a `0,5` a grelha já
/// escolhe o corredor errado: 9 %). Abaixo de `1,0` é o oráculo a errar (`0,2 %` residual).
pub const STEINER_M: f64 = 0.25;

/// O tecto das varridas do polimento (cada varrida desliza cada raiz uma vez; pára antes quando
/// nenhuma se mexe mais de [`EPS`]).
const POLISH_SWEEPS: usize = 32;

/// Uma aresta de fronteira de custo, pelos vértices em ordem CANÓNICA (o menor índice primeiro): a
/// grelha é a mesma para quem chegar de qualquer lado.
#[derive(Clone, Copy, Debug)]
struct Aresta {
    lo: u32,
    hi: u32,
    plo: V2,
    phi: V2,
    len: f64,
    kk: u32,
}

impl Aresta {
    fn new(mesh: &NavMesh, a: u32, b: u32, passo: f64) -> Option<Self> {
        let (lo, hi) = (a.min(b), a.max(b));
        let (plo, phi) = (mesh.vert(lo), mesh.vert(hi));
        let len = dist(plo, phi);
        (len > EPS).then(|| Aresta {
            lo,
            hi,
            plo,
            phi,
            len,
            kk: (len / passo).ceil().max(1.0) as u32,
        })
    }

    fn param(&self, x: V2) -> f64 {
        dot(sub(x, self.plo), sub(self.phi, self.plo)) / (self.len * self.len)
    }

    fn ponto(&self, j: u32) -> V2 {
        lerp(self.plo, self.phi, f64::from(j) / f64::from(self.kk))
    }

    /// O vértice da malha em `x`, se `x` é uma das pontas.
    fn vertice(&self, mesh: &NavMesh, x: V2) -> Option<u32> {
        [self.lo, self.hi]
            .into_iter()
            .find(|&v| same(x, mesh.vert(v)))
    }
}

impl Polyanya {
    /// **Atravessar**: o intervalo `[left, right]` (na aresta de entrada `entry` de `P`) chegou a uma
    /// área de custo diferente de `w` — raízes nas pontas e na grelha da aresta.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn refract(
        &mut self,
        mesh: &NavMesh,
        root: u32,
        p: u32,
        entry: u32,
        left: V2,
        right: V2,
        w: f64,
        t: V2,
    ) {
        let poly = &mesh.polys()[p as usize];
        let n = poly.len();
        let k = entry as usize;
        let (va, vb) = (poly.verts[k], poly.verts[(k + 1) % n]);
        let Some(de) = poly.nbrs[k] else { return };
        let Some(ar) = Aresta::new(mesh, va, vb, self.steiner_m()) else {
            return;
        };
        let (tl, tr) = (ar.param(left), ar.param(right));
        let (t0, t1) = (tl.min(tr), tl.max(tr));
        let tol = EPS / ar.len;
        let mut pontos: Vec<(V2, Option<u32>)> = Vec::new();
        let j0 = (t0 * f64::from(ar.kk) - tol).ceil().max(0.0) as u32;
        let j1 = (t1 * f64::from(ar.kk) + tol).floor().min(f64::from(ar.kk)) as u32;
        for j in j0..=j1 {
            pontos.push((ar.ponto(j), Some(j)));
        }
        for x in [left, right] {
            if !pontos.iter().any(|&(q, _)| same(q, x)) {
                pontos.push((x, None));
            }
        }
        let r = self.roots[root as usize];
        for (x, j) in pontos {
            let g = r.g + w * dist(r.p, x);
            let lado = Lado {
                into: p,
                other: de,
                range: (left, right),
            };
            self.boundary_root(mesh, root, &ar, x, j, g, w, lado, true, t);
        }
    }

    /// Uma raiz de fronteira em `x` (na aresta `ar`), a emitir para `lado.into` — ou, num vértice,
    /// para o leque inteiro. Ver o cabeçalho (a poda, e o deslize quando `slide`).
    #[allow(clippy::too_many_arguments)]
    fn boundary_root(
        &mut self,
        mesh: &NavMesh,
        prev: u32,
        ar: &Aresta,
        x: V2,
        j: Option<u32>,
        g: f64,
        w_in: f64,
        lado: Lado,
        slide: bool,
        t: V2,
    ) {
        if let Some(v) = ar.vertice(mesh, x) {
            self.vertex_root(mesh, prev, v, g, w_in, lado.range, t);
            return;
        }
        let into = lado.into;
        let w_into = self.cost(mesh, into);
        let tx = ar.param(x);
        match j {
            Some(j) => {
                let best = self
                    .steiner_g
                    .entry((ar.lo, ar.hi, into, j))
                    .or_insert(f64::INFINITY);
                if *best <= g + EPS {
                    self.stats.pruned_refractions += 1;
                    return;
                }
                *best = g;
            }
            None => {
                let jf = (tx * f64::from(ar.kk)).floor().max(0.0) as u32;
                let vizinho = [jf, (jf + 1).min(ar.kk)].into_iter().any(|jj| {
                    self.steiner_g
                        .get(&(ar.lo, ar.hi, into, jj))
                        .is_some_and(|&gj| gj + w_into * dist(ar.ponto(jj), x) <= g + EPS)
                });
                let lista = self.ponta_g.entry((ar.lo, ar.hi, into)).or_default();
                if vizinho
                    || lista
                        .iter()
                        .any(|&(tp, gp)| gp + w_into * (tp - tx).abs() * ar.len <= g + EPS)
                {
                    self.stats.pruned_refractions += 1;
                    return;
                }
                lista.push((tx, g));
            }
        }
        self.promete(
            Pendente {
                prev,
                x,
                j,
                g,
                w_in,
                lado,
                ar: *ar,
                slide,
                vertice: None,
            },
            t,
        );
    }

    /// ⭐ A raiz entra no heap como PROMESSA, com o limite inferior `g + w_min·|x − t|` — só gera os
    /// filhos quando sai, e as que ficam para lá do custo da resposta nunca custam nada.
    fn promete(&mut self, p: Pendente, t: V2) {
        let f = p.g + self.wmin * dist(p.x, t);
        let idx = self.pendentes.len() as u32;
        self.pendentes.push(p);
        self.push(
            f,
            Node {
                root: p.prev,
                kind: Kind::Pending { idx },
                w: p.w_in,
            },
        );
    }

    /// A promessa saiu do heap: se entretanto não chegou uma MAIS barata ao mesmo sítio, nasce a
    /// raiz, os filhos dela e o deslize.
    pub(super) fn materialize(&mut self, mesh: &NavMesh, idx: u32, t: V2) {
        let p = self.pendentes[idx as usize];
        if let Some(v) = p.vertice {
            if self.fan_g[v as usize] < p.g - EPS {
                return;
            }
            self.vertex_root_now(mesh, p, v, t);
            return;
        }
        if let Some(j) = p.j
            && self
                .steiner_g
                .get(&(p.ar.lo, p.ar.hi, p.lado.into, j))
                .is_some_and(|&b| b < p.g - EPS)
        {
            return;
        }
        let (x, g, ar, lado, into) = (p.x, p.g, p.ar, p.lado, p.lado.into);
        let w_into = self.cost(mesh, into);
        self.stats.refractions += 1;
        let ri = self.roots.len() as u32;
        self.roots.push(Root {
            p: x,
            g,
            prev: p.prev,
            w_in: p.w_in,
            range: Some(lado.range),
        });
        self.push_from_point(mesh, ri, into, None, t);
        let slide = p.slide;
        // O deslize só para TRÁS e só se lá é mais caro: para a frente, os pontos que a raiz anterior
        // vê já nasceram dela mais baratos (desigualdade triangular) e os outros alcançam-se dobrando
        // nos cantos do lado barato; para trás, a procura do lado de cá nunca vê a aresta onde a raiz
        // está (é colinear) — é o «sair e voltar a entrar» da lama.
        let w_outro = self.cost(mesh, lado.other);
        if slide && w_outro > w_into {
            self.slide(mesh, ri, &ar, x, g, w_into, lado.other, into, t);
        }
    }

    /// Uma raiz num VÉRTICE de fronteira: o leque inteiro (uma vez por vértice, a mais barata), e o
    /// deslize ao longo de cada aresta de fronteira que lhe toca — é o que contorna uma área
    /// arredondada encostado a ela.
    #[allow(clippy::too_many_arguments)]
    fn vertex_root(
        &mut self,
        mesh: &NavMesh,
        prev: u32,
        v: u32,
        g: f64,
        w_in: f64,
        range: (V2, V2),
        t: V2,
    ) {
        if self.fan_g[v as usize] <= g + EPS {
            self.stats.pruned_refractions += 1;
            return;
        }
        if self.fan_g[v as usize].is_infinite() && self.root_g[v as usize].is_infinite() {
            self.touched.push(v);
        }
        self.fan_g[v as usize] = g;
        let pv = mesh.vert(v);
        self.promete(
            Pendente {
                prev,
                x: pv,
                j: None,
                g,
                w_in,
                lado: Lado {
                    into: 0,
                    other: 0,
                    range,
                },
                ar: Aresta {
                    lo: v,
                    hi: v,
                    plo: pv,
                    phi: pv,
                    len: 0.0,
                    kk: 1,
                },
                slide: false,
                vertice: Some(v),
            },
            t,
        );
    }

    /// A raiz-vértice prometida nasce: o leque, e o deslize nas arestas de fronteira dela.
    fn vertex_root_now(&mut self, mesh: &NavMesh, p: Pendente, v: u32, t: V2) {
        let (prev, g, w_in, range) = (p.prev, p.g, p.w_in, p.lado.range);
        self.stats.refractions += 1;
        let ri = self.roots.len() as u32;
        let pv = mesh.vert(v);
        self.roots.push(Root {
            p: pv,
            g,
            prev,
            w_in,
            range: Some(range),
        });
        for &q in mesh.polys_at_vertex(v) {
            self.push_from_point(mesh, ri, q, Some(v), t);
        }
        // As arestas de fronteira que tocam `v` (cada uma vista dos dois polígonos: uma vez).
        let mut arestas: Vec<(u32, u32, u32)> = Vec::new();
        for &q in mesh.polys_at_vertex(v) {
            let poly = &mesh.polys()[q as usize];
            let n = poly.len();
            for i in 0..n {
                let (a, b) = (poly.verts[i], poly.verts[(i + 1) % n]);
                let Some(nb) = poly.nbrs[i] else { continue };
                let o = if a == v { b } else { a };
                if (a == v || b == v)
                    && self.cost(mesh, q) != self.cost(mesh, nb)
                    && !arestas.iter().any(|&(x, _, _)| x == o)
                {
                    arestas.push((o, q, nb));
                }
            }
        }
        for (o, q, nb) in arestas {
            let Some(ar) = Aresta::new(mesh, v, o, self.steiner_m()) else {
                continue;
            };
            let (cq, cn) = (self.cost(mesh, q), self.cost(mesh, nb));
            let (caro, barato) = if cq > cn { (q, nb) } else { (nb, q) };
            self.slide(mesh, ri, &ar, pv, g, cq.min(cn), caro, barato, t);
        }
    }

    /// **Deslizar**: de uma raiz em `x` (custo `g`) ao longo da aresta `ar` a `c_e` por metro, cada
    /// ponto da grelha (as pontas incluídas) a emitir para o lado caro `caro`.
    #[allow(clippy::too_many_arguments)]
    fn slide(
        &mut self,
        mesh: &NavMesh,
        ri: u32,
        ar: &Aresta,
        x: V2,
        g: f64,
        c_e: f64,
        caro: u32,
        barato: u32,
        t: V2,
    ) {
        let lado = Lado {
            into: caro,
            other: barato,
            range: (ar.plo, ar.phi),
        };
        for j in 0..=ar.kk {
            let y = ar.ponto(j);
            if !same(x, y) {
                let gy = g + c_e * dist(x, y);
                self.boundary_root(mesh, ri, ar, y, Some(j), gy, c_e, lado, false, t);
            }
        }
    }

    /// O passo da grelha desta procura (o produto usa [`STEINER_M`]; a sonda varia-o).
    fn steiner_m(&self) -> f64 {
        self.steiner_override.unwrap_or(STEINER_M)
    }

    /// A sonda e os gates escolhem outro passo da grelha (`None` volta ao do produto).
    pub fn set_steiner_spacing(&mut self, m: Option<f64>) {
        self.steiner_override = m.filter(|&m| m > EPS);
    }
}

/// Uma raiz de fronteira à espera de sair do heap (ver `Polyanya::promete`).
#[derive(Clone, Copy, Debug)]
pub(super) struct Pendente {
    prev: u32,
    x: V2,
    j: Option<u32>,
    g: f64,
    w_in: f64,
    lado: Lado,
    ar: Aresta,
    slide: bool,
    vertice: Option<u32>,
}

/// Para onde uma raiz de fronteira emite (`into`), o lado de lá da aresta (`other`), e onde ela pode
/// deslizar no polimento (`range`).
#[derive(Clone, Copy, Debug)]
struct Lado {
    into: u32,
    other: u32,
    range: (V2, V2),
}

/// O polimento de Snell sobre os pontos `(ponto, custo do troço que chega, gama de deslize)`. Só se
/// mexem as raízes de fronteira; a partida, o alvo e os cantos ficam.
pub(super) fn polish(mesh: &NavMesh, costs: &[f64], pts: &mut [(V2, f64, Option<(V2, V2)>)]) {
    let real = |a: V2, b: V2, c: V2| -> Option<f64> {
        Some(segment_cost(mesh, costs, a, b)? + segment_cost(mesh, costs, b, c)?)
    };
    for _ in 0..POLISH_SWEEPS {
        let mut mexeu = false;
        for i in 1..pts.len().saturating_sub(1) {
            let Some((l, r)) = pts[i].2 else { continue };
            let (p, x, nx) = (pts[i - 1].0, pts[i].0, pts[i + 1].0);
            let (w1, w2) = (pts[i].1, pts[i + 1].1);
            let novo = snell(p, nx, l, r, w1, w2);
            if dist(novo, x) <= EPS {
                continue;
            }
            let (Some(antes), Some(depois)) = (real(p, x, nx), real(p, novo, nx)) else {
                continue;
            };
            if depois < antes {
                pts[i].0 = novo;
                mexeu = true;
            }
        }
        if !mexeu {
            break;
        }
    }
}

/// O ponto de `[l, r]` que minimiza `w1·|x − p| + w2·|n − x|` — convexo na posição, logo a derivada
/// muda de sinal uma vez: bissecção (60 passos, abaixo do `f64` de qualquer aresta da malha).
fn snell(p: V2, n: V2, l: V2, r: V2, w1: f64, w2: f64) -> V2 {
    let d = sub(r, l);
    let deriv = |tau: f64| {
        let x = lerp(l, r, tau);
        let (a, b) = (sub(x, p), sub(x, n));
        let (la, lb) = (dot(a, a).sqrt().max(EPS), dot(b, b).sqrt().max(EPS));
        w1 * dot(a, d) / la + w2 * dot(b, d) / lb
    };
    if deriv(0.0) >= 0.0 {
        return l;
    }
    if deriv(1.0) <= 0.0 {
        return r;
    }
    let (mut a, mut b) = (0.0, 1.0);
    for _ in 0..60 {
        let m = 0.5 * (a + b);
        if deriv(m) < 0.0 {
            a = m;
        } else {
            b = m;
        }
    }
    lerp(l, r, 0.5 * (a + b))
}
