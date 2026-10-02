//! ⭐⭐⭐ **A MALHA POR MOSAICOS** (plano 30 §2.4, W6) — quando UMA porta muda, refaz-se só o quadrado
//! que ela toca, e não a região inteira.
//!
//! # Porque existe (medido, `examples/medir_mudanca.rs`)
//!
//! A construção inteira de uma região de `100 × 100 m` com `1 000` obstáculos custa `~70 ms` — quatro
//! quadros —, e metade é a UNIÃO dos obstáculos (`32 ms`), que cresce mais depressa que a cena. Um
//! mosaico de `10 m` com os `17` que lhe tocam custa `0,29 ms`. ⇒ a região parte-se numa grelha fixa
//! (ancorada na origem da grelha inteira, não na região), cada mosaico guarda os seus polígonos e a
//! ASSINATURA do que o construiu, e uma actualização reconstrói só os mosaicos cuja assinatura mudou.
//!
//! # ⚠️⚠️ A costura entre mosaicos é EXACTA por construção, e reparada se não for
//!
//! Dois mosaicos vizinhos têm de concordar ponto a ponto na linha que partilham — senão a malha
//! montada tem uma parede invisível nessa linha. Duas leis o garantem:
//!
//! 1. **O corte é CANÓNICO** ([`corta`]): cada obstáculo recuado é cortado pelo rectângulo do mosaico
//!    ANTES da união, e o ponto onde uma aresta cruza a linha da costura é calculado a partir da
//!    aresta ORIGINAL (nunca de um pedaço já cortado por outra linha), com os extremos por ordem fixa
//!    e arredondamento inteiro ⇒ os dois lados escrevem o MESMO ponto.
//! 2. **A montagem repara as junções em T** ([`monta`]): todo vértice que cai numa linha de costura
//!    entra nas arestas de costura que o atravessam. Um polígono convexo com um ponto colinear a mais
//!    continua convexo (a `ph2d-nav` aceita os `180°`), e a vizinhança casa aresta a aresta.
//!
//! # ⚠️ Determinismo
//!
//! Tudo inteiro na grelha; os mosaicos por ordem da chave (`BTreeMap`); os obstáculos pela ordem de
//! entrada (a do chamador, que é a das entidades). Uma actualização incremental dá a MESMA malha, ao
//! bit, que uma construção a frio da mesma entrada (gate `mosaicos::incremental_e_a_frio_dao_o_mesmo`).

use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};

use clipper2_rust::{FillRule, Path64, Paths64, Point64, difference_64, union_subjects_64};
use ph2d_nav::{NavMesh, V2};

use crate::lattice::{P, SCALE, to_lattice, to_world};
use crate::{Params, Shape, inflate, triangulate};

/// ⭐ **O lado de um mosaico, em metros** — o MEDIDO (plano 30 §14.1): o custo de uma mudança é o
/// dos mosaicos que ela toca mais a montagem, e a montagem cresce com os polígonos a mais nas costuras.
pub const TILE_M: f64 = 10.0;

/// Um mosaico construído: a assinatura do que o construiu e os polígonos, na grelha inteira.
#[derive(Clone, Debug, Default)]
struct Mosaico {
    sig: u64,
    pts: Vec<P>,
    polys: Vec<Vec<u32>>,
}

/// O que a última actualização fez (os gates e a sonda de custo lêem daqui).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TileStats {
    /// Os mosaicos que cobrem a região.
    pub tiles: usize,
    /// Os mosaicos reconstruídos NESTA actualização.
    pub rebuilt: usize,
    /// Os mosaicos cuja triangulação recusou (ficam VAZIOS — o agente diz «sem caminho» ali).
    pub failed: usize,
}

/// ⭐ **A malha andável de UMA região para UM raio, por mosaicos.** Ver o cabeçalho do módulo.
#[derive(Clone, Debug)]
pub struct TiledMesh {
    params: Params,
    /// O lado do mosaico, em unidades da grelha.
    lado: i64,
    /// A assinatura de TUDO o que entrou na última actualização (o atalho de nada mudou).
    sig: Option<u64>,
    mosaicos: BTreeMap<(i64, i64), Mosaico>,
    mesh: NavMesh,
    stats: TileStats,
}

