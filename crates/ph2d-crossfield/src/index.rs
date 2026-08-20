//! **O ÍNDICE DE SINGULARIDADE** — onde a cruz não fecha, e quanto.
//!
//! ⚠️ **Esta é a régua do F2, e ela mede o CAMPO, não a malha de saída.** Todas
//! as medições anteriores desta linha contaram vértices irregulares da malha
//! extraída — que é o campo **mais** o extrator. Um campo bom lido por um
//! extrator local ainda dá números ruins, e um número ruim não diz de quem é a
//! culpa. O índice sai do campo direto.
//!
//! # A lei, e a invariante que a prova
//!
//! Dando uma volta em torno de um vértice interior, a cruz é transportada de face
//! em face. O total acumulado — os `κ` do transporte mais os quartos de volta dos
//! `p` — tem de ser um múltiplo de `π/2`, e esse múltiplo é o **índice**.
//!
//! ⭐ **`Σ_v índice_v = 4·χ`** (Poincaré–Hopf para um campo 4-RoSy). Numa esfera
//! isso é **8**; num toro, **0**. É uma invariante **topológica**: ela não depende
//! do solver, da malha ou dos pesos, e é por isso que ela é o gate — um campo que
//! a viole está errado, e nenhuma inspeção visual diria isso.

use std::collections::BTreeMap;

use ph2d_mesh::Mesh;

use crate::{CrossField, Dual, QUARTER};

/// **O ÍNDICE de cada vértice**, em quartos de volta.
///
/// `0` é um vértice regular. Numa grade de quads, `+1` é uma singularidade de
/// valência 3 e `−1` uma de valência 5.
///
/// ⚠️ **Vértices de BORDA devolvem `0`**, e é uma omissão declarada: a lei de
/// Poincaré–Hopf acima vale para superfícies fechadas, e o índice de borda pede
/// a condição de contorno que o F3 (features) vai trazer.
#[must_use]
pub fn vertex_index(mesh: &Mesh, dual: &Dual, field: &CrossField) -> Vec<i32> {
    let faces = mesh.faces();
    let adj = mesh.adjacency();

    // Aresta da malha -> aresta dual, para achar o `κ`/`p` de cada passo do anel.
    let mut of_edge: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for (e, de) in dual.edges().iter().enumerate() {
        if let Some(key) = shared_edge(faces, de.f as usize, de.g as usize) {
            of_edge.insert(key, e);
        }
    }

    (0..mesh.vert_count())
        .map(|v| {
            if adj.is_border(v) {
                return 0;
            }
            let ring = adj.vert_faces.neighbours(v);
            if ring.len() < 3 {
                return 0;
            }
            let Some(order) = ordered_ring(faces, v as u32, ring) else {
                // ⚠️ Um anel que não fecha é um vértice não-manifold. Devolver
                // `0` seria afirmar que ele é regular; devolver o índice de um
                // anel parcial seria pior. O F1 (sanitização) é quem o remove.
                return 0;
            };
            let mut total = 0.0f64;
            for i in 0..order.len() {
                let (f, g) = (order[i], order[(i + 1) % order.len()]);
                let Some(key) = shared_edge(faces, f as usize, g as usize) else {
                    return 0;
                };
                let Some(&e) = of_edge.get(&key) else {
                    return 0;
                };
                let de = &dual.edges()[e];
                // ⚠️ **O SINAL depende do sentido em que o anel atravessa a
                // aresta dual.** `κ` foi medido como `α_g − α_f` para o par
                // `(de.f, de.g)`; percorrer de `g` para `f` é a volta contrária,
                // e somar o mesmo número seria contar o transporte ao contrário.
                let forward = de.f == f && de.g == g;
                let step = f64::from(de.kappa) + f64::from(QUARTER) * f64::from(field.period(e));
                total += if forward { step } else { -step };
            }
            // O total é um múltiplo de π/2 a menos de erro numérico.
            (total / f64::from(QUARTER)).round() as i32
        })
        .collect()
}

