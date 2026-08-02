//! **FECHAR BURACO** — a malha aberta vira sólido.
//!
//! Adaptado de `reference/sculptgl/src/editing/HoleFilling.js`, MIT — ver
//! `LICENSES/sculptgl-MIT.txt`.
//!
//! Duas razões, e a segunda é a que aperta:
//!
//! 1. **O artista importou um modelo com furos** e quer tapá-los antes de
//!    esculpir. Um furo não é só feio: o pincel atravessa por ele, a silhueta
//!    vaza e a simetria não tem o que espelhar do outro lado.
//! 2. **Voxelizar exige malha FECHADA** (W7). O flood fill assinado que decide
//!    *dentro* × *fora* não tem resposta numa superfície com beira, e o remesh
//!    inteiro se apoia nisso.
//!
//! # O contorno é uma CADEIA de arestas de valência 1
//!
//! Uma aresta usada por **uma** face é beira. Colhidas na ordem em que a face
//! que as possui as percorre, elas encadeiam: a ponta de uma é o começo da
//! seguinte, e o contorno fecha quando se volta ao começo.
//!
//! ⚠️ **A direção do encadeamento é a das FACES, e o remendo tem de andar ao
//! CONTRÁRIO.** Duas faces vizinhas são consistentes quando percorrem a aresta
//! partilhada em sentidos opostos, então o triângulo que tapa a aresta `a → b`
//! é `(b, a, centro)`. Escrevê-lo como `(a, b, centro)` — que é o que o original
//! faz — produz um remendo de normal INVERTIDA: a malha fecha, todo gate de
//! contagem fica verde, e o buraco vira uma tampa preta. Há gate de
//! **orientação**, não de contagem.
//!
//! # O remendo é um LEQUE, e a escolha é do original
//!
//! Um vértice novo no centroide do contorno, e um triângulo por aresta. É o que
//! o `HoleFilling.js` faz (o comentário dele diz *"stupid naive hole filling for
//! now"*), e para uma malha de escultura ele tem uma vantagem que as
//! alternativas não têm: **é manifold e orientável por construção, sempre**. Um
//! *ear clipping* não acrescenta vértice, mas exige projetar o contorno num
//! plano — e num furo torto ele se auto-intersecta.
//!
//! ⚠️ **Divergência: um contorno de TRÊS arestas vira UM triângulo**, sem
//! vértice novo. O leque ali põe um vértice de valência 3 no meio do que já era
//! um triângulo — uma verruga de topologia que o artista teria de limpar depois.
//!
//! ⚠️ **O centroide AFUNDA numa superfície curva**, e isso não é corrigido aqui:
//! empurrá-lo pela normal média exige um número que ninguém mediu, e inventá-lo
//! seria escrever um limite sem a tabela ao lado (CLAUDE.md §0). O que se faz
//! depois é o que se faria a qualquer tampa: um Smooth.

use crate::face::Face;
use crate::mesh::Mesh;

/// O que aconteceu ao fechar — e o que desfazer precisa.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HoleFill {
    filled: usize,
    left_open: usize,
    verts_before: usize,
    faces_before: usize,
}

impl HoleFill {
    /// Quantos contornos foram tapados.
    #[must_use]
    pub fn filled(&self) -> usize {
        self.filled
    }

    /// Quantas arestas de beira **sobraram**, por não formarem contorno fechado.
    ///
    /// ⚠️ Ela existe para o número aparecer, e não para o chamador tratá-la: uma
    /// beira que não fecha é uma malha com topologia estranha (dois furos que se
    /// tocam num vértice), e *deixar em silêncio* é como o artista conclui que o
    /// botão não funciona.
    #[must_use]
    pub fn left_open(&self) -> usize {
        self.left_open
    }

    /// A operação mudou alguma coisa?
    #[must_use]
    pub fn is_noop(&self) -> bool {
        self.filled == 0
    }

    /// Quantos vértices a malha tinha ANTES — o que desfazer trunca.
    #[must_use]
    pub fn verts_before(&self) -> usize {
        self.verts_before
    }

    /// Quantas faces a malha tinha ANTES.
    #[must_use]
    pub fn faces_before(&self) -> usize {
        self.faces_before
    }
}

