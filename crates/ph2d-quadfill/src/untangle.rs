//! ⭐⭐⭐ **DESFAZER AS GRAVATAS** — o quad que se auto-cruza, endireitado no sítio.
//!
//! # ⛔⛔⛔ Por que ele existe: UMA gravata deitou fora a melhor candidata que esta peça já teve
//!
//! Medido em 2026-09-03 na escultura do dono (`_base_sculpt`, `Detail 1`, com a calota da fase
//! zero — [`ph2d_remesh_iso::Cap`]), com as três primeiras chaves do selector impressas pela
//! primeira vez:
//!
//! | candidata | furos | ilhas | **gravatas** | pontas amputadas | grade no bico |
//! |---|---|---|---|---|---|
//! | a que o produto escolheu | `0` | `1` | **`0`** | ⛔ **`3` de `5`** | ⛔ `2,81` |
//! | ⭐ a que ele deitou fora | `0` | `1` | **`1`** | ⭐ **`0` de `5`** | ⭐ `0,81` |
//!
//! A chave das gravatas é a **3.ª** e a da amputação a **4.ª**: uma face dobrada ganha a três
//! pontas cortadas. ⚠️ **E a gravata nem estava na ponta** — a `5,7` células do bico mais
//! próximo, um quad dobrado solto no flanco.
//!
//! ⛔ **A saída NÃO é reordenar as chaves** — a ordem foi medida em 30/08 sobre um report do
//! dono (*«destruiu completamente a malha»*, `125` gravatas), e o doc do
//! `sculpt3d_retopo_extract` já escreve a lei: *«a saída não é reordenar o critério, é produzir
//! a candidata que tem as duas coisas»*. É isso que este módulo faz.
//!
//! ⚠️ **Onde não há gravata, ele é o mapa identidade AO BIT** — e há gate. *Uma cura que toca
//! na malha que já estava boa é uma regressão à espera de um smoke.*

use ph2d_mesh::Mesh;

/// Quantas rondas de relaxação local uma gravata tem para se desfazer.
///
/// ⚠️ **Poucas, e de propósito:** isto move `4` vértices por face acusada, com a cerca de
/// viagem por cima — quem precisa de mais do que isto não tem uma face dobrada, tem uma região
/// mal traçada, e essa é a chave da frente do selector, não esta.
const MAX_ROUNDS: usize = 16;

/// ⭐⭐⭐ **A CERCA DESTA REPARAÇÃO — e ela NÃO é a do acabamento**, de propósito.
///
/// ⚠️ **As duas respondem a perguntas diferentes:** a [`crate::EXTRACT_TRAVEL_RESCUE`] (meia
/// aresta) limita um **deslize global** da grade sobre o relevo; esta limita uma **reparação
/// local** de uma face partida. ⛔ E o número não é escolhido: *um quad só se auto-cruza quando
/// um vértice passa PARA LÁ do vizinho*, logo a viagem de volta é da ordem de **uma** aresta —
/// com meia, a cura é impossível por construção. `2` é isso com folga; acima disso não é uma
/// dobra local, é uma região mal traçada, e essa é a chave da frente do selector.
pub const UNTANGLE_TRAVEL: f32 = 2.0;

/// Meio passo — o mesmo amortecimento do alisamento da casa ([`crate::finish`]): um passo
/// inteiro sobre uma umbrella não é contractivo.
const LAMBDA: f32 = 0.5;

/// ⭐⭐⭐ **Endireita as gravatas de `mesh`, pousando na `surface`** — devolve **quantas
/// desapareceram**.
///
/// `travel` é a cerca de viagem em unidades da aresta mediana (a mesma de
/// [`crate::EXTRACT_TRAVEL`]); `≤ 0` ou não-finito = sem cerca.
///
/// ⚠️ **A aceitação tem DUAS metades**, e cada uma responde a uma pergunta:
///
/// - **desceram?** — o censo é global, logo trocar uma gravata por outra noutro sítio não passa;
/// - **a forma não piorou?** — a mesma lei de [`crate::acceptable`] que a passagem de
///   acabamento já usa. *Duas leis de aceitação seriam duas respostas à mesma pergunta.*
///
/// ⛔ Se qualquer das duas falhar, a malha volta **exactamente** ao que era.
#[must_use]
pub fn untangle_bowties(mesh: &mut Mesh, surface: &Mesh, travel: f32) -> usize {
    let antes = bowtie_count(mesh);
    if antes == 0 {
        return 0;
    }
    let forma_antes = crate::quad_shape(mesh);
    let origin: Vec<[f32; 3]> = mesh.positions().to_vec();
    let moveis = moving_set(mesh);
    if moveis.is_empty() {
        return 0;
    }
    let unit = median_edge(mesh);
    let max_travel = if travel.is_finite() && travel > 0.0 && unit > 0.0 {
        unit * travel
    } else {
        f32::INFINITY
    };
    let seed = crate::finish::bbox_seed(surface);

    let mut melhor = antes;
    let mut best_pos: Option<Vec<[f32; 3]>> = None;
    for _ in 0..MAX_ROUNDS {
        relax_once(mesh, surface, &moveis, &origin, max_travel, seed);
        let agora = bowtie_count(mesh);
        if agora < melhor
            && crate::finish_extract::acceptable(&crate::quad_shape(mesh), &forma_antes)
        {
            melhor = agora;
            best_pos = Some(mesh.positions().to_vec());
        }
        if agora == 0 {
            break;
        }
    }
    match best_pos {
        Some(p) => {
            mesh.positions_mut().copy_from_slice(&p);
            mesh.rebuild();
            antes - melhor
        }
        None => {
            // ⛔ **Repor é incondicional** — a malha de trabalho já foi mexida pelas rondas que
            // não foram aceites, e deixá-la assim entregaria um alisamento que ninguém pediu.
            mesh.positions_mut().copy_from_slice(&origin);
            mesh.rebuild();
            0
        }
    }
}