/// **QUANTAS singularidades e a SOMA dos índices** — `(quantas, soma)`.
///
/// ⚠️ **A soma é a invariante e a contagem é o produto.** Duas malhas podem ter a
/// mesma soma (obrigatoriamente, se têm o mesmo gênero) e uma delas ter **oito**
/// singularidades e a outra **duzentas**, em pares `+1/−1` que se cancelam. É a
/// contagem que o artista vê.
#[must_use]
pub fn singularities(mesh: &Mesh, dual: &Dual, field: &CrossField) -> (usize, i32) {
    let idx = vertex_index(mesh, dual, field);
    let count = idx.iter().filter(|k| **k != 0).count();
    let sum = idx.iter().sum();
    (count, sum)
}

/// A aresta que duas faces partilham, como par ordenado `(menor, maior)`.
fn shared_edge(faces: &[ph2d_mesh::Face], f: usize, g: usize) -> Option<(u32, u32)> {
    let (a, b) = (faces[f].verts(), faces[g].verts());
    let mut common: Vec<u32> = a.iter().copied().filter(|x| b.contains(x)).collect();
    if common.len() != 2 {
        return None;
    }
    common.sort_unstable();
    Some((common[0], common[1]))
}

/// **AS FACES DO ANEL, EM ORDEM** — o passeio que dá a volta ao vértice.
///
/// ⚠️ **Sem a ordem não há índice.** O transporte é um produto de rotações, e um
/// produto só é o mesmo em qualquer ordem se as rotações comutarem — que é
/// exatamente o que não acontece aqui. Somar os `κ` do conjunto de faces
/// incidentes, em ordem de índice, devolveria um número que parece um índice e
/// não é.
fn ordered_ring(faces: &[ph2d_mesh::Face], v: u32, ring: &[u32]) -> Option<Vec<u32>> {
    // Para cada face do anel, as duas arestas que saem de `v`.
    let spoke = |f: u32| -> Option<(u32, u32)> {
        let t = faces[f as usize].verts();
        let k = t.iter().position(|x| *x == v)?;
        Some((t[(k + 1) % t.len()], t[(k + t.len() - 1) % t.len()]))
    };
    // raio -> as faces que o usam.
    let mut by_spoke: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for &f in ring {
        let (a, b) = spoke(f)?;
        by_spoke.entry(a).or_default().push(f);
        by_spoke.entry(b).or_default().push(f);
    }
    if by_spoke.values().any(|v| v.len() != 2) {
        return None;
    }

    let mut out = Vec::with_capacity(ring.len());
    let start = ring[0];
    // ⚠️ **O SENTIDO do passeio é o da FACE, e não uma escolha.** `spoke` devolve
    // `(seguinte, anterior)` na ordem em que a face lista os vértices, que é
    // anti-horária vista de FORA. Entrar pelo `anterior` faz o primeiro passo
    // sair pelo `seguinte` — a volta no sentido da orientação da superfície.
    //
    // ⚠️ **A primeira versão entrava pelo `seguinte` e dava a volta ao contrário**:
    // a soma dos índices saía `−8` numa esfera onde a topologia exige `+8`. O
    // módulo da resposta estava certo e o sinal não, que é a assinatura exata de
    // um passeio invertido — e nenhuma inspeção do campo teria mostrado isso.
    let (_, mut prev_spoke) = spoke(start)?;
    let mut cur = start;
    for _ in 0..ring.len() {
        out.push(cur);
        let (a, b) = spoke(cur)?;
        // Sai pelo raio que NÃO foi por onde se entrou.
        let next_spoke = if a == prev_spoke { b } else { a };
        let pair = by_spoke.get(&next_spoke)?;
        let next = if pair[0] == cur { pair[1] } else { pair[0] };
        prev_spoke = next_spoke;
        cur = next;
    }
    // O anel tem de fechar exatamente no ponto de partida.
    if cur != start || out.len() != ring.len() {
        return None;
    }
    Some(out)
}