impl TiledMesh {
    /// Uma malha vazia, à espera da primeira [`Self::update`]. `tile_m` é o lado do mosaico em metros
    /// (o produto usa [`TILE_M`]; os gates variam-no).
    #[must_use]
    pub fn new(params: Params, tile_m: f64) -> Self {
        Self {
            params,
            lado: ((tile_m * SCALE).round() as i64).max(1),
            sig: None,
            mosaicos: BTreeMap::new(),
            mesh: vazia(),
            stats: TileStats::default(),
        }
    }

    /// A malha montada.
    #[must_use]
    pub fn mesh(&self) -> &NavMesh {
        &self.mesh
    }

    /// O que a última actualização fez.
    #[must_use]
    pub fn stats(&self) -> TileStats {
        self.stats
    }

    /// ⭐ **Põe a malha em dia** com esta região (convexa) e estes obstáculos. Devolve `true` se a
    /// malha MUDOU (quem a usa esquece os caminhos), `false` se nenhum mosaico mudou — ⚠️ um
    /// obstáculo que mexe FORA da região muda a entrada e não a malha, e não acorda ninguém.
    pub fn update<S: Borrow<Shape>>(&mut self, region: &[V2], obstacles: &[S]) -> bool {
        let r = self.params.agent_radius.max(0.0);
        let anel = inflate::inset_region(region, r);
        let mut h = Fnv::new();
        anel.iter().for_each(|&p| h.p(p));
        let sig_regiao = h.0;
        let assin: Vec<u64> = obstacles.iter().map(|o| assinatura(o.borrow())).collect();
        assin.iter().for_each(|&a| h.u64(a));
        if self.sig == Some(h.0) {
            self.stats.rebuilt = 0;
            return false;
        }
        self.sig = Some(h.0);

        let mut stats = TileStats::default();
        let mut novos: BTreeMap<(i64, i64), Mosaico> = BTreeMap::new();
        if anel.len() >= 3 {
            let (lo, hi) = caixa_de(&anel);
            let (ix, iy) = self.indice(lo.0, lo.1);
            let (jx, jy) = self.indice(hi.0, hi.1);
            // Que mosaicos cada obstáculo toca — pela caixa dele alargada por `2r` (por excesso: um
            // obstáculo posto num mosaico que não alcança é cortado para nada lá). ⚠️ `2r` e não `r`:
            // o canto em esquadria recua até ao limite `2` e o polígono do disco circunscreve-o
            // (`1/cos(π/n)`, `1,41` a `n = 4`) — com `r` um obstáculo ficava fora de um mosaico que
            // ele tapa (medido: `6e-4 m²` de área a mais).
            let mut por_mosaico: BTreeMap<(i64, i64), Vec<usize>> = BTreeMap::new();
            let folga = (2.0 * r * SCALE).ceil() as i64 + 4;
            for (i, o) in obstacles.iter().enumerate() {
                let (a, b) = caixa(o.borrow());
                let (ax, ay) = self.indice(a.0 - folga, a.1 - folga);
                let (bx, by) = self.indice(b.0 + folga, b.1 + folga);
                // Só os mosaicos da REGIÃO (um chão de 1 km não enche a tabela de vazios).
                for x in ax.max(ix)..=bx.min(jx) {
                    for y in ay.max(iy)..=by.min(jy) {
                        por_mosaico.entry((x, y)).or_default().push(i);
                    }
                }
            }
            for x in ix..=jx {
                for y in iy..=jy {
                    let quem = por_mosaico.get(&(x, y)).map_or(&[][..], Vec::as_slice);
                    let mut h = Fnv::new();
                    h.u64(sig_regiao);
                    quem.iter().for_each(|&i| h.u64(assin[i]));
                    let sig = h.0;
                    stats.tiles += 1;
                    let m = match self.mosaicos.remove(&(x, y)) {
                        Some(m) if m.sig == sig => m,
                        _ => {
                            stats.rebuilt += 1;
                            let obs: Vec<&Shape> =
                                quem.iter().map(|&i| obstacles[i].borrow()).collect();
                            let (m, falhou) = self.constroi(sig, &anel, (x, y), &obs);
                            stats.failed += usize::from(falhou);
                            m
                        }
                    };
                    novos.insert((x, y), m);
                }
            }
        }
        // O que sobrou do mapa antigo saiu da região: também muda a malha.
        let mudou = stats.rebuilt > 0 || !self.mosaicos.is_empty();
        self.mosaicos = novos;
        if mudou {
            self.mesh = monta(&self.mosaicos, self.lado);
        }
        self.stats = stats;
        mudou
    }

