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
//! `p` — **mais o defeito angular `K_v`** é um múltiplo exato de `π/2`, e esse
//! múltiplo é o **índice**:
//!
//! ```text
//! índice_v = ( Σ_anel ±(κ_e + (π/2)·p_e)  +  K_v ) / (π/2),   K_v = 2π − Σ ângulos
//! ```
//!
//! ⚠️ **O `K_v` FALTAVA, e ele só se esconde em malha bem distribuída** (achado de
//! 2026-08-21). Os `κ` medem a rotação da MOLDURA de face para face; dar a volta
//! ao vértice roda a moldura pela holonomia da superfície, que é exatamente `K_v`.
//! Somar só os `κ` mede o transporte **mais** essa rotação geométrica, e a soma
//! deixa de ser um múltiplo de `π/2`. Medido, o pior resíduo do arredondamento:
//!
//! | malha | sem `K_v` | ambíguos | **com `K_v`** |
//! |---|---|---|---|
//! | `uv_sphere(96,144)` | 0,0009 | 0 | **0,0001** |
//! | `torus(32,16)` | 0,049 | 0 | **0,0000** |
//! | ⛔ `sphere_shuffled` | **0,4999** | **1 468** | **0,0001** |
//! | ⛔ `sphere_noisy` | **0,5000** | **4 472** | **0,0000** |
//!
//! ⭐ **`0,5` é um empate — o `round` a decidir por sorteio**, e era isso que
//! fazia a soma sair `−147` numa esfera. *Numa malha uniforme `K_v ≈ 4π/N` é
//! minúsculo, e o erro passa por ruído numérico durante todo o desenvolvimento.*
//!
//! ⭐ **`Σ_v índice_v = 4·χ`** (Poincaré–Hopf para um campo 4-RoSy). Numa esfera
//! isso é **8**; num toro, **0**. É uma invariante **topológica**: ela não depende
//! do solver, da malha ou dos pesos, e é por isso que ela é o gate — um campo que
//! a viole está errado, e nenhuma inspeção visual diria isso.
//!
//! ⚠️ **E ela agora é EXATA e não aproximada**, por duas parcelas que se fecham:
//! cada aresta dual é percorrida por dois anéis em sentidos opostos (logo
//! `Σ_v total_v = 0`), e `Σ_v K_v = 2π·χ` por Gauss–Bonnet. *Só com o `K_v` é que
//! a invariante passa a ser uma identidade em vez de uma coincidência.*

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
    vertex_index_with_report(mesh, dual, field).0
}

/// **O que a régua não conseguiu medir, e quão longe de um inteiro ela ficou.**
///
/// ⚠️ **Ele existe porque a régua tinha QUATRO `return 0` silenciosos**, e um
/// vértice cujo índice não se sabe calcular estava a ser afirmado **regular**
/// (2026-08-21). O sintoma era a soma dos índices sair `−147` em `sphere_shuffled`
/// onde Poincaré–Hopf exige `+8` — e nada dizia que a conta tinha desistido.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct IndexReport {
    /// Vértices de borda — **omissão declarada**, não defeito.
    pub border: usize,
    /// Anel com menos de três faces: não há volta a dar.
    pub thin: usize,
    /// ⛔ O anel não fecha — vértice não-manifold.
    pub open_ring: usize,
    /// ⛔ Duas faces consecutivas do anel não deram aresta dual: a malha diz uma
    /// coisa e o grafo dual diz outra.
    pub lost_edge: usize,
    /// ⛔ **Duas arestas duais com a MESMA chave de aresta de malha.** Só acontece
    /// numa aresta com três ou mais faces, e a segunda **apaga** a primeira do
    /// mapa — a partir daí o anel lê o `κ` da face errada.
    pub key_collisions: usize,
    /// ⭐ **O pior resíduo do arredondamento**, em quartos de volta. O índice sai
    /// de `round(total / (π/2))`, e este número diz quanto o `round` teve de
    /// inventar. `0,5` é um empate — a régua a decidir por sorteio.
    pub worst_residual: f64,
    /// Quantos vértices ficaram com resíduo acima de `0,25` — ou seja, mais perto
    /// do meio do caminho do que de um inteiro.
    pub ambiguous: usize,
}

impl IndexReport {
    /// Quantos vértices a régua **desistiu** de medir (sem contar a borda, que é
    /// omissão declarada) — cada um deles entra na soma como se fosse regular.
    #[must_use]
    pub fn gave_up(&self) -> usize {
        self.thin + self.open_ring + self.lost_edge
    }
}

/// **O TOTAL TRANSPORTADO em volta de cada vértice, cru e sem arredondar.**
///
/// ⚠️ **É diagnóstico, e existe para não haver uma segunda cópia do passeio.**
/// Vértices que a régua não consegue medir devolvem `0,0` — quem quiser saber
/// quais é o [`IndexReport`] que responde.
#[must_use]
pub fn ring_totals(mesh: &Mesh, dual: &Dual, field: &CrossField) -> Vec<f64> {
    vertex_index_full(mesh, dual, field).2
}

/// **O índice, mais o relatório do que a régua não conseguiu medir.**
#[must_use]
pub fn vertex_index_with_report(
    mesh: &Mesh,
    dual: &Dual,
    field: &CrossField,
) -> (Vec<i32>, IndexReport) {
    let (idx, rep, _) = vertex_index_full(mesh, dual, field);
    (idx, rep)
}