/// Os vértices das faces acusadas — e **só** eles.
fn moving_set(mesh: &Mesh) -> Vec<u32> {
    let (_, per_face) = crate::local_shape(mesh);
    let mut v: Vec<u32> = Vec::new();
    for (f, d) in mesh.faces().iter().zip(per_face.iter()) {
        if d.kind == crate::QuadKind::Bowtie {
            v.extend_from_slice(f.verts());
        }
    }
    v.sort_unstable();
    v.dedup();
    v
}

fn bowtie_count(mesh: &Mesh) -> usize {
    crate::local_shape(mesh).0.bowties
}

fn median_edge(mesh: &Mesh) -> f32 {
    let pos = mesh.positions();
    let mut e: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            e.push(dist(a, b));
        }
    }
    if e.is_empty() {
        return 0.0;
    }
    e.sort_by(f32::total_cmp);
    e[e.len() / 2]
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
}

/// Um passo de Laplaciano **tangencial** nos vértices móveis, com reprojeção e cerca.
///
/// ⚠️ **Tangencial pela mesma razão do alisamento da casa:** a componente normal encolhe a peça
/// e a reprojeção a seguir esconde o encolhimento sem o desfazer.
fn relax_once(
    mesh: &mut Mesh,
    surface: &Mesh,
    moveis: &[u32],
    origin: &[[f32; 3]],
    max_travel: f32,
    seed: f32,
) {
    let neighbours: Vec<Vec<u32>> = {
        let adj = mesh.adjacency();
        moveis
            .iter()
            .map(|v| adj.vert_verts.neighbours(*v as usize).to_vec())
            .collect()
    };
    let normals: Vec<[f32; 3]> = mesh.normals().to_vec();
    let mut novos: Vec<(u32, [f32; 3])> = Vec::with_capacity(moveis.len());
    {
        let pos = mesh.positions();
        for (i, &v) in moveis.iter().enumerate() {
            let ns = &neighbours[i];
            if ns.len() < 3 {
                continue;
            }
            let p = pos[v as usize];
            let mut sum = [0.0f32; 3];
            for &w in ns {
                let q = pos[w as usize];
                for k in 0..3 {
                    sum[k] += q[k];
                }
            }
            #[allow(clippy::cast_precision_loss)]
            let inv = 1.0 / ns.len() as f32;
            let d = [
                sum[0].mul_add(inv, -p[0]),
                sum[1].mul_add(inv, -p[1]),
                sum[2].mul_add(inv, -p[2]),
            ];
            let nv = normals[v as usize];
            let along = d[0].mul_add(nv[0], d[1].mul_add(nv[1], d[2] * nv[2]));
            let q = [
                LAMBDA.mul_add(along.mul_add(-nv[0], d[0]), p[0]),
                LAMBDA.mul_add(along.mul_add(-nv[1], d[1]), p[1]),
                LAMBDA.mul_add(along.mul_add(-nv[2], d[2]), p[2]),
            ];
            let q = ph2d_remesh_iso::project_onto(surface, q, seed);
            // ⛔ **Fora da cerca, o vértice fica onde está** — e não «encostado à cerca»: um
            // ponto truncado sai da superfície, e a reprojeção seguinte mediria outra coisa.
            if dist(q, origin[v as usize]) <= max_travel {
                novos.push((v, q));
            }
        }
    }
    {
        let pos = mesh.positions_mut();
        for (v, q) in novos {
            pos[v as usize] = q;
        }
    }
    mesh.rebuild();
}

#[cfg(test)]
#[path = "untangle_tests.rs"]
mod tests;