    /// O mosaico que contém o ponto da grelha `(x, y)`.
    fn indice(&self, x: i64, y: i64) -> (i64, i64) {
        (x.div_euclid(self.lado), y.div_euclid(self.lado))
    }

    /// Constrói um mosaico. `true` na segunda posição = a triangulação recusou (mosaico vazio).
    fn constroi(
        &self,
        sig: u64,
        anel: &[P],
        (x, y): (i64, i64),
        obs: &[&Shape],
    ) -> (Mosaico, bool) {
        let lo = (x * self.lado, y * self.lado);
        let hi = (lo.0 + self.lado, lo.1 + self.lado);
        let parte = corta(anel, lo, hi);
        if parte.len() < 3 {
            return (
                Mosaico {
                    sig,
                    ..Mosaico::default()
                },
                false,
            );
        }
        let n = self.params.disk_sides.max(4).next_power_of_two();
        let r = self.params.agent_radius.max(0.0);
        let buracos: Paths64 = obs
            .iter()
            .map(|o| corta(&inflate::inflate(o, r, self.params.corner, n), lo, hi))
            .filter(|ring| ring.len() >= 3)
            .map(|ring| caminho(&ring))
            .collect();
        let anda: Paths64 = if buracos.is_empty() {
            vec![caminho(&parte)]
        } else {
            let unidos = union_subjects_64(&buracos, FillRule::NonZero);
            difference_64(&vec![caminho(&parte)], &unidos, FillRule::NonZero)
        };
        let aneis: Vec<Vec<P>> = anda
            .iter()
            .map(|p| p.iter().map(|q| (q.x, q.y)).collect())
            .collect();
        match triangulate::triangulate(&aneis) {
            Ok((pts, tris)) => {
                let polys = if self.params.merge {
                    triangulate::merge_convex(&pts, &tris)
                } else {
                    tris.iter().map(|t| t.to_vec()).collect()
                };
                (Mosaico { sig, pts, polys }, false)
            }
            Err(_) => (
                Mosaico {
                    sig,
                    ..Mosaico::default()
                },
                true,
            ),
        }
    }
}

/// ⭐⭐ **A montagem**: os vértices de todos os mosaicos fundidos por posição na grelha, as junções em
/// T das costuras reparadas, e a [`NavMesh`] pela porta única dela.
fn monta(mosaicos: &BTreeMap<(i64, i64), Mosaico>, lado: i64) -> NavMesh {
    // ⚠️ Só um vértice NUMA LINHA DE COSTURA (`x = k·lado` ou `y = k·lado`) pode ser de dois
    // mosaicos: os outros entram direitos, e só estes passam pelo índice (medido: a montagem é a
    // maior parte de uma mudança, e o índice de todos os vértices era um terço dela).
    let mut indice: BTreeMap<P, u32> = BTreeMap::new();
    let mut verticais: BTreeMap<i64, BTreeSet<i64>> = BTreeMap::new();
    let mut horizontais: BTreeMap<i64, BTreeSet<i64>> = BTreeMap::new();
    let mut pts: Vec<P> = Vec::new();
    let mut polys: Vec<Vec<u32>> = Vec::new();
    for m in mosaicos.values() {
        let mapa: Vec<u32> = m
            .pts
            .iter()
            .map(|&p| {
                let (vx, hy) = (p.0.rem_euclid(lado) == 0, p.1.rem_euclid(lado) == 0);
                if !(vx || hy) {
                    pts.push(p);
                    return (pts.len() - 1) as u32;
                }
                *indice.entry(p).or_insert_with(|| {
                    if vx {
                        verticais.entry(p.0).or_default().insert(p.1);
                    }
                    if hy {
                        horizontais.entry(p.1).or_default().insert(p.0);
                    }
                    pts.push(p);
                    (pts.len() - 1) as u32
                })
            })
            .collect();
        polys.extend(
            m.polys
                .iter()
                .map(|p| p.iter().map(|&v| mapa[v as usize]).collect()),
        );
    }
    let polys: Vec<Vec<u32>> = polys
        .into_iter()
        .map(|p| {
            let n = p.len();
            let mut out = Vec::with_capacity(n + 2);
            for i in 0..n {
                let (a, b) = (pts[p[i] as usize], pts[p[(i + 1) % n] as usize]);
                out.push(p[i]);
                let meio: Vec<P> = if a.0 == b.0 && a.0.rem_euclid(lado) == 0 {
                    entre(verticais.get(&a.0), a.1, b.1)
                        .map(|y| (a.0, y))
                        .collect()
                } else if a.1 == b.1 && a.1.rem_euclid(lado) == 0 {
                    entre(horizontais.get(&a.1), a.0, b.0)
                        .map(|x| (x, a.1))
                        .collect()
                } else {
                    Vec::new()
                };
                out.extend(meio.iter().map(|q| indice[q]));
            }
            out
        })
        .collect();
    let verts: Vec<V2> = pts.iter().map(|&p| to_world(p)).collect();
    NavMesh::from_polygons(verts, polys).unwrap_or_else(|_| vazia())
}

