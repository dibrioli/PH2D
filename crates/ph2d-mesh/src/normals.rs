//! Normais — por face (Newell) e por vértice (**gather** sobre a adjacência).
//!
//! Adaptado de `reference/sculptgl/src/mesh/Mesh.js` (`updateVerticesNormal`,
//! `updateFacesAabbAndNormal`), MIT — ver `LICENSES/sculptgl-MIT.txt`.
//!
//! ⚠️ **A normal do vértice é um GATHER, e essa é a decisão inteira.** A
//! alternativa óbvia — cada face *espalhar* sua normal nos três vértices — quer
//! ou atômicos ou uma ordem de soma que muda com o escalonamento, e soma de
//! `f32` não é associativa: a mesma malha daria normais diferentes entre
//! execuções. Lendo pela adjacência, cada vértice **escreve só o seu** e lê uma
//! lista de ordem fixa ⇒ o resultado é determinístico e as saídas são
//! disjuntas, que é exatamente a forma que o ADR-0109 pede para paralelizar
//! depois por linhas sem mudar um byte.
//!
//! ⚠️ **Serial de propósito nesta wave.** O `docs/3D/03.5` prevê `rayon`, mas o
//! `par_chunks_mut` entra quando a sonda `measure_normals` (M3) disser que ele é
//! preciso — o kernel já está na forma que torna a mudança de três linhas e
//! byte-idêntica por construção.

use crate::face::Face;

/// A normal de uma face por **Newell**: soma dos produtos cruzados das arestas.
///
/// Newell e não "cruzado de duas arestas" porque um **quad não é plano** — o
/// cruzado escolheria um canto e a normal saltaria conforme qual. Newell é a
/// média de área do polígono inteiro, e num triângulo reduz exatamente ao
/// cruzado, então há uma fórmula e não duas.
#[must_use]
pub fn face_normal(positions: &[[f32; 3]], face: Face) -> [f32; 3] {
    let vs = face.verts();
    let n = vs.len();
    let mut nx = 0.0f32;
    let mut ny = 0.0f32;
    let mut nz = 0.0f32;
    for i in 0..n {
        let a = positions[vs[i] as usize];
        let b = positions[vs[(i + 1) % n] as usize];
        nx += (a[1] - b[1]) * (a[2] + b[2]);
        ny += (a[2] - b[2]) * (a[0] + b[0]);
        nz += (a[0] - b[0]) * (a[1] + b[1]);
    }
    normalize([nx, ny, nz])
}

/// Recalcula TODAS as normais de face.
pub fn recompute_face_normals(positions: &[[f32; 3]], faces: &[Face], out: &mut Vec<[f32; 3]>) {
    out.clear();
    out.reserve(faces.len());
    for &f in faces {
        out.push(face_normal(positions, f));
    }
}

/// Recalcula as normais dos vértices em `verts` (todos, se `None`).
///
/// A normal é a média **normalizada** das normais das faces vizinhas. O
/// SculptGL para na média crua (que não é unitária) e deixa o shader dele
/// normalizar; aqui ela sai unitária da porta, porque os consumidores da CPU —
/// a doação do G-buffer, o ajuste de plano do Flatten — leem geometria e não
/// pixels.
///
/// Ponderar por ÁREA seria estritamente melhor em malha irregular e sai quase
/// de graça (bastaria guardar o comprimento de Newell); fica **nomeado e não
/// feito** até haver um smoke que mostre a diferença, porque o oráculo disso é
/// aparência, não número.
pub fn recompute_vertex_normals(
    face_normals: &[[f32; 3]],
    vert_faces: &crate::adjacency::Csr,
    normals: &mut [[f32; 3]],
    verts: Option<&[u32]>,
) {
    match verts {
        None => {
            for (v, out) in normals.iter_mut().enumerate() {
                *out = gather(face_normals, vert_faces, v);
            }
        }
        Some(list) => {
            for &v in list {
                normals[v as usize] = gather(face_normals, vert_faces, v as usize);
            }
        }
    }
}

fn gather(face_normals: &[[f32; 3]], vert_faces: &crate::adjacency::Csr, v: usize) -> [f32; 3] {
    let ring = vert_faces.neighbours(v);
    if ring.is_empty() {
        // Vértice solto: sem face não há orientação. `+Y` é uma escolha
        // arbitrária mas ESTÁVEL — um `NaN` daqui envenenaria o buffer inteiro.
        return [0.0, 1.0, 0.0];
    }
    let mut acc = [0.0f32; 3];
    for &fi in ring {
        let n = face_normals[fi as usize];
        acc[0] += n[0];
        acc[1] += n[1];
        acc[2] += n[2];
    }
    normalize(acc)
}

/// Normaliza, com o degenerado devolvendo `+Y` em vez de `NaN`.
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len2 = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if len2 <= f32::MIN_POSITIVE {
        return [0.0, 1.0, 0.0];
    }
    let inv = 1.0 / len2.sqrt();
    [v[0] * inv, v[1] * inv, v[2] * inv]
}
