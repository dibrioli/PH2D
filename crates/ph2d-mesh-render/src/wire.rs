//! **O WIREFRAME** — a lista de arestas que o passe de linhas desenha.
//!
//! # Por que uma lista de LINHAS e não `PolygonMode::Line`
//!
//! O wgpu oferece `PolygonMode::Line`, e ele seria uma linha de código — mas é
//! uma **feature OPCIONAL do device** (`POLYGON_MODE_LINE`). Um pipeline que a
//! pede num adaptador que não a tem **falha na criação**, e o modo de falha é o
//! pior possível: o app abre, o artista marca a caixa, e nada acontece. Uma
//! `LineList` é topologia de base — existe em todo backend, inclusive nos que a
//! próxima máquina do Enio possa ter.
//!
//! # Por que o anel de vizinhos, e não um passe sobre as faces
//!
//! Varrer as faces emitiria **cada aresta interior duas vezes** (as duas faces
//! que a compartilham) e exigiria um passe de dedup. O anel de vizinhos já
//! responde a pergunta sem trabalho nenhum: uma aresta `{a, b}` é emitida pelo
//! lado de índice MENOR, então `u > v` a torna única por construção. É a mesma
//! regra de posse que o [`ph2d_mesh::EdgeIds`] usa para numerar arestas — e a
//! numeração dele **não é pedida aqui de propósito**: desenhar não precisa de
//! id, precisa de par, e construir o grafo inteiro custaria 89% a mais para
//! responder uma pergunta que ninguém faz.
//!
//! # O preço, medido em vez de estimado
//!
//! Numa superfície fechada `2E = Σ grau ≥ 3F`, então `E ≤ 3V − 6` e a lista
//! custa **no máximo 24 bytes por vértice** (dois `u32` por aresta) — o teto é
//! da malha toda triangular, e uma com quads paga menos. A esfera do smoke
//! (6050 vértices) paga 145 KB; um milhão de vértices pagaria 24 MB. É por isso
//! que
//! ela é construída **sob demanda** — ver [`crate::MeshRenderer::upload_wire_at`]:
//! um artista com o wireframe desligado não paga um byte, e é assim que a
//! maioria esculpe.

use ph2d_mesh::Mesh;

/// Escreve em `out` os pares de índices de **cada aresta única** da malha.
///
/// ⚠️ Ela **limpa** `out` — é um rascunho reusado entre chamadas, e devolver a
/// lista por valor faria uma alocação por rebuild de topologia.
pub fn wire_indices(mesh: &Mesh, out: &mut Vec<u32>) {
    out.clear();
    let ring = &mesh.adjacency().vert_verts;
    for v in 0..ring.len() {
        let a = u32::try_from(v).unwrap_or(u32::MAX);
        for &b in ring.neighbours(v) {
            // A posse pelo índice menor: sem ela, toda aresta interior sai
            // duas vezes e o device desenha o dobro das linhas pelo dobro do
            // preço, com o mesmo desenho na tela.
            if b > a {
                out.push(a);
                out.push(b);
            }
        }
    }
}

#[cfg(test)]
#[path = "wire_tests.rs"]
mod tests;