/// Os valores de `linha` estritamente entre `de` e `para`, pela ordem de `de` para `para`.
fn entre(linha: Option<&BTreeSet<i64>>, de: i64, para: i64) -> Box<dyn Iterator<Item = i64> + '_> {
    let Some(l) = linha else {
        return Box::new(std::iter::empty());
    };
    if de < para {
        Box::new(l.range(de + 1..para).copied())
    } else if para < de {
        Box::new(l.range(para + 1..de).rev().copied())
    } else {
        Box::new(std::iter::empty())
    }
}

/// Sobre que recta está uma aresta do polígono que se corta: a de uma aresta ORIGINAL (os extremos
/// dela) ou a de uma linha de corte (eixo — horizontal ou vertical, logo os cruzamentos são exactos).
#[derive(Clone, Copy)]
enum Apoio {
    Original(P, P),
    Eixo,
}

/// ⭐⭐ **O corte CANÓNICO de um polígono convexo pelo rectângulo `[lo, hi]`** (Sutherland–Hodgman),
/// onde todo ponto de cruzamento sai da aresta ORIGINAL que o contém. Ver o cabeçalho (lei 1).
fn corta(poly: &[P], lo: P, hi: P) -> Vec<P> {
    let n = poly.len();
    let mut v: Vec<P> = poly.to_vec();
    let mut e: Vec<Apoio> = (0..n)
        .map(|i| Apoio::Original(poly[i], poly[(i + 1) % n]))
        .collect();
    for (eixo, c, maior) in [
        (0, lo.0, true),
        (0, hi.0, false),
        (1, lo.1, true),
        (1, hi.1, false),
    ] {
        let dentro = |p: P| {
            let q = if eixo == 0 { p.0 } else { p.1 };
            if maior { q >= c } else { q <= c }
        };
        let m = v.len();
        let mut ov: Vec<P> = Vec::with_capacity(m + 2);
        let mut oe: Vec<Apoio> = Vec::with_capacity(m + 2);
        for i in 0..m {
            let (a, b, s) = (v[i], v[(i + 1) % m], e[i]);
            match (dentro(a), dentro(b)) {
                (true, true) => {
                    ov.push(a);
                    oe.push(s);
                }
                (true, false) => {
                    ov.push(a);
                    oe.push(s);
                    ov.push(cruza(s, a, b, eixo, c));
                    oe.push(Apoio::Eixo);
                }
                (false, true) => {
                    ov.push(cruza(s, a, b, eixo, c));
                    oe.push(s);
                }
                (false, false) => {}
            }
        }
        v = ov;
        e = oe;
        if v.is_empty() {
            return v;
        }
    }
    v.dedup();
    if v.len() > 1 && v.first() == v.last() {
        v.pop();
    }
    v
}

