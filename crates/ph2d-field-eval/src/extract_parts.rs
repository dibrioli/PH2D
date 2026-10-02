//! ⭐⭐⭐ **AS PEÇAS DA EXTRAÇÃO** — uma malha por SÓLIDO conexo (ordem do dono, 2026-10-02).
//!
//! *«Objetos que se sobrepõem em operações booleanas se fundem em um só; uma peça afastada das outras
//! vira objeto separado.»* ⇒ a pergunta é de **sólido**, não de superfície: a esfera oca tem duas
//! cascas e é UMA peça; a bolinha solta na cavidade é OUTRA. Por isso o rótulo vive nas AMOSTRAS de
//! dentro (`f < 0`), com 6-vizinhança, e não nas faces.
//!
//! ⚠️ **E a célula que cruza a superfície une TODOS os seus cantos de dentro**: o Dual Contouring põe
//! UM vértice por célula, então dois sólidos a menos de uma célula um do outro já partilham vértice
//! na malha — à resolução da grade eles TOCAM, e chamá-los separados daria uma malha com um vértice
//! em duas peças.
//!
//! Corre DENTRO da varredura do [`crate::extract`] (as duas camadas que ela já tem), e o
//! [`crate::extract::extract`] sem rótulo sai byte a byte igual — é a mesma função com `None`.

use ph2d_mesh::{Face, Mesh};

use crate::MeshError;

const NONE: u32 = u32::MAX;

/// ⭐ **As PEÇAS da extração** — uma malha por sólido conexo. As peças PARTEM a malha do
/// [`crate::extract::extract`]: mesmos vértices e faces, nenhum a mais nem a menos.
///
/// # Errors
/// Os do [`crate::extract::extract`].
pub fn extract_parts(
    doc: &ph2d_field::FieldDoc,
    reg: &crate::hybrid::Registry,
    depth: u8,
) -> Result<Vec<Mesh>, MeshError> {
    let m = (1usize << depth) + 1;
    let mut labeler = Labeler::new(m);
    let threads = crate::extract_planes::threads_for(m);
    let (positions, faces) =
        crate::extract::sweep(doc, reg, depth, Some(&mut labeler), threads, true)?;
    split(positions, faces, labeler)
}

/// União-busca sobre as amostras de dentro.
#[derive(Default)]
struct Uf {
    parent: Vec<u32>,
}

impl Uf {
    fn make(&mut self) -> u32 {
        let id = self.parent.len() as u32;
        self.parent.push(id);
        id
    }

    fn find(&mut self, mut a: u32) -> u32 {
        let mut root = a;
        while self.parent[root as usize] != root {
            root = self.parent[root as usize];
        }
        while self.parent[a as usize] != root {
            let next = self.parent[a as usize];
            self.parent[a as usize] = root;
            a = next;
        }
        root
    }

    fn union(&mut self, a: u32, b: u32) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            // O menor fica raiz: a resposta não depende da ordem de chegada (HR-5).
            let (lo, hi) = if ra < rb { (ra, rb) } else { (rb, ra) };
            self.parent[hi as usize] = lo;
        }
    }
}

/// ⭐ **O rotulador que viaja na varredura** — as duas camadas de rótulos, e o rótulo de cada vértice.
pub(crate) struct Labeler {
    uf: Uf,
    m: usize,
    lo: Vec<u32>,
    hi: Vec<u32>,
    /// O rótulo (por resolver) de cada vértice emitido, na ordem dos vértices.
    verts: Vec<u32>,
}

impl Labeler {
    fn new(m: usize) -> Self {
        Self {
            uf: Uf::default(),
            m,
            lo: vec![NONE; m * m],
            hi: vec![NONE; m * m],
            verts: Vec::new(),
        }
    }

    /// Rotula uma camada de amostras. `first` = é a camada `0` (vai para `lo`); senão vai para `hi`
    /// e liga-se à `lo` pelo eixo `z`.
    pub(crate) fn plane(&mut self, values: &[f32], first: bool) {
        let m = self.m;
        let mut out = std::mem::take(if first { &mut self.lo } else { &mut self.hi });
        for j in 0..m {
            for i in 0..m {
                let s = j * m + i;
                if values[s] >= 0.0 {
                    out[s] = NONE;
                    continue;
                }
                let id = self.uf.make();
                out[s] = id;
                if i > 0 && out[s - 1] != NONE {
                    self.uf.union(id, out[s - 1]);
                }
                if j > 0 && out[s - m] != NONE {
                    self.uf.union(id, out[s - m]);
                }
                if !first && self.lo[s] != NONE {
                    self.uf.union(id, self.lo[s]);
                }
            }
        }
        if first {
            self.lo = out;
        } else {
            self.hi = out;
        }
    }

    /// A célula `(i, j)` da camada corrente emitiu um vértice: une os cantos de dentro e guarda o
    /// rótulo dele.
    pub(crate) fn cell_vertex(&mut self, i: usize, j: usize) {
        let m = self.m;
        let mut first = NONE;
        for b in 0..8 {
            let s = (j + ((b >> 1) & 1)) * m + i + (b & 1);
            let l = if b & 4 == 0 { self.lo[s] } else { self.hi[s] };
            if l == NONE {
                continue;
            }
            if first == NONE {
                first = l;
            } else {
                self.uf.union(first, l);
            }
        }
        debug_assert_ne!(first, NONE, "célula com vértice sem canto de dentro");
        self.verts.push(first);
    }

    /// A camada `hi` passa a `lo`.
    pub(crate) fn advance(&mut self) {
        std::mem::swap(&mut self.lo, &mut self.hi);
    }

    /// O rótulo final e COMPACTO de cada vértice — `0..k`, na ordem do primeiro vértice de cada peça.
    fn resolve(mut self) -> (Vec<u32>, usize) {
        let mut compact: std::collections::BTreeMap<u32, u32> = std::collections::BTreeMap::new();
        let mut out = Vec::with_capacity(self.verts.len());
        for v in std::mem::take(&mut self.verts) {
            let root = self.uf.find(v);
            let next = compact.len() as u32;
            out.push(*compact.entry(root).or_insert(next));
        }
        let k = compact.len();
        (out, k)
    }
}

/// Parte a malha inteira em uma por peça. Uma face é da peça dos seus vértices (todos iguais por
/// construção: o quad rodeia uma aresta da grade cujo canto de dentro é canto das quatro células).
fn split(
    positions: Vec<[f32; 3]>,
    faces: Vec<Face>,
    labeler: Labeler,
) -> Result<Vec<Mesh>, MeshError> {
    let (label, k) = labeler.resolve();
    let mut local = vec![u32::MAX; positions.len()];
    let mut pos: Vec<Vec<[f32; 3]>> = vec![Vec::new(); k];
    for (v, p) in positions.into_iter().enumerate() {
        let part = label[v] as usize;
        local[v] = pos[part].len() as u32;
        pos[part].push(p);
    }
    let mut fs: Vec<Vec<Face>> = vec![Vec::new(); k];
    for f in faces {
        let part = label[f.0[0] as usize] as usize;
        debug_assert!(
            f.verts()
                .iter()
                .all(|&v| label[v as usize] as usize == part),
            "face partilhada entre duas peças"
        );
        let mut g = f;
        for slot in &mut g.0[..f.vert_count()] {
            *slot = local[*slot as usize];
        }
        fs[part].push(g);
    }
    pos.into_iter()
        .zip(fs)
        .map(|(p, f)| Mesh::from_parts(p, f).map_err(|e| MeshError::Rejected(format!("{e:?}"))))
        .collect()
}
