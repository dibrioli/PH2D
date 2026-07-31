//! A malha residente: os buffers, a adjacência, as normais e o índice espacial
//! — a estrutura que o ADR-0150 chama de **representação primária**.
//!
//! Layout SoA (*struct of arrays*), adaptado de
//! `reference/sculptgl/src/mesh/MeshData.js`, MIT — ver `LICENSES/sculptgl-MIT.txt`.
//! SoA e não AoS porque é assim que os buffers sobem para a GPU e porque um dab
//! toca **posição** de milhares de vértices sem ler a cor de nenhum.
//!
//! ⚠️ **Cor e máscara são PREGUIÇOSAS**, o padrão dos planos do impasto no
//! Painter (`heights`/`covers`/`mats`, alocados só quando a camada é tocada):
//! uma malha importada que ninguém pintou não paga 12 B/vértice de cor nem
//! 4 B/vértice de máscara. `None` significa *o default uniforme*, e a
//! `measure_memory` mede as duas situações em vez de eu afirmar qual importa.
//!
//! ⚠️ **O que NÃO é guardado, e por quê:** centro e caixa por face. O SculptGL
//! os mantém (24 B/face — a 5 M faces, 120 MB) para atualizar o octree
//! incrementalmente. Os dois são O(1) a partir de posição+face, então aqui são
//! computados no build e descartados; quando a atualização incremental chegar
//! (W2), ela recomputa os da PEGADA, que é trabalho limitado pelo pincel.

use crate::aabb::Aabb;
use crate::adjacency::Adjacency;
use crate::face::Face;
use crate::normals;
use crate::octree::Octree;

/// A malha de triângulos/quads residente na CPU.
#[derive(Clone, Debug, Default)]
pub struct Mesh {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Option<Vec<[f32; 3]>>,
    masks: Option<Vec<f32>>,
    faces: Vec<Face>,
    face_normals: Vec<[f32; 3]>,
    adjacency: Adjacency,
    octree: Octree,
    bounds: Aabb,
}

/// Cor de um vértice que a malha ainda não pintou (branco).
pub const DEFAULT_COLOR: [f32; 3] = [1.0, 1.0, 1.0];
/// Máscara de um vértice que ninguém mascarou (0 = totalmente esculpível).
pub const DEFAULT_MASK: f32 = 0.0;

impl Mesh {
    /// Constrói a partir de posições e faces cruas: valida os índices, deriva
    /// normais, adjacência e octree.
    ///
    /// **Valida**, e isso não é zelo: um índice fora de alcance vindo de um OBJ
    /// de terceiro vira, sem checagem, uma leitura errada em cada kernel que
    /// tocar aquele vértice — e o sintoma aparece a três waves de distância,
    /// numa normal torta.
    pub fn from_parts(positions: Vec<[f32; 3]>, faces: Vec<Face>) -> Result<Self, MeshError> {
        let n = positions.len() as u32;
        for (fi, f) in faces.iter().enumerate() {
            for &v in f.verts() {
                if v >= n {
                    return Err(MeshError::VertexOutOfRange {
                        face: fi,
                        vertex: v,
                        vert_count: positions.len(),
                    });
                }
            }
        }
        let mut mesh = Self {
            normals: vec![[0.0, 1.0, 0.0]; positions.len()],
            positions,
            faces,
            ..Self::default()
        };
        mesh.rebuild();
        Ok(mesh)
    }

    /// Recomputa tudo que é DERIVADO de posições e faces: normais de face,
    /// normais de vértice, adjacência, octree e a caixa do mundo.
    ///
    /// Porta única de propósito. Uma wave que reconstrói só metade disto deixa
    /// o sistema **estável e errado** — o octree apontando para onde a malha
    /// estava —, e esse é o modo de falha que não levanta erro nenhum.
    pub fn rebuild(&mut self) {
        normals::recompute_face_normals(&self.positions, &self.faces, &mut self.face_normals);
        self.adjacency = Adjacency::build(self.positions.len(), &self.faces);
        normals::recompute_vertex_normals(
            &self.face_normals,
            &self.adjacency.vert_faces,
            &mut self.normals,
            None,
        );
        self.octree = Octree::build(&self.positions, &self.faces);
        self.bounds = Aabb::from_points(&self.positions);
    }

    #[must_use]
    pub fn vert_count(&self) -> usize {
        self.positions.len()
    }

