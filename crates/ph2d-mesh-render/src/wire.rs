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
    wire_indices_com(mesh, false, out);
}

/// ⭐⭐⭐⭐ **O mesmo, podendo esconder a DIAGONAL de cada triângulo** — a vista
/// que faz uma grade APARECER.
///
/// # ⛔⛔⛔⛔ Porque ela existe: o dono não conseguia ver o que a medição via
///
/// A wave do pente de topologia foi reprovada com *«não percebo nenhuma
/// vantagem visualmente»* **enquanto a régua lia fileiras de `10`–`23` arestas
/// contra `2`–`4` do controlo**. Desenhada a grandeza sozinha (as arestas
/// alinhadas a preto, as outras a cinzento), as fileiras atravessam o traço
/// inteiro — *elas estavam lá e ninguém as via*, porque numa malha triangulada
/// elas são **uma família de arestas entre três**, e o arame desenha as três com
/// o mesmo traço.
///
/// ⚠️ **Eu próprio as li como «retalhos»** olhando o arame cheio, e escrevi isso
/// num handoff e num report ao dono. *Uma leitura a olho de um arame cheio não é
/// uma medição* — e é isso que esta vista corrige.
///
/// # A lei, e porque ela é INTRÍNSECA
///
/// Uma grade quadrada triangulada tem lados `ρ`, `ρ` e `ρ√2`: **a diagonal é a
/// aresta mais longa do triângulo**. Esconder a mais longa devolve os quadrados.
///
/// ⭐ E ela não pergunta nada ao pincel — nem direcção de traço, nem campo, nem
/// estado de gesto. *Onde há grade ela mostra a grade; onde não há, mostra
/// ruído, que é a resposta certa.*
///
/// ⚠️⚠️ **Só esconde a aresta que é a mais longa nos DOIS triângulos dela**, e a
/// cerca é o que separa esta vista de um arame rasgado: com um só lado a decidir,
/// uma aresta partilhada por um triângulo esguio e um gordo desaparecia e abria
/// um buraco na malha. ⛔ E uma aresta com **um** triângulo só (a beira de uma
/// peça aberta) **nunca** se esconde — ali ela é a silhueta.
///
/// ⚠️ **Uma malha de QUADS não muda nada**, por construção: a diagonal de um
/// quadrilátero não é uma aresta, logo não está na lista de onde se tira.
pub fn wire_indices_com(mesh: &Mesh, so_a_grade: bool, out: &mut Vec<u32>) {
    out.clear();
    let escondida = if so_a_grade {
        diagonais(mesh)
    } else {
        std::collections::BTreeSet::new()
    };
    let ring = &mesh.adjacency().vert_verts;
    for v in 0..ring.len() {
        let a = u32::try_from(v).unwrap_or(u32::MAX);
        for &b in ring.neighbours(v) {
            // A posse pelo índice menor: sem ela, toda aresta interior sai
            // duas vezes e o device desenha o dobro das linhas pelo dobro do
            // preço, com o mesmo desenho na tela.
            if b > a {
                if escondida.contains(&(a, b)) {
                    continue;
                }
                out.push(a);
                out.push(b);
            }
        }
    }
}

/// As arestas que são a **mais longa** de TODOS os triângulos que as contêm, e
/// que têm mais do que um.
fn diagonais(mesh: &Mesh) -> std::collections::BTreeSet<(u32, u32)> {
    let p = mesh.positions();
    // Por aresta: quantos triângulos a contêm, e em quantos ela é a mais longa.
    let mut conta: std::collections::BTreeMap<(u32, u32), (u32, u32)> =
        std::collections::BTreeMap::new();
    for f in mesh.faces() {
        for t in 0..f.tri_count() {
            let tri = f.tri_at(t);
            let lado = |i: usize| -> f32 {
                let (a, b) = (p[tri[i] as usize], p[tri[(i + 1) % 3] as usize]);
                let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2]))
            };
            let mut maior = 0usize;
            for i in 1..3 {
                if lado(i) > lado(maior) {
                    maior = i;
                }
            }
            for i in 0..3 {
                let (a, b) = (tri[i], tri[(i + 1) % 3]);
                let e = conta.entry((a.min(b), a.max(b))).or_insert((0, 0));
                e.0 += 1;
                if i == maior {
                    e.1 += 1;
                }
            }
        }
    }
    conta
        .into_iter()
        .filter(|(_, (faces, maiores))| *faces > 1 && faces == maiores)
        .map(|(k, _)| k)
        .collect()
}

#[cfg(test)]
#[path = "wire_tests.rs"]
mod tests;
