//! **Só os vértices que um conjunto de faces usa** — a porta única da
//! compactação.
//!
//! # ⚠️⚠️ Porque isto é uma PORTA e não um laço de dez linhas repetido
//!
//! Recortar uma peça de uma malha maior é sempre o mesmo gesto — **escolher
//! faces** — e o pool de posições é da malha INTEIRA. Quem guarda o pool sem
//! compactar fica com uma nuvem de **vértices órfãos**: o [`crate::Mesh::from_parts`]
//! aceita-os (nenhuma face os cita, logo nenhum índice está fora de alcance), o
//! octree indexa-os, a caixa do mundo enxerga-os e toda busca por
//! *«o vértice mais próximo daqui»* pode **aterrar num deles**.
//!
//! ⛔ **E o modo de falha não se parece com a causa.** Medido em 2026-09-14, na
//! cena do pincel de contorno (`=42`): uma meia esfera recortada de uma
//! `uv_sphere(32, 48)` ficava com `1 490` posições para `768` faces — **`721`
//! órfãos** — e o cursor da cena, que é *o ponto mais alto da peça*, caía no
//! **pólo norte da metade deitada fora**, a `1,0` da boca. A busca da âncora
//! devolvia esse órfão, ele não tem aresta nenhuma, logo não é vértice de
//! borda, e a lei recusava o traço inteiro com `SemBordaAoAlcance`. *A queixa
//! apontava para o verbo e a causa era a fixtura.*
//!
//! A mesma lei já estava escrita — e com o mecanismo no doc — dentro do
//! importador de OBJ, onde ela nasceu (um arquivo de dez objectos daria a cada
//! peça os vértices das outras nove). Estava escrita **num sítio só e privada**,
//! que é exactamente como o segundo consumidor a reescreve mal.

use crate::Face;

/// As posições que `faces` de facto usa, as faces reindexadas, e **de onde cada
/// posição nova veio**.
///
/// A terceira saída é o que torna isto uma porta em vez de um utilitário: todo
/// canal PARALELO à malha (cor, máscara, um vetor por vértice de quem chama)
/// compacta-se com um `map` sobre ela, em vez de uma segunda cópia desta
/// aritmética — e duas cópias é como os dois lados passam a discordar sobre
/// quem sobrou.
///
/// ⚠️ **A ordem é a do PRIMEIRO uso**, percorrendo `faces` na ordem dada: ela é
/// determinística e independe do pool de entrada, então recortar a mesma peça
/// de duas malhas diferentes dá a mesma numeração.
///
/// ⚠️ **A sentinela [`crate::face::TRI`] fica de fora do remapeamento** — o 4.º
/// slot de um triângulo é `u32::MAX`, e remapeá-la transformaria o triângulo
/// num quad com um canto que não existe. É o mesmo motivo por que
/// [`Face::verts`] existe.
#[must_use]
pub fn compact_for_faces(
    positions: &[[f32; 3]],
    faces: &[Face],
) -> (Vec<[f32; 3]>, Vec<Face>, Vec<u32>) {
    let mut remap = vec![u32::MAX; positions.len()];
    let mut verts = Vec::new();
    let mut origem = Vec::new();
    let mut out = Vec::with_capacity(faces.len());
    for f in faces {
        let n = f.verts().len();
        let mut mapped = f.0;
        for slot in &mut mapped[..n] {
            let old = *slot as usize;
            if remap[old] == u32::MAX {
                remap[old] = u32::try_from(verts.len()).unwrap_or(u32::MAX);
                verts.push(positions[old]);
                origem.push(*slot);
            }
            *slot = remap[old];
        }
        out.push(Face(mapped));
    }
    (verts, out, origem)
}

#[cfg(test)]
#[path = "compact_tests.rs"]
mod tests;