    #[must_use]
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// Quantos triângulos a rasterização vê — o número que os tetos por tier
    /// (ADR-0104) falam, e que **não** é a contagem de faces quando há quads.
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.faces.iter().map(Face::tri_count).sum()
    }

    #[must_use]
    pub fn positions(&self) -> &[[f32; 3]] {
        &self.positions
    }

    #[must_use]
    pub fn normals(&self) -> &[[f32; 3]] {
        &self.normals
    }

    #[must_use]
    pub fn faces(&self) -> &[Face] {
        &self.faces
    }

    #[must_use]
    pub fn face_normals(&self) -> &[[f32; 3]] {
        &self.face_normals
    }

    #[must_use]
    pub fn adjacency(&self) -> &Adjacency {
        &self.adjacency
    }

    #[must_use]
    pub fn octree(&self) -> &Octree {
        &self.octree
    }

    #[must_use]
    pub fn bounds(&self) -> Aabb {
        self.bounds
    }

    /// A cor por vértice, se a malha já foi pintada. `None` = `DEFAULT_COLOR`
    /// em todo vértice.
    #[must_use]
    pub fn colors(&self) -> Option<&[[f32; 3]]> {
        self.colors.as_deref()
    }

    /// A máscara por vértice, se alguém mascarou. `None` = `DEFAULT_MASK`.
    #[must_use]
    pub fn masks(&self) -> Option<&[f32]> {
        self.masks.as_deref()
    }

    /// Materializa o plano de cor (aloca no primeiro uso — ver o doc do módulo).
    pub fn colors_mut(&mut self) -> &mut [[f32; 3]] {
        self.colors
            .get_or_insert_with(|| vec![DEFAULT_COLOR; self.positions.len()])
    }

    /// Materializa o plano de máscara.
    pub fn masks_mut(&mut self) -> &mut [f32] {
        self.masks
            .get_or_insert_with(|| vec![DEFAULT_MASK; self.positions.len()])
    }

    /// As posições para escrita — o que um kernel de pincel move.
    ///
    /// Quem escreve aqui **fica devendo** um `refresh_region`: a normal, o
    /// octree e a caixa passam a descrever a malha de antes. É deliberado que a
    /// dívida seja explícita em vez de a porta reconstruir tudo sozinha — um
    /// dab que reconstruísse o octree inteiro seria O(malha) num gesto que é
    /// O(pegada), que é precisamente o que este índice existe para evitar.
    pub fn positions_mut(&mut self) -> &mut [[f32; 3]] {
        &mut self.positions
    }

    /// Os vértices dentro da esfera — a consulta que um dab faz.
    ///
    /// O octree responde com as faces das folhas tocadas (conservador); aqui
    /// entra o filtro EXATO por distância e a deduplicação, porque um vértice
    /// aparece em todas as faces do anel dele.
    pub fn verts_in_sphere(
        &self,
        center: [f32; 3],
        radius: f32,
        scratch: &mut QueryScratch,
        out: &mut Vec<u32>,
    ) {
        out.clear();
        self.octree
            .faces_in_sphere(center, radius, &mut scratch.faces);
        if scratch.seen.len() != self.positions.len() {
            scratch.seen = vec![0u32; self.positions.len()];
            scratch.epoch = 0;
        }
        scratch.epoch = scratch.epoch.wrapping_add(1);
        // O carimbo 0 é o "nunca visto" do vetor recém-criado, então a época
        // nunca pode valer 0 — sem isto a primeira consulta após um wrap
        // devolveria vazio, uma vez a cada 4 bilhões e impossível de reproduzir.
        if scratch.epoch == 0 {
            scratch.epoch = 1;
            scratch.seen.fill(0);
        }
        let r2 = radius * radius;
        for &fi in &scratch.faces {
            for &v in self.faces[fi as usize].verts() {
                if scratch.seen[v as usize] == scratch.epoch {
                    continue;
                }
                scratch.seen[v as usize] = scratch.epoch;
                let p = self.positions[v as usize];
                let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
                if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= r2 {
                    out.push(v);
                }
            }
        }
    }

    /// Recalcula as normais afetadas por um deslocamento em `moved`.
    ///
    /// É a metade *limitada pela pegada* do `rebuild`: as normais de face das
    /// faces que tocam `moved`, e depois as normais de vértice de TODOS os
    /// vértices dessas faces — não só os que se moveram. Um vizinho parado ao
    /// lado de uma face que girou tem a normal mudada, e esquecê-lo deixa uma
    /// costura visível exatamente na borda do pincel, que é onde o artista está
    /// olhando.
    ///
    /// ⚠️ **NÃO mexe no octree.** Depois de um deslocamento ele descreve as
    /// posições anteriores; enquanto o dab move menos que a folga das caixas
    /// frouxas isso é invisível, e a atualização incremental por região é da W2
    /// (o custo dela é da PEGADA, como este passe). Reconstruí-lo aqui tornaria
    /// um gesto `O(pegada)` num gesto `O(malha)`, que é o oposto do desenho.
    pub fn refresh_region(&mut self, moved: &[u32], scratch: &mut RegionScratch) {
        if moved.is_empty() {
            return;
        }
        scratch.reset(self.faces.len(), self.positions.len());

        scratch.faces.clear();
        for &v in moved {
            for &fi in self.adjacency.vert_faces.neighbours(v as usize) {
                if !scratch.face_seen[fi as usize] {
                    scratch.face_seen[fi as usize] = true;
                    scratch.faces.push(fi);
                }
            }
        }
        scratch.verts.clear();
        for &fi in &scratch.faces {
            for &v in self.faces[fi as usize].verts() {
                if !scratch.vert_seen[v as usize] {
                    scratch.vert_seen[v as usize] = true;
                    scratch.verts.push(v);
                }
            }
        }
        // Calcula para um vetor CONTÍGUO e espalha — é o que deixa a leitura
        // pura e permite o `rayon`. Escrever direto nos índices esparsos seria
        // escrita concorrente sobre o mesmo `Vec`.
        normals::face_normals_of(
            &self.positions,
            &self.faces,
            &scratch.faces,
            &mut scratch.tmp,
        );
        for (&fi, n) in scratch.faces.iter().zip(&scratch.tmp) {
            self.face_normals[fi as usize] = *n;
        }
        normals::vertex_normals_of(
            &self.face_normals,
            &self.adjacency.vert_faces,
            &scratch.verts,
            &mut scratch.tmp,
        );
        for (&v, n) in scratch.verts.iter().zip(&scratch.tmp) {
            self.normals[v as usize] = *n;
        }
        // Limpa só o que sujou — zerar os vetores inteiros faria deste passe
        // `O(malha)` pela porta dos fundos.
        for &fi in &scratch.faces {
            scratch.face_seen[fi as usize] = false;
        }
        for &v in &scratch.verts {
            scratch.vert_seen[v as usize] = false;
        }
    }

    /// O buffer de índices que a GPU consome (quads triangulados).
    pub fn triangle_indices(&self, out: &mut Vec<[u32; 3]>) {
        out.clear();
        out.reserve(self.triangle_count());
        for f in &self.faces {
            f.triangles(out);
        }
    }
}

