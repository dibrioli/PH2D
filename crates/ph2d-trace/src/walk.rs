//! **O PASSEIO QUE SEGUE O CAMPO** — de onde vêm as paredes.
//!
//! # A regra de um passo
//!
//! Estando num vértice `v` com uma direção `d`, escolhe-se o vizinho cuja aresta
//! melhor se alinha com `d`, anda-se, e **a direção é reencostada à cruz do
//! vértice novo**. Repete até bater em algo.
//!
//! ⚠️ **O reencosto é o passo que faz disto um traçado de CAMPO** e não uma reta.
//! Sem ele o passeio segue a direção inicial e atravessa a curvatura da forma;
//! com ele, ele curva junto com a superfície — que é o que uma separatriz é.
//!
//! # ⚠️ Por que a conversão LOSSY do F2 serve aqui, e não servia lá
//!
//! [`ph2d_crossfield::to_vertex_dirs`] descarta os saltos de período, e foi
//! exatamente isso que fez o F2 não render quando ligado ao extrator antigo: um
//! extrator **re-deriva estrutura global** a partir das direções médias, e a
//! média de um campo com singularidades reais é pior, para ele, que um campo
//! liso. Aqui não se re-deriva nada: as singularidades **já estão decididas**
//! pelo F2 (é delas que os passeios partem) e o passeio é **local** — ele só
//! precisa de saber para onde continuar no vértice seguinte.
//! *A mesma perda é fatal num sítio e irrelevante no outro.*

use std::collections::{BTreeMap, BTreeSet};

use ph2d_crossfield::{CrossField, Dual, to_vertex_dirs, vertex_index};
use ph2d_mesh::Mesh;

use crate::TraceReport;
use crate::ring::Ring;

/// ⚠️ **O teto de passos de UMA separatriz**, como múltiplo de `sqrt(vértices)`.
///
/// Uma separatriz atravessa a forma, e a travessia de uma malha de `V` vértices
/// é da ordem de `sqrt(V)` arestas. O múltiplo dá folga para ela dar voltas antes
/// de encontrar alguém. ⚠️ **Ele não decide qualidade nenhuma**: quem o atinge é
/// **descartado** e contado em [`TraceReport::dangling`] — o número é para ser
/// lido, não para ser confiado.
const STEPS_PER_SQRT_VERT: f32 = 24.0;

/// O mínimo de arestas de uma separatriz que vale a pena guardar.
///
/// ⚠️ **Uma parede de uma aresta só não separa nada** — ela nasce quando duas
/// separatrizes da mesma singularidade saem quase na mesma direção e a segunda
/// bate na primeira no primeiro passo. Guardá-la só põe um canto falso no meio
/// do lado de um patch.
const MIN_EDGES: usize = 2;

/// **AS PAREDES** — as arestas que o traçado ocupou.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Walls {
    /// As arestas, como pares `(menor, maior)`.
    pub edges: BTreeSet<(u32, u32)>,
    /// Por vértice, quantas arestas de parede o tocam — o **grau no traçado**.
    pub degree: BTreeMap<u32, u32>,
}

impl Walls {
    /// Esta aresta é parede?
    #[must_use]
    pub fn blocks(&self, a: u32, b: u32) -> bool {
        self.edges.contains(&(a.min(b), a.max(b)))
    }

    /// **QUANTAS ARESTAS DE PAREDE tocam cada vértice** — derivado de [`Self::edges`].
    ///
    /// ⭐ **É a RAMIFICAÇÃO do traçado, e não o mesmo que [`Self::degree`].** O
    /// `degree` conta pontas de aresta **por passeio**, então uma aresta que duas
    /// separatrizes partilham é contada duas vezes e um vértice interior aparece
    /// com grau 4. Aqui a fonte é o conjunto de arestas, que é único por
    /// construção — e é isso que torna `> 2` uma pergunta sobre a **estrutura**.
    ///
    /// A leitura: `2` é o interior de uma separatriz · `3` é uma junção em T ou
    /// uma singularidade de índice `+1` · `4` é um cruzamento · `5` é uma
    /// singularidade de índice `−1`.
    #[must_use]
    pub fn branching(&self) -> BTreeMap<u32, usize> {
        let mut out: BTreeMap<u32, usize> = BTreeMap::new();
        for &(a, b) in &self.edges {
            *out.entry(a).or_default() += 1;
            *out.entry(b).or_default() += 1;
        }
        out
    }
}

/// **O PASSEADOR** — segura o que o passeio precisa e nada mais.
#[derive(Debug, Clone)]
pub struct Walker<'a> {
    mesh: &'a Mesh,
    dirs: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    index: Vec<i32>,
    /// Aresta dirigida -> face. É o que permite pivotar em volta de um vértice.
    half: BTreeMap<(u32, u32), u32>,
    max_steps: usize,
}

impl<'a> Walker<'a> {
    /// Prepara o passeio: as direções por vértice e os índices do campo.
    #[must_use]
    pub fn new(mesh: &'a Mesh, dual: &Dual, field: &CrossField) -> Self {
        let verts = mesh.vert_count();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let max_steps = (STEPS_PER_SQRT_VERT * (verts as f32).sqrt()).ceil() as usize + 8;
        let mut half = BTreeMap::new();
        for (fi, f) in mesh.faces().iter().enumerate() {
            let v = f.verts();
            for k in 0..v.len() {
                half.insert(
                    (v[k], v[(k + 1) % v.len()]),
                    u32::try_from(fi).unwrap_or(u32::MAX),
                );
            }
        }
        Self {
            mesh,
            dirs: to_vertex_dirs(mesh, dual, field),
            normals: mesh.normals().to_vec(),
            index: vertex_index(mesh, dual, field),
            half,
            max_steps,
        }
    }