/// **Tapa todo contorno aberto da malha.**
///
/// ⚠️ **Ela só ACRESCENTA** — nenhum vértice nem face que já existia é tocado. É
/// isso que faz o desfazer custar dois `usize` (truncar) em vez de uma cópia do
/// documento, e é uma propriedade do algoritmo, não uma coincidência: um remendo
/// é geometria NOVA colada na beira.
pub fn fill_holes(mesh: &mut Mesh) -> HoleFill {
    let verts_before = mesh.vert_count();
    let faces_before = mesh.face_count();
    let Some(loops) = border_loops(mesh) else {
        return HoleFill {
            filled: 0,
            left_open: 0,
            verts_before,
            faces_before,
        };
    };

    let mut positions = mesh.positions().to_vec();
    let mut faces = mesh.faces().to_vec();
    let mut new_colors: Vec<[f32; 3]> = Vec::new();
    let mut new_masks: Vec<f32> = Vec::new();
    let mut filled = 0usize;

    for ring in &loops.closed {
        // ⚠️ Um contorno de TRÊS arestas é um triângulo, e um leque ali seria um
        // vértice de valência 3 no meio dele. O `rev` é a mesma lei do leque: o
        // remendo anda ao contrário das faces.
        if ring.len() == 3 {
            faces.push(Face::tri(ring[2], ring[1], ring[0]));
            filled += 1;
            continue;
        }
        let centre = u32::try_from(positions.len()).unwrap_or(u32::MAX);
        let n = ring.len() as f32;
        let mut sum = [0.0f32; 3];
        for &v in ring {
            let p = mesh.positions()[v as usize];
            for k in 0..3 {
                sum[k] += p[k];
            }
        }
        positions.push([sum[0] / n, sum[1] / n, sum[2] / n]);
        // Os canais do vértice novo são a média do contorno — a mesma lei do
        // original, e a única que não inventa cor nem máscara.
        if let Some(c) = mesh.colors() {
            let mut s = [0.0f32; 3];
            for &v in ring {
                for k in 0..3 {
                    s[k] += c[v as usize][k];
                }
            }
            new_colors.push([s[0] / n, s[1] / n, s[2] / n]);
        }
        if let Some(m) = mesh.masks() {
            let s: f32 = ring.iter().map(|&v| m[v as usize]).sum();
            new_masks.push(s / n);
        }
        for k in 0..ring.len() {
            let (a, b) = (ring[k], ring[(k + 1) % ring.len()]);
            faces.push(Face::tri(b, a, centre));
        }
        filled += 1;
    }

    if filled == 0 {
        return HoleFill {
            filled: 0,
            left_open: loops.left_open,
            verts_before,
            faces_before,
        };
    }

    let colors = mesh.colors().map(|c| {
        let mut v = c.to_vec();
        v.extend_from_slice(&new_colors);
        v
    });
    let masks = mesh.masks().map(|m| {
        let mut v = m.to_vec();
        v.extend_from_slice(&new_masks);
        v
    });
    let mut out = Mesh::from_parts(positions, faces).expect("o remendo não inventa índice");
    if let Some(c) = colors {
        out.colors_mut().copy_from_slice(&c);
    }
    if let Some(m) = masks {
        out.put_masks(m);
    }
    *mesh = out;

    HoleFill {
        filled,
        left_open: loops.left_open,
        verts_before,
        faces_before,
    }
}

/// Os contornos fechados, mais quantas arestas de beira ficaram de fora.
struct Loops {
    closed: Vec<Vec<u32>>,
    left_open: usize,
}

/// Encadeia as arestas de beira em contornos.
///
/// `None` quando não há beira nenhuma — a malha já é fechada, e o caminho mais
/// curto é o que não aloca.
fn border_loops(mesh: &Mesh) -> Option<Loops> {
    let edges = mesh.edges();
    // `next[a] = b` para a aresta de beira que SAI de `a`, na direção da face
    // que a possui.
    let mut next = vec![u32::MAX; mesh.vert_count()];
    let mut border = 0usize;
    for (f, face) in mesh.faces().iter().enumerate() {
        let v = face.verts();
        for k in 0..v.len() {
            let Some(e) = edges.face_edge(f, k) else {
                continue;
            };
            if edges.valence(e) != 1 {
                continue;
            }
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            border += 1;
            // ⚠️ Dois furos que se tocam num vértice dão DUAS saídas ali, e não
            // há resposta boa para qual seguir. A segunda é descartada, e as
            // arestas que ficarem sem caminho aparecem no `left_open` — a malha
            // fica aberta ali, dito em voz alta.
            if next[a as usize] == u32::MAX {
                next[a as usize] = b;
            }
        }
    }
    if border == 0 {
        return None;
    }

    let mut visited = vec![false; mesh.vert_count()];
    let mut closed: Vec<Vec<u32>> = Vec::new();
    let mut used = 0usize;
    for start in 0..mesh.vert_count() {
        if visited[start] || next[start] == u32::MAX {
            continue;
        }
        let mut ring = vec![u32::try_from(start).unwrap_or(u32::MAX)];
        visited[start] = true;
        let mut cur = next[start];
        loop {
            if cur as usize == start {
                break;
            }
            // Beco sem saída, ou um caminho que reencontra um contorno já
            // fechado: os dois deixam a beira aberta.
            if next[cur as usize] == u32::MAX || visited[cur as usize] {
                ring.clear();
                break;
            }
            visited[cur as usize] = true;
            ring.push(cur);
            cur = next[cur as usize];
        }
        if ring.len() >= 3 {
            used += ring.len();
            closed.push(ring);
        }
    }
    Some(Loops {
        closed,
        left_open: border - used,
    })
}

#[cfg(test)]
#[path = "holes_tests.rs"]
mod tests;