/// Onde a aresta `a → b` (sobre `s`) cruza a recta `eixo = c` — da aresta ORIGINAL, com os extremos
/// por ordem fixa e o arredondamento inteiro ao mais perto (metade para longe do zero).
fn cruza(s: Apoio, a: P, b: P, eixo: u8, c: i64) -> P {
    let (p, q) = match s {
        Apoio::Original(p, q) => (p, q),
        Apoio::Eixo => (a, b),
    };
    let (p, q) = if p <= q { (p, q) } else { (q, p) };
    let (pu, pv, qu, qv) = if eixo == 0 {
        (p.0, p.1, q.0, q.1)
    } else {
        (p.1, p.0, q.1, q.0)
    };
    let den = i128::from(qu - pu);
    let w = if den == 0 {
        pv
    } else {
        pv + divide_ao_mais_perto(i128::from(qv - pv) * i128::from(c - pu), den) as i64
    };
    if eixo == 0 { (c, w) } else { (w, c) }
}

/// `n / d` arredondado ao inteiro mais perto, metade para longe do zero.
fn divide_ao_mais_perto(n: i128, d: i128) -> i128 {
    let (n, d) = if d < 0 { (-n, -d) } else { (n, d) };
    if n >= 0 {
        (2 * n + d) / (2 * d)
    } else {
        -((-2 * n + d) / (2 * d))
    }
}

/// A caixa de um obstáculo na grelha (antes do recuo).
fn caixa(o: &Shape) -> (P, P) {
    let pontos: Vec<V2> = match o {
        Shape::Convex(p) => p.clone(),
        Shape::Circle { center, radius } => vec![
            [center[0] - radius, center[1] - radius],
            [center[0] + radius, center[1] + radius],
        ],
        Shape::Capsule { a, b, radius } => vec![
            [a[0] - radius, a[1] - radius],
            [a[0] + radius, a[1] + radius],
            [b[0] - radius, b[1] - radius],
            [b[0] + radius, b[1] + radius],
        ],
    };
    caixa_de(&pontos.into_iter().map(to_lattice).collect::<Vec<_>>())
}

fn caixa_de(pts: &[P]) -> (P, P) {
    pts.iter().fold(
        ((i64::MAX, i64::MAX), (i64::MIN, i64::MIN)),
        |(lo, hi), &(x, y)| ((lo.0.min(x), lo.1.min(y)), (hi.0.max(x), hi.1.max(y))),
    )
}

/// A assinatura de um obstáculo: a forma e os bits de cada número.
fn assinatura(o: &Shape) -> u64 {
    let mut h = Fnv::new();
    match o {
        Shape::Convex(p) => {
            h.byte(0);
            p.iter().flatten().for_each(|&c| h.u64(c.to_bits()));
        }
        Shape::Circle { center, radius } => {
            h.byte(1);
            [center[0], center[1], *radius]
                .iter()
                .for_each(|&c| h.u64(c.to_bits()));
        }
        Shape::Capsule { a, b, radius } => {
            h.byte(2);
            [a[0], a[1], b[0], b[1], *radius]
                .iter()
                .for_each(|&c| h.u64(c.to_bits()));
        }
    }
    h.0
}

fn caminho(ring: &[P]) -> Path64 {
    ring.iter().map(|&(x, y)| Point64::new(x, y)).collect()
}

fn vazia() -> NavMesh {
    NavMesh::from_polygons(Vec::new(), Vec::new())
        .unwrap_or_else(|_| unreachable!("uma malha sem polígonos é sempre válida"))
}

/// FNV-1a de 64 bits — a assinatura só precisa de distinguir.
struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
    fn byte(&mut self, b: u8) {
        self.0 ^= u64::from(b);
        self.0 = self.0.wrapping_mul(0x0100_0000_01b3);
    }
    fn u64(&mut self, v: u64) {
        v.to_le_bytes().into_iter().for_each(|b| self.byte(b));
    }
    fn p(&mut self, p: P) {
        self.u64(p.0 as u64);
        self.u64(p.1 as u64);
    }
}

#[cfg(test)]
#[path = "tiles_tests.rs"]
mod tests;