/// Buffers reutilizados entre consultas — a consulta é feita por movimento do
/// mouse, e alocar por dab é o que transforma um gesto em serrilhado.
#[derive(Clone, Debug, Default)]
pub struct QueryScratch {
    faces: Vec<u32>,
    seen: Vec<u32>,
    epoch: u32,
}

impl QueryScratch {
    /// Bytes que este scratch segura — a sonda de memória o soma para que o
    /// custo do gesto não fique fora da conta.
    #[must_use]
    pub fn capacity_bytes(&self) -> usize {
        (self.faces.capacity() + self.seen.capacity()) * size_of::<u32>()
    }
}

/// Buffers reutilizados pelo [`Mesh::refresh_region`].
///
/// Os `*_seen` são vetores do TAMANHO da malha, mas o passe só os toca onde
/// escreveu e os limpa na saída — é o que torna o custo função da pegada e não
/// da malha, e é a razão de eles viverem aqui em vez de nascerem por dab.
#[derive(Clone, Debug, Default)]
pub struct RegionScratch {
    faces: Vec<u32>,
    verts: Vec<u32>,
    face_seen: Vec<bool>,
    vert_seen: Vec<bool>,
    /// Saída contígua das portas paralelas, reusada pelas duas metades do passe.
    tmp: Vec<[f32; 3]>,
}

impl RegionScratch {
    fn reset(&mut self, faces: usize, verts: usize) {
        if self.face_seen.len() != faces {
            self.face_seen = vec![false; faces];
        }
        if self.vert_seen.len() != verts {
            self.vert_seen = vec![false; verts];
        }
    }

    /// Bytes que este scratch segura.
    #[must_use]
    pub fn capacity_bytes(&self) -> usize {
        (self.faces.capacity() + self.verts.capacity()) * size_of::<u32>()
            + self.face_seen.capacity()
            + self.vert_seen.capacity()
            + self.tmp.capacity() * size_of::<[f32; 3]>()
    }
}

/// O que impede uma malha inválida de existir.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshError {
    VertexOutOfRange {
        face: usize,
        vertex: u32,
        vert_count: usize,
    },
}

impl core::fmt::Display for MeshError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::VertexOutOfRange {
                face,
                vertex,
                vert_count,
            } => write!(
                f,
                "face {face} aponta para o vértice {vertex}, e a malha tem {vert_count}"
            ),
        }
    }
}

impl core::error::Error for MeshError {}

#[cfg(test)]
#[path = "mesh_tests.rs"]
mod tests;
