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
/// ⭐⭐⭐⭐ **QUANTOS BRAÇOS TEM CADA VÉRTICE NA VISTA DA GRADE** — a régua que o
/// olho do artista usa naquela vista.
///
/// Depois de [`wire_indices_com`] esconder as diagonais, um cruzamento de grade
/// tem **quatro** braços. Os que não têm são as células irregulares, e são elas
/// que o dono chama de *«áreas ainda não muito boas»*.
///
/// ⚠️⚠️ **Ela existe porque a régua vizinha está SATURADA:** a fracção de
/// arestas alinhadas tem tecto `2/3` numa grade perfeita (um terço são
/// diagonais, a `45°`), o produto lê `65,4 %` — `98,1 %` do tecto — e *ler uma
/// fracção sem saber de que* quase gastou uma wave a perseguir dois pontos
/// numa coluna que já não tinha por onde subir.
///
/// ⭐ **Ela deriva da MESMA lista que a vista desenha**, nunca de uma segunda
/// cópia da regra de esconder: se a lei da vista mudar, esta régua muda com ela.
///
/// ⚠️ Quem lê tem de filtrar o que interessa — um vértice na **beira** da
/// pegada tem menos braços por estar na beira, não por ser irregular.
#[must_use]
pub fn bracos_na_vista_da_grade(mesh: &Mesh) -> Vec<u32> {
    let mut arestas = Vec::new();
    wire_indices_com(mesh, true, &mut arestas);
    let mut grau = vec![0u32; mesh.vert_count()];
    for par in arestas.as_chunks::<2>().0 {
        grau[par[0] as usize] += 1;
        grau[par[1] as usize] += 1;
    }
    grau
}

/// que têm mais do que um.
/// ⭐⭐⭐⭐ **QUE ARESTAS A VISTA DA GRADE ESCONDE — um EMPARELHAMENTO, e não um
/// teste aresta a aresta.**
///
/// Esconder uma aresta interior é dizer *«estes dois triângulos são UM
/// quadrado»*, logo a pergunta certa é **quem faz par com quem** — e um par usa
/// os dois triângulos, portanto é um **emparelhamento**, com a restrição de que
/// cada triângulo entra num par só.
///
/// # ⛔⛔ A regra anterior era uma preferência MÚTUA, e ela perdia metade
///
/// Ela escondia uma aresta *iff ela fosse a mais longa dos **DOIS** triângulos*.
/// Isso é um casamento por acordo mútuo, e numa célula **enviesada** ele não
/// acontece: ali a malha partiu o quadrado pela diagonal **CURTA** — que é a
/// escolha CERTA, porque partir pela longa daria triângulos de `25°`–`25°`–`130°`
/// — e a diagonal curta não é a mais longa de ninguém, logo ninguém a escondia.
///
/// **Medido** (fracção de cruzamentos com os quatro braços, quatro rumos):
/// dos `7 %` que a vista não fechava, **`182` tinham a malha com valência `6`,
/// ou seja PERFEITA** — quem falhava era a vista. E a margem não era fina: a
/// candidata perdia por `15 %` na mediana, com só `6,5 %` a menos de `3 %` de
/// fechar. *A malha estava certa; a régua de esconder é que não a sabia ler.*
///
/// # As três regras, medidas
///
/// | regra | com pente | **sem pente** (o controlo) | cintilação |
/// |---|---|---|---|
/// | mútua (a anterior) | `93,01 %` | **`48,29 %`** | `0,86 %` |
/// | **esta** (par, com plausibilidade) | **`96,10 %`** | `66,86 %` | `1,09 %` |
/// | guloso puro (sem a cerca) | `96,18 %` | `72,00 %` | `1,30 %` |
///
/// ⭐ **A CERCA — *«só é candidata quem é a mais longa de PELO MENOS UM dos dois
/// triângulos»* — é o que separa esta do guloso puro:** ela compra a mesma
/// regularidade (`96,10` contra `96,18`) e mantém o **contraste** com a malha
/// por pentear (`66,9` contra `72,0`) e menos cintilação.
///
/// ⚠️⚠️ **E o contraste é uma coluna do produto, não uma vaidade:** a vista
/// existe para o artista ver ONDE a grade dele está. Uma regra que emparelha
/// tudo mostra quadrados também onde não há grade nenhuma, e a diferença que ele
/// procura fica mais fraca. *A que shipa é a mais ambiciosa que ainda deixa os
/// dois lados distinguíveis.*
///
/// ⚠️ **A ordem é por comprimento DECRESCENTE**, e ela é load-bearing **na
/// coluna da CINTILAÇÃO, não na da regularidade** — medido: crescente lê
/// `96,87 %` de regularidade (melhor!) e **`1,44 %`** de cintilação contra os
/// `1,09` desta. ⭐ *A aresta mais longa é a mais parecida com uma diagonal,
/// logo emparelhá-la primeiro é o guloso que respeita o próprio critério* —
/// duas razões a apontar ao mesmo lado.
///
/// ⚠️ O desempate é pela **chave da aresta**: uma vista que mudasse de sorteio
/// piscaria entre quadros.
///
/// ⛔ **Uma aresta com um triângulo só NUNCA se esconde:** ali ela é a
/// silhueta da peça, e escondê-la abre um buraco no arame.
fn diagonais(mesh: &Mesh) -> std::collections::BTreeSet<(u32, u32)> {
    let p = mesh.positions();
    // Por aresta: os triângulos que a contêm, o comprimento, e em quantos deles
    // ela é a mais longa.
    let mut cand: std::collections::BTreeMap<(u32, u32), (Vec<usize>, f64, u32)> =
        std::collections::BTreeMap::new();
    let mut n_tri = 0usize;
    for f in mesh.faces() {
        for t in 0..f.tri_count() {
            let tri = f.tri_at(t);
            let lado2 = |i: usize| -> f32 {
                let (a, b) = (p[tri[i] as usize], p[tri[(i + 1) % 3] as usize]);
                let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2]))
            };
            let mut maior = 0usize;
            for i in 1..3 {
                if lado2(i) > lado2(maior) {
                    maior = i;
                }
            }
            for i in 0..3 {
                let (a, b) = (tri[i], tri[(i + 1) % 3]);
                let e = cand
                    .entry((a.min(b), a.max(b)))
                    .or_insert_with(|| (Vec::new(), f64::from(lado2(i)).sqrt(), 0));
                e.0.push(n_tri);
                if i == maior {
                    e.2 += 1;
                }
            }
            n_tri += 1;
        }
    }

    let mut ordem: Vec<((u32, u32), usize, usize, f64)> = cand
        .into_iter()
        .filter_map(|(k, (tris, l, maiores))| {
            // A cerca da plausibilidade, e a da silhueta.
            (tris.len() == 2 && maiores >= 1).then(|| (k, tris[0], tris[1], l))
        })
        .collect();
    ordem.sort_by(|a, b| b.3.total_cmp(&a.3).then(a.0.cmp(&b.0)));

    let mut gasto = vec![false; n_tri];
    let mut out = std::collections::BTreeSet::new();
    for (k, t0, t1, _) in ordem {
        if gasto[t0] || gasto[t1] {
            continue;
        }
        gasto[t0] = true;
        gasto[t1] = true;
        out.insert(k);
    }
    out
}

#[cfg(test)]
#[path = "wire_tests.rs"]
mod tests;