    /// Os vértices singulares, em ordem — as sementes.
    #[must_use]
    pub fn singular(&self) -> Vec<u32> {
        (0..self.index.len())
            .filter(|&v| self.index[v] != 0)
            .map(|v| u32::try_from(v).unwrap_or(u32::MAX))
            .collect()
    }

    /// **AS QUATRO DIREÇÕES** da cruz no vértice `v`.
    #[must_use]
    pub fn cross(&self, v: usize) -> [[f32; 3]; 4] {
        let d = self.dirs[v];
        let t = cross3(self.normals[v], d);
        [d, t, [-d[0], -d[1], -d[2]], [-t[0], -t[1], -t[2]]]
    }

    /// **TRAÇA TUDO** — de cada singularidade, nas quatro direções.
    ///
    /// ⚠️ **A ordem é a dos índices de vértice e depois a das quatro direções**, e
    /// ela importa: uma separatriz pára ao bater numa já traçada, logo o
    /// resultado depende de quem correu antes. Fixá-la é o que torna a
    /// decomposição a mesma em qualquer máquina.
    #[must_use]
    pub fn trace_all(&self) -> (Walls, TraceReport) {
        let mut walls = Walls::default();
        let sing = self.singular();
        // As singularidades são nós desde o início: uma separatriz que chega a
        // uma delas terminou, mesmo que nenhuma outra tenha passado por ali.
        let mut on_trace = vec![false; self.mesh.vert_count()];
        for &v in &sing {
            on_trace[v as usize] = true;
        }
        let mut report = TraceReport {
            singularities: sing.len(),
            open_rings: (0..self.mesh.vert_count())
                .filter(|&v| {
                    Ring::build(self.mesh, &self.half, u32::try_from(v).unwrap_or(0)).is_none()
                })
                .count(),
            ..TraceReport::default()
        };
        for &v in &sing {
            for first in self.seeds_of(v) {
                match self.walk_from(v, first, &on_trace) {
                    Some(path) if path.len() > MIN_EDGES => {
                        for pair in path.windows(2) {
                            let (a, b) = (pair[0], pair[1]);
                            walls.edges.insert((a.min(b), a.max(b)));
                            *walls.degree.entry(a).or_default() += 1;
                            *walls.degree.entry(b).or_default() += 1;
                        }
                        for &w in &path {
                            on_trace[w as usize] = true;
                        }
                        report.separatrices += 1;
                    }
                    // ⚠️ Um caminho curto demais não é um erro do passeio: é a
                    // segunda saída da mesma singularidade a bater na primeira.
                    Some(_) => {}
                    None => report.dangling += 1,
                }
            }
        }
        (walls, report)
    }

    /// **AS SEMENTES de uma singularidade** — os vizinhos por onde as
    /// separatrizes saem. Ver [`crate::ring::Ring::seeds`].
    #[must_use]
    pub fn seeds_of(&self, v: u32) -> Vec<u32> {
        let Some(ring) = Ring::build(self.mesh, &self.half, v) else {
            return Vec::new();
        };
        // A fase: o vizinho mais alinhado com a cruz do vértice.
        let pos = self.mesh.positions();
        let d = self.dirs[v as usize];
        let mut phase = 0usize;
        let mut best = f32::NEG_INFINITY;
        for (i, &c) in ring.verts.iter().enumerate() {
            let e = normalize_or(sub(pos[c as usize], pos[v as usize]), d);
            let s = dot(e, d).abs();
            if s > best {
                best = s;
                phase = i;
            }
        }
        ring.seeds(self.index[v as usize], phase)
    }

    /// **UM PASSEIO**, com o primeiro vizinho já escolhido. Devolve o caminho de
    /// vértices, ou `None` se ele não fechou dentro do limite de passos.
    #[must_use]
    pub fn walk_from(&self, start: u32, first: u32, on_trace: &[bool]) -> Option<Vec<u32>> {
        let pos = self.mesh.positions();
        let adj = self.mesh.adjacency();
        let mut path = vec![start, first];
        if on_trace[first as usize] {
            return Some(path);
        }
        let mut dir = snap(
            self.cross(first as usize),
            normalize_or(
                sub(pos[first as usize], pos[start as usize]),
                self.dirs[first as usize],
            ),
        );
        let mut prev = start;
        let mut cur = first;
        for _ in 0..self.max_steps {
            let mut best = (f32::NEG_INFINITY, u32::MAX);
            for &w in adj.vert_verts.neighbours(cur as usize) {
                if w == prev {
                    continue;
                }
                let e = sub(pos[w as usize], pos[cur as usize]);
                let s = dot(normalize_or(e, dir), dir);
                if s > best.0 {
                    best = (s, w);
                }
            }
            let next = best.1;
            if next == u32::MAX {
                return None;
            }
            path.push(next);
            if on_trace[next as usize] {
                return Some(path);
            }
            let e = normalize_or(sub(pos[next as usize], pos[cur as usize]), dir);
            dir = snap(self.cross(next as usize), e);
            prev = cur;
            cur = next;
        }
        None
    }
}

/// A direção da cruz mais alinhada com `e`.
fn snap(cross: [[f32; 3]; 4], e: [f32; 3]) -> [f32; 3] {
    let mut best = (f32::NEG_INFINITY, cross[0]);
    for c in cross {
        let s = dot(c, e);
        if s > best.0 {
            best = (s, c);
        }
    }
    best.1
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}

fn normalize_or(v: [f32; 3], fallback: [f32; 3]) -> [f32; 3] {
    let n = dot(v, v).sqrt();
    if n > 1e-20 {
        [v[0] / n, v[1] / n, v[2] / n]
    } else {
        fallback
    }
}
