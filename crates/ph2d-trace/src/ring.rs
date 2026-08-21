//! **O ANEL DE UM VÉRTICE, EM ORDEM E COM ÂNGULOS** — de onde saem as sementes.
//!
//! # Por que uma singularidade não emite QUATRO separatrizes
//!
//! Um vértice regular de um campo 4-RoSy tem quatro direções a `90°` umas das
//! outras. **Uma singularidade não.** Num vértice de índice `k` a cruz só fecha
//! depois de `4 − k` quartos de volta, e as separatrizes saem espaçadas por
//! `Θ/(4−k)` — onde `Θ` é o ângulo total à volta do vértice, que num cone **não
//! é `2π`**.
//!
//! | índice | separatrizes | o que é, numa grade de quads |
//! |---|---|---|
//! | `+1` | **3** | o vértice de valência 3 |
//! | `−1` | **5** | o de valência 5 |
//! | `0` | — | regular, não emite |
//!
//! ⚠️ **Emitir sempre quatro parece inofensivo e não é**: medido em 2026-08-20
//! numa esfera de 830 vértices, quatro sementes por singularidade davam **23
//! patches** onde o oráculo dá **8**. A quarta separatriz de cada singularidade
//! não tem para onde ir e vai bater no meio de outra, e cada batida dessas é uma
//! junção em T a mais e um patch a mais.

use std::collections::BTreeMap;

use ph2d_mesh::{Face, Mesh};

/// O anel ordenado de um vértice: os vizinhos e o ângulo acumulado até cada um.
#[derive(Debug, Clone, PartialEq)]
pub struct Ring {
    /// Os vizinhos, na ordem em que se dá a volta.
    pub verts: Vec<u32>,
    /// O ângulo acumulado até cada vizinho — `cum[0]` é sempre `0`.
    pub cum: Vec<f32>,
    /// O ângulo total à volta do vértice. ⚠️ **Num cone não é `2π`**, e é
    /// exatamente essa diferença que faz a singularidade.
    pub total: f32,
}

impl Ring {
    /// **CONSTRÓI o anel** pivotando de face em face. `None` se ele não fecha —
    /// um vértice não-manifold ou de borda.
    #[must_use]
    pub fn build(mesh: &Mesh, half: &BTreeMap<(u32, u32), u32>, v: u32) -> Option<Self> {
        let faces = mesh.faces();
        let pos = mesh.positions();
        let adj = mesh.adjacency();
        let incident = adj.vert_faces.neighbours(v as usize);
        let first = *incident.first()?;
        let mut verts = Vec::with_capacity(incident.len());
        let mut cum = Vec::with_capacity(incident.len());
        let mut total = 0.0f32;
        let mut f = first;
        for _ in 0..incident.len() {
            let c = after(faces, f, v)?;
            verts.push(c);
            cum.push(total);
            total += angle_at(pos, faces, f, v)?;
            // A face do outro lado da aresta `(v, c)`.
            f = *half.get(&(c, v))?;
            if f == first {
                break;
            }
        }
        if verts.len() != incident.len() || f != first {
            return None;
        }
        Some(Self { verts, cum, total })
    }

    /// **AS SEMENTES de uma singularidade de índice `k`** — os vizinhos por onde
    /// as separatrizes saem.
    ///
    /// ⚠️ **A fase vem do CAMPO** (o vizinho mais alinhado com a cruz do vértice),
    /// e o espaçamento vem da GEOMETRIA (`Θ/m`). Trocar um pelo outro dá
    /// separatrizes que saem no sítio certo mas espaçadas erradas, ou o inverso —
    /// e as duas coisas produzem um patch a mais sem nenhum erro visível.
    #[must_use]
    pub fn seeds(&self, index: i32, phase: usize) -> Vec<u32> {
        let m = 4 - index;
        if m <= 0 || self.verts.is_empty() || self.total <= 0.0 {
            return Vec::new();
        }
        #[allow(clippy::cast_sign_loss)]
        let m = m as usize;
        #[allow(clippy::cast_precision_loss)]
        let step = self.total / m as f32;
        let base = self.cum.get(phase).copied().unwrap_or(0.0);
        let mut out = Vec::with_capacity(m);
        for j in 0..m {
            #[allow(clippy::cast_precision_loss)]
            let target = (base + step * j as f32) % self.total;
            let mut best = (f32::INFINITY, self.verts[0]);
            for (i, &c) in self.verts.iter().enumerate() {
                let d = circular(self.cum[i], target, self.total);
                if d < best.0 {
                    best = (d, c);
                }
            }
            if !out.contains(&best.1) {
                out.push(best.1);
            }
        }
        out
    }
}

/// A distância entre dois ângulos, dando a volta.
fn circular(a: f32, b: f32, total: f32) -> f32 {
    let d = (a - b).abs() % total;
    d.min(total - d)
}

/// O vértice que vem depois de `v` na face `f`.
fn after(faces: &[Face], f: u32, v: u32) -> Option<u32> {
    let t = faces.get(f as usize)?.verts();
    let k = t.iter().position(|&x| x == v)?;
    Some(t[(k + 1) % t.len()])
}

/// O ângulo que a face `f` ocupa no vértice `v`.
fn angle_at(pos: &[[f32; 3]], faces: &[Face], f: u32, v: u32) -> Option<f32> {
    let t = faces.get(f as usize)?.verts();
    let k = t.iter().position(|&x| x == v)?;
    let a = t[(k + t.len() - 1) % t.len()];
    let b = t[(k + 1) % t.len()];
    let (o, a, b) = (pos[v as usize], pos[a as usize], pos[b as usize]);
    let u = unit([a[0] - o[0], a[1] - o[1], a[2] - o[2]]);
    let w = unit([b[0] - o[0], b[1] - o[1], b[2] - o[2]]);
    Some(
        u[0].mul_add(w[0], u[1].mul_add(w[1], u[2] * w[2]))
            .clamp(-1.0, 1.0)
            .acos(),
    )
}

fn unit(v: [f32; 3]) -> [f32; 3] {
    let n = v[0].mul_add(v[0], v[1].mul_add(v[1], v[2] * v[2])).sqrt();
    if n > 1e-20 {
        [v[0] / n, v[1] / n, v[2] / n]
    } else {
        [1.0, 0.0, 0.0]
    }
}
