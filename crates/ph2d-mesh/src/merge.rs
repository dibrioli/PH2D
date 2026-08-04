//! **A FUSÃO** — várias peças viram UMA malha, em coordenadas de MUNDO.
//!
//! ## Por que ela mora aqui, e não na cena
//!
//! Concatenar geometria é uma operação de MALHA: ela não pergunta nada sobre
//! quem está selecionado, que peça é a ativa, nem o que o artista apertou. Aqui
//! ela é gateada **sem device** — a cena inteira precisa de um `wgpu::Device`
//! para existir, e o defeito que uma fusão tem (um índice deslocado errado,
//! um canal perdido) é aritmético, não visual.
//!
//! ## A decisão que decide tudo: a saída é em MUNDO
//!
//! ⚠️ Desde a W8.1 a posição de uma peça vive na [`Pose`], não na geometria.
//! Concatenar as posições LOCAIS empilharia todas as peças na origem — é o
//! mesmo defeito que o `export` nomeia, e a mesma cura: **toda posição
//! atravessa [`Pose::point_to_world`]**. A peça que sai fica em
//! [`Pose::IDENTITY`], porque a pose dela já está assada nos vértices.
//!
//! ## Os canais por-vértice sobrevivem
//!
//! ⚠️ **A máscara e a cor viajam junto**, e quem não tem plano fica com o
//! default — que é exatamente o que *"ninguém pintou aqui"* significa
//! ([`DEFAULT_MASK`] = 0 é *totalmente esculpível*). Descartá-los seria destruir
//! trabalho autorado em silêncio: uma máscara é o que o artista pintou para
//! PROTEGER, e ela some no gesto que ele achou que só juntava peças.

use crate::face::Face;
use crate::mesh::{Mesh, MeshError};
use crate::pose::Pose;

/// **FUNDE as peças numa malha só, em coordenadas de mundo.**
///
/// ⚠️ Ela **não** solda nada: duas superfícies que se tocam continuam sendo
/// duas superfícies dentro da mesma malha, e o vão entre elas continua lá. Isso
/// não é uma limitação escondida, é a divisão de trabalho — quem transforma um
/// amontoado de peças numa casca fechada é o **remesh** (`ph2d-sdf`), e ele já
/// existe. Fundir é o passo que o torna possível: ele opera numa malha.
///
/// Com uma peça só ela **assa a pose** e mais nada; com nenhuma devolve a malha
/// vazia. Os dois casos são totais de propósito — a recusa (*não há o que
/// fundir*) é do chamador, que é quem sabe o que dizer ao artista.
pub fn merge(pieces: &[(&Mesh, Pose)]) -> Result<Mesh, MeshError> {
    let verts: usize = pieces.iter().map(|(m, _)| m.vert_count()).sum();
    let face_count: usize = pieces.iter().map(|(m, _)| m.face_count()).sum();
    let mut positions = Vec::with_capacity(verts);
    let mut faces = Vec::with_capacity(face_count);

    // ⚠️ **O deslocamento é a contagem ACUMULADA, e a face é reconstruída pelos
    // `verts()`** — nunca somando no `Face.0` cru. O quarto elemento de um
    // triângulo é o sentinela [`crate::TRI`], e somar nele produziria um índice
    // grande e válido: uma face que aponta para o vértice errado de outra peça,
    // sem erro nenhum. Ver `Face::verts`.
    let mut base = 0u32;
    for (mesh, pose) in pieces {
        positions.extend(mesh.positions().iter().map(|&p| pose.point_to_world(p)));
        for f in mesh.faces() {
            let v = f.verts();
            faces.push(if f.is_tri() {
                Face::tri(v[0] + base, v[1] + base, v[2] + base)
            } else {
                Face::quad(v[0] + base, v[1] + base, v[2] + base, v[3] + base)
            });
        }
        base += mesh.vert_count() as u32;
    }

    // ⚠️ O `?` não pode disparar hoje (os deslocamentos são exatos por
    // construção) e fica: ele é o gate que a aritmética acima terá no dia em que
    // alguém a reescrever.
    let mut out = Mesh::from_parts(positions, faces)?;

    // ⚠️ **Os planos só nascem se ALGUÉM os tinha** — materializá-los sempre
    // custaria 16 B/vértice numa cena que ninguém pintou nem mascarou, e faria a
    // fusão de duas malhas virgens produzir uma malha que **não** é virgem.
    if pieces.iter().any(|(m, _)| m.colors().is_some()) {
        let dst = out.colors_mut();
        let mut at = 0;
        for (m, _) in pieces {
            if let Some(c) = m.colors() {
                dst[at..at + m.vert_count()].copy_from_slice(c);
            }
            at += m.vert_count();
        }
    }
    if pieces.iter().any(|(m, _)| m.masks().is_some()) {
        let dst = out.masks_mut();
        let mut at = 0;
        for (m, _) in pieces {
            if let Some(k) = m.masks() {
                dst[at..at + m.vert_count()].copy_from_slice(k);
            }
            at += m.vert_count();
        }
    }
    Ok(out)
}

#[cfg(test)]
#[path = "merge_tests.rs"]
mod tests;
