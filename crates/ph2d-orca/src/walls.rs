//! **As paredes** como o ORCA as lê: cadeias FECHADAS de arestas, com o espaço livre à DIREITA de cada
//! uma (a convenção do artigo: um obstáculo é um polígono anti-horário e o agente anda fora dele).
//!
//! Cada entrada `i` é um vértice e a aresta que sai dele para [`Walls::next`]. Um vértice é
//! CONVEXO quando a cadeia vira à esquerda nele (visto do espaço livre, é uma quina que se contorna);
//! num CÔNCAVO (o canto de uma sala) a perna do cone prolonga a parede.

use crate::v2::{V2, abs_sq, dist_sq_to_segment, left_of, normalize, sub};

#[derive(Clone, Debug, Default)]
pub struct Walls {
    point: Vec<V2>,
    next: Vec<u32>,
    prev: Vec<u32>,
    dir: Vec<V2>,
    convex: Vec<bool>,
}

impl Walls {
    /// Polígonos-OBSTÁCULO, cada um anti-horário (área com sinal positiva): o agente anda fora deles.
    /// Um polígono de dois pontos é um segmento solto. ⚠️ Um polígono horário prende o agente DENTRO
    /// dele — medido no Godot, que tem a mesma convenção (a 1.ª corrida do oráculo do desvio).
    #[must_use]
    pub fn from_polygons(polys: &[Vec<V2>]) -> Self {
        let mut w = Self::default();
        for poly in polys {
            let n = poly.len();
            if n < 2 {
                continue;
            }
            let base = w.point.len() as u32;
            for (k, &p) in poly.iter().enumerate() {
                w.point.push(p);
                w.next.push(base + ((k + 1) % n) as u32);
                w.prev.push(base + ((k + n - 1) % n) as u32);
            }
        }
        w.finish();
        w
    }

    /// ⭐ **As paredes de uma MALHA ANDÁVEL**: `walls` são as arestas `(de, para)` da fronteira com o
    /// espaço ANDÁVEL à esquerda (o sentido dos polígonos da malha). Invertidas, ficam com ele à
    /// direita — a convenção do ORCA.
    ///
    /// ⚠️ Num vértice onde a fronteira se TOCA a si própria (duas quinas encostadas) há duas
    /// continuações; fica a de menor índice — a ordem é a da malha, igual nos três sistemas.
    #[must_use]
    pub fn from_walkable_walls(verts: &[V2], walls: &[(u32, u32)]) -> Self {
        // A aresta invertida `para → de` é a entrada `k`, no ponto `para`.
        let n = walls.len();
        let mut w = Self::default();
        let mut starting_at: std::collections::BTreeMap<u32, u32> =
            std::collections::BTreeMap::new();
        let mut ending_at: std::collections::BTreeMap<u32, u32> = std::collections::BTreeMap::new();
        for (k, &(de, para)) in walls.iter().enumerate() {
            w.point.push(verts[para as usize]);
            starting_at.entry(para).or_insert(k as u32);
            ending_at.entry(de).or_insert(k as u32);
        }
        for &(de, para) in walls {
            // A entrada `k` vai de `para` a `de`; a seguinte começa em `de`, a anterior acaba em `para`.
            let k = w.next.len() as u32;
            w.next.push(starting_at.get(&de).copied().unwrap_or(k));
            w.prev.push(ending_at.get(&para).copied().unwrap_or(k));
        }
        debug_assert_eq!(w.next.len(), n);
        w.finish();
        w
    }

    fn finish(&mut self) {
        let n = self.point.len();
        self.dir = (0..n)
            .map(|i| normalize(sub(self.point[self.next[i] as usize], self.point[i])))
            .collect();
        self.convex = (0..n)
            .map(|i| {
                let p = self.prev[i] as usize;
                let q = self.next[i] as usize;
                // Um segmento solto (dois pontos) tem as duas pontas convexas.
                p == q || left_of(self.point[p], self.point[i], self.point[q]) >= 0.0
            })
            .collect();
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.point.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.point.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn point(&self, i: usize) -> V2 {
        self.point[i]
    }

    #[inline]
    #[must_use]
    pub fn next(&self, i: usize) -> usize {
        self.next[i] as usize
    }

    #[inline]
    #[must_use]
    pub fn prev(&self, i: usize) -> usize {
        self.prev[i] as usize
    }

    /// A direcção unitária da aresta que sai de `i`.
    #[inline]
    #[must_use]
    pub fn dir(&self, i: usize) -> V2 {
        self.dir[i]
    }

    #[inline]
    #[must_use]
    pub fn convex(&self, i: usize) -> bool {
        self.convex[i]
    }

    /// **As paredes que contam para um agente em `pos`**: as que olham para ele (ele está do lado
    /// LIVRE) e estão a menos de `range` — pela ordem da distância, e do índice num empate. A ordem
    /// importa: uma parede cujo cone já está coberto pelas anteriores não dá semi-plano.
    pub fn near(&self, pos: V2, range: f64, out: &mut Vec<u32>) {
        out.clear();
        let range_sq = range * range;
        let mut found: Vec<(f64, u32)> = Vec::new();
        for i in 0..self.point.len() {
            let a = self.point[i];
            let b = self.point[self.next[i] as usize];
            if abs_sq(sub(b, a)) <= 0.0 || left_of(a, b, pos) >= 0.0 {
                continue;
            }
            let d = dist_sq_to_segment(a, b, pos);
            if d < range_sq {
                found.push((d, i as u32));
            }
        }
        found.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)));
        out.extend(found.into_iter().map(|(_, i)| i));
    }
}
