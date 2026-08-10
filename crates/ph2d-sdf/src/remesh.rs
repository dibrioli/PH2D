//! O **botão**: uma malha entra, uma malha nova sai.
//!
//! Adaptado de `Remesh.remesh` do SculptGL (MIT). Licença em
//! `LICENSES/sculptgl-MIT.txt`.

use ph2d_mesh::{Mesh, MeshError, fill_holes};

use crate::field::{DEFAULT_RESOLUTION, VoxelField};
use crate::surface_nets::surface_nets;

/// Por que um remesh RECUSA.
///
/// ⚠️ **Ele não vive no `MeshError`**, e a razão é de dono: aquele enum fala de
/// uma malha malformada (um índice de face fora de alcance), e *"o campo saiu
/// sem interior"* é fato do CAMPO. Pendurá-lo lá faria a `ph2d-mesh` conhecer
/// voxel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemeshError {
    /// **A onda do flood fill alcançou TODA célula** — não sobrou dentro, logo
    /// não há nível zero, logo a extração devolveria uma malha vazia.
    ///
    /// ⚠️ **É alcançável no produto e não é um teto:** varrendo a esfera
    /// `uv(96,144)`, ONZE das cem resoluções entre 100 e 200 vazam (`112, 151,
    /// 160, 161, 168, 180, 181, 193, 194, 196, 197`), e o
    /// [`DEFAULT_RESOLUTION`] que shipa é **150** — vizinho de uma delas. Quem
    /// decide é o alinhamento da grade contra os triângulos, então outra malha
    /// vaza noutros números.
    NoInterior {
        /// A resolução pedida — o número que o artista pode mudar.
        resolution: u32,
        /// Quantas células a grade tinha, para o log poder dizer o tamanho.
        cells: usize,
    },
    /// A malha reconstruída não pôde ser montada.
    Mesh(MeshError),
}

impl From<MeshError> for RemeshError {
    fn from(e: MeshError) -> Self {
        Self::Mesh(e)
    }
}

impl core::fmt::Display for RemeshError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoInterior { resolution, cells } => write!(
                f,
                "o campo saiu sem interior na resolução {resolution} ({cells} células): \
                 a onda alcançou tudo e não há superfície a extrair"
            ),
            Self::Mesh(e) => write!(f, "{e}"),
        }
    }
}

impl core::error::Error for RemeshError {}

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
pub fn remesh(mesh: &Mesh, resolution: u32) -> Result<(Mesh, RemeshReport), RemeshError> {
    let verts_before = mesh.vert_count();
    let faces_before = mesh.face_count();

    let mut closed = Mesh::from_parts(mesh.positions().to_vec(), mesh.faces().to_vec())?;
    let fill = fill_holes(&mut closed);

    let mut field = VoxelField::for_bounds(closed.bounds(), resolution);
    field.voxelize(&closed);
    let inside = field.flood_fill();
    let cells = field.cell_count();

    // ⚠️ **A RECUSA, e por que ela vem ANTES da extração.** Sem interior o
    // `surface_nets` devolve uma malha VAZIA sem errar — e o chamador a instala
    // no lugar da escultura, com log de sucesso. Recusar aqui custa a extração
    // que não teria o que extrair, e devolve ao artista a única coisa que ele
    // pode usar: o nome da causa e o número que ele controla.
    if inside == 0 {
        return Err(RemeshError::NoInterior { resolution, cells });
    }

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
pub fn remesh_default(mesh: &Mesh) -> Result<(Mesh, RemeshReport), RemeshError> {
    remesh(mesh, DEFAULT_RESOLUTION)
}

#[cfg(test)]
#[path = "remesh_tests.rs"]
mod tests;
