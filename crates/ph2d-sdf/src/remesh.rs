//! O **botão**: uma malha entra, uma malha nova sai.
//!
//! Adaptado de `Remesh.remesh` do SculptGL (MIT). Licença em
//! `LICENSES/sculptgl-MIT.txt`.

use ph2d_mesh::{Mesh, MeshError, fill_holes};

use crate::field::{DEFAULT_RESOLUTION, VoxelField};
use crate::surface_nets::surface_nets;

/// O que o remesh fez, para quem quiser dizê-lo ao artista.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemeshReport {
    /// Vértices antes e depois.
    pub verts: (usize, usize),
    /// Faces antes e depois.
    pub faces: (usize, usize),
    /// Quantos buracos foram tapados para o campo poder ter um dentro.
    pub holes_filled: usize,
    /// Células da grade — o número que explica o custo.
    pub cells: usize,
}

/// Reconstrói `mesh` por voxelização, na resolução dada.
///
/// # A ordem, e por que cada passo precede o seguinte
///
/// 1. **Tapar buracos.** Uma superfície com beira não tem dentro: o flood fill
///    entra pelo furo e o campo inteiro sai positivo, devolvendo nada. A malha
///    de entrada é clonada antes — tapar é uma exigência do ALGORITMO, não uma
///    edição que o artista pediu.
/// 2. **Voxelizar** — a distância, que é local.
/// 3. **Flood fill** — o sinal, que não é.
/// 4. **Extrair** a superfície de nível zero.
///
/// ⚠️ **Não há um 5º passo de re-alinhamento**, e a referência tem: lá os
/// vértices saem em coordenadas de GRADE e um `alignMeshBound` os re-escala pela
/// diagonal da caixa no fim. Aqui a extração já emite mundo (`origem + coord ×
/// passo`), então não há para onde re-alinhar — e some junto o erro de uma
/// escala derivada de uma diagonal, que não é a mesma coisa que a caixa.
pub fn remesh(mesh: &Mesh, resolution: u32) -> Result<(Mesh, RemeshReport), MeshError> {
    let verts_before = mesh.vert_count();
    let faces_before = mesh.face_count();

    let mut closed = Mesh::from_parts(mesh.positions().to_vec(), mesh.faces().to_vec())?;
    let fill = fill_holes(&mut closed);

    let mut field = VoxelField::for_bounds(closed.bounds(), resolution);
    field.voxelize(&closed);
    field.flood_fill();
    let cells = field.cell_count();

    let out = surface_nets(&field)?;
    let report = RemeshReport {
        verts: (verts_before, out.vert_count()),
        faces: (faces_before, out.face_count()),
        holes_filled: fill.filled(),
        cells,
    };
    Ok((out, report))
}

/// [`remesh`] na resolução da referência.
pub fn remesh_default(mesh: &Mesh) -> Result<(Mesh, RemeshReport), MeshError> {
    remesh(mesh, DEFAULT_RESOLUTION)
}

#[cfg(test)]
#[path = "remesh_tests.rs"]
mod tests;
