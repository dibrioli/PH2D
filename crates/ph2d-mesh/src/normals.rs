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
//! disjuntas, que é exatamente a forma que o ADR-0109 pede para paralelizar.
//!
//! ⚠️ **E paralelizamos, porque a MEDIÇÃO pediu.** A W1/M3 mediu um dab de raio
//! grande numa malha de 5 M triângulos em 23,7 ms, dos quais **16,0 eram estas
//! normais** — 67% do gesto. O paralelismo é **byte-idêntico por construção**:
//! ele muda qual thread avalia qual vértice, nunca a ordem da soma DENTRO de um
//! vértice, que é a única ordem que o `f32` enxerga. O caminho serial fica como
//! oráculo, com gate de paridade.

use rayon::prelude::*;

use crate::face::Face;

/// A partir de quantos itens vale acordar o `rayon`.
///
/// **Medido, não escolhido** (`measure_normals`, workstation de 32 threads):
/// abaixo de alguns milhares de itens o custo de distribuir o trabalho passa do
/// trabalho, e um dab de DETALHE — que toca centenas de vértices — ficaria mais
/// lento. Se a tabela do M3 mudar, este número muda junto, com a tabela ao lado.
pub const PAR_MIN: usize = 4_096;

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
    out.resize(faces.len(), [0.0, 1.0, 0.0]);
    if faces.len() < PAR_MIN {
        for (o, &f) in out.iter_mut().zip(faces) {
            *o = face_normal(positions, f);
        }
    } else {
        out.par_iter_mut()
            .zip(faces.par_iter())
            .for_each(|(o, &f)| *o = face_normal(positions, f));
    }
}

/// As normais das faces listadas, **na ordem de `which`** — o chamador espalha.
///
/// A separação existe pelo paralelismo: escrever direto nos índices esparsos do
/// buffer da malha seria escrita concorrente sobre o mesmo `Vec`. Computar para
/// um vetor contíguo e espalhar depois mantém a leitura pura e a escrita
/// sequencial, que é barata.
pub fn face_normals_of(
    positions: &[[f32; 3]],
    faces: &[Face],
    which: &[u32],
    out: &mut Vec<[f32; 3]>,
) {
    out.clear();
    out.resize(which.len(), [0.0, 1.0, 0.0]);
    if which.len() < PAR_MIN {
        for (o, &fi) in out.iter_mut().zip(which) {
            *o = face_normal(positions, faces[fi as usize]);
        }
    } else {
        out.par_iter_mut()
            .zip(which.par_iter())
            .for_each(|(o, &fi)| *o = face_normal(positions, faces[fi as usize]));
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
        None if normals.len() < PAR_MIN => {
            for (v, out) in normals.iter_mut().enumerate() {
                *out = gather(face_normals, vert_faces, v);
            }
        }
        None => {
            normals
                .par_iter_mut()
                .enumerate()
                .for_each(|(v, out)| *out = gather(face_normals, vert_faces, v));
        }
        // Índices esparsos: quem quer paralelismo nesta rota usa
        // [`vertex_normals_of`] e espalha — é o que o `refresh_region` faz.
        // Este braço fica como o caminho SIMPLES e como oráculo de paridade.
        Some(list) => {
            for &v in list {
                normals[v as usize] = gather(face_normals, vert_faces, v as usize);
            }
        }
    }
}

/// As normais dos vértices listados, **na ordem de `which`** — o chamador
/// espalha. Irmã da [`face_normals_of`], pelo mesmo motivo.
pub fn vertex_normals_of(
    face_normals: &[[f32; 3]],
    vert_faces: &crate::adjacency::Csr,
    which: &[u32],
    out: &mut Vec<[f32; 3]>,
) {
    out.clear();
    out.resize(which.len(), [0.0, 1.0, 0.0]);
    if which.len() < PAR_MIN {
        for (o, &v) in out.iter_mut().zip(which) {
            *o = gather(face_normals, vert_faces, v as usize);
        }
    } else {
        out.par_iter_mut()
            .zip(which.par_iter())
            .for_each(|(o, &v)| *o = gather(face_normals, vert_faces, v as usize));
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