#[allow(clippy::missing_panics_doc)]
fn vertex_index_full(
    mesh: &Mesh,
    dual: &Dual,
    field: &CrossField,
) -> (Vec<i32>, IndexReport, Vec<f64>) {
    let faces = mesh.faces();
    let adj = mesh.adjacency();
    let mut rep = IndexReport::default();
    let mut totals = Vec::with_capacity(mesh.vert_count());
    let defect = angle_defects(mesh);

    // Aresta da malha -> aresta dual, para achar o `κ`/`p` de cada passo do anel.
    let mut of_edge: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for (e, de) in dual.edges().iter().enumerate() {
        if let Some(key) = shared_edge(faces, de.f as usize, de.g as usize)
            && of_edge.insert(key, e).is_some()
        {
            rep.key_collisions += 1;
        }
    }

    let mut out = Vec::with_capacity(mesh.vert_count());
    for (v, &k_v) in defect.iter().enumerate() {
        if adj.is_border(v) {
            rep.border += 1;
            out.push(0);
            totals.push(0.0);
            continue;
        }
        let ring = adj.vert_faces.neighbours(v);
        if ring.len() < 3 {
            rep.thin += 1;
            out.push(0);
            totals.push(0.0);
            continue;
        }
        let Some(order) = ordered_ring(faces, u32::try_from(v).unwrap_or(u32::MAX), ring) else {
            // ⚠️ Um anel que não fecha é um vértice não-manifold. Devolver
            // `0` seria afirmar que ele é regular; devolver o índice de um
            // anel parcial seria pior. O F1 (sanitização) é quem o remove.
            rep.open_ring += 1;
            out.push(0);
            totals.push(0.0);
            continue;
        };
        let mut total = 0.0f64;
        let mut lost = false;
        for i in 0..order.len() {
            let (f, g) = (order[i], order[(i + 1) % order.len()]);
            let Some(e) = shared_edge(faces, f as usize, g as usize).and_then(|k| of_edge.get(&k))
            else {
                lost = true;
                break;
            };
            let de = &dual.edges()[*e];
            // ⚠️ **O SINAL depende do sentido em que o anel atravessa a
            // aresta dual.** `κ` foi medido como `α_g − α_f` para o par
            // `(de.f, de.g)`; percorrer de `g` para `f` é a volta contrária,
            // e somar o mesmo número seria contar o transporte ao contrário.
            let forward = de.f == f && de.g == g;
            let step = f64::from(de.kappa) + f64::from(QUARTER) * f64::from(field.period(*e));
            total += if forward { step } else { -step };
        }
        if lost {
            rep.lost_edge += 1;
            out.push(0);
            totals.push(0.0);
            continue;
        }
        // ⭐ **`+ K_v` é o que torna isto um múltiplo EXATO de π/2** — ver o doc
        // do módulo, com a tabela que o mediu. O resíduo diz quanto o `round`
        // ainda teve de engolir, e ele agora é ruído de `f64` em toda fixtura.
        let quarters = (total + k_v) / f64::from(QUARTER);
        let residual = (quarters - quarters.round()).abs();
        rep.worst_residual = rep.worst_residual.max(residual);
        if residual > 0.25 {
            rep.ambiguous += 1;
        }
        out.push(quarters.round() as i32);
        totals.push(total);
    }
    (out, rep, totals)
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

/// **O DEFEITO ANGULAR de cada vértice** — `K_v = 2π − Σ(ângulos incidentes)`.
///
/// ⭐ É a holonomia da superfície em torno do vértice: quanto uma moldura roda
/// por dar a volta. Sem ele o [`vertex_index`] mede o transporte **mais** essa
/// rotação, e o resultado deixa de ser um múltiplo de `π/2`.
///
/// ⚠️ **O controle é Gauss–Bonnet: `Σ_v K_v = 2π·χ`** — `4π` numa esfera, `0` num
/// toro. Medido sobre as seis fixturas da sonda `index_formula`, dá `2,000` e
/// `0,000` de `χ` ao terceiro decimal. *Uma soma que já tem oráculo não precisa de
/// gate novo: ela É o gate.*
///
/// ⚠️ Em `f64` de propósito: em `f32` a subtração de `2π` por uma soma de seis a
/// nove ângulos perde os dígitos que decidem o arredondamento.
fn angle_defects(mesh: &Mesh) -> Vec<f64> {
    let pos = mesh.positions();
    let mut k = vec![f64::from(core::f32::consts::TAU); mesh.vert_count()];
    for f in mesh.faces() {
        let v = f.verts();
        for i in 0..v.len() {
            let (o, a, b) = (
                pos[v[i] as usize],
                pos[v[(i + 1) % v.len()] as usize],
                pos[v[(i + v.len() - 1) % v.len()] as usize],
            );
            let (u, w) = (sub3(a, o), sub3(b, o));
            let (lu, lw) = (norm3(u), norm3(w));
            if lu < 1.0e-12 || lw < 1.0e-12 {
                continue;
            }
            k[v[i] as usize] -= f64::from((dot3(u, w) / (lu * lw)).clamp(-1.0, 1.0).acos());
        }
    }
    k
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn norm3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}
