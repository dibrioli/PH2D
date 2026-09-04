//! ⭐⭐⭐ **APAGAR AS ABAS** — a cura TOPOLÓGICA do que a relaxação não pode curar.
//!
//! Irmão de [`super`] por RESPONSABILIDADE, não por tamanho: o pai endireita uma face
//! **movendo vértices** (relaxação local, cercada por [`super::UNTANGLE_TRAVEL`]); este
//! **apaga faces e refaz o remendo**. ⚠️ A fronteira entre os dois é uma medição, não uma
//! preferência: *a relaxação NÃO cura uma dobra — ela troca a espécie do defeito* (a
//! gravata sai e a mesma face fica a apontar contra a vizinhança), e foi essa medição que
//! obrigou a existir a cura topológica.

use ph2d_mesh::Mesh;

// ⚠️ Um módulo filho enxerga os itens PRIVADOS do pai — por isso nada aqui precisou de
// subir a visibilidade para caber neste arquivo. O corte não alargou superfície nenhuma.
use super::{GRUPO_MINIMO, defect_count, grupos_dobrados};

/// ⭐⭐⭐ **APAGA AS ABAS e fecha o buraco** — a cura TOPOLÓGICA do que a relaxação não pode curar.
///
/// # ⛔⛔⛔ O que é uma aba, medido na peça do dono (2026-09-03, 3.º report)
///
/// Uma **língua** de faces dobrada para trás sobre si mesma. Retrato do caso dele: `5` faces,
/// `12` vértices, `1,93 h²` de área **ao todo** (um quad normal é `1`), com ângulos de `174°` a
/// `179°` às vizinhas — *a superfície volta atrás*. Três das cinco faces são migalhas
/// (`0,005`–`0,044 h²`).
///
/// ⛔ **Ela nasce na EXTRACÇÃO** (medido com `PH2D_EXTRACT_FINISH=0`: a malha crua já a traz) e a
/// causa é o **mapa dobrar** ali — `134` triângulos dobrados no domínio da peça dele, `4,5 %`.
/// ⛔⛔ E a cura de fundo **já foi medida e recusada**: o solver injectivo do `ph2d-gridmap`
/// zera as dobras do mapa contínuo e foi ele que produziu o *«destruiu completamente a malha»* de
/// 30/08 (ver `injective_solve::enabled`).
///
/// # A operação, e as três condições que a recusam
///
/// Apagar o grupo deixa um buraco cujo bordo é **um** laço; ele é fechado por um **leque** de
/// `L/2` quads à volta de um vértice novo (a lei clássica do polígono de lados pares).
/// ⛔ Recusa-se — e a malha fica **exactamente** como estava — quando:
///
/// 1. o bordo do grupo **não é um laço só** (a aba não é um disco);
/// 2. o laço tem um número **ímpar** de lados (não há leque de quads);
/// 3. o resultado **não melhora**: as faces do avesso têm de descer, sem furos novos, sem ilhas
///    novas e sem mais faces péssimas.
#[must_use]
pub fn remove_flaps(mesh: &mut Mesh, surface: &Mesh) -> usize {
    let mut curadas = 0usize;
    // ⚠️ **Uma aba de cada vez, e recomeçando o censo**: apagar uma muda os índices de face.
    for _ in 0..MAX_FLAPS {
        let Some(grupo) = grupos_dobrados(mesh)
            .into_iter()
            .filter(|g| g.len() >= GRUPO_MINIMO)
            .max_by_key(Vec::len)
        else {
            break;
        };
        // ⭐⭐⭐ **O DISCO É MAIOR QUE A ABA, e a razão é medida:** os vértices emaranhados ficam
        // na BORDA do grupo, logo apagar só as faces dele deixa o buraco com o mesmo contorno
        // torcido e o remendo volta a dobrar (medido: `avesso 2 -> 2`). Crescer um anel põe-nos
        // no INTERIOR do disco, e eles desaparecem com ele.
        let disco = grow_one_ring(mesh, &grupo);
        if !remove_one_flap(mesh, surface, &disco) {
            break;
        }
        curadas += 1;
    }
    curadas
}

/// Quantas abas se tentam apagar numa passagem — ver [`remove_flaps`].
///
/// ⚠️ Poucas de propósito: uma malha com dezenas de abas não tem um defeito local, tem um mapa
/// partido, e essa é a chave da frente do selector, não esta.
const MAX_FLAPS: usize = 8;

/// As faces do grupo **mais** todas as que partilham um vértice com ele — ver o uso.
fn grow_one_ring(mesh: &Mesh, grupo: &[u32]) -> Vec<u32> {
    let vs: std::collections::BTreeSet<u32> = grupo
        .iter()
        .filter_map(|&i| mesh.faces().get(i as usize))
        .flat_map(|f| f.verts().iter().copied())
        .collect();
    let mut fora: Vec<u32> = mesh
        .faces()
        .iter()
        .enumerate()
        .filter(|(_, f)| f.verts().iter().any(|v| vs.contains(v)))
        .map(|(i, _)| u32::try_from(i).unwrap_or(0))
        .collect();
    fora.sort_unstable();
    fora.dedup();
    fora
}

fn remove_one_flap(mesh: &mut Mesh, surface: &Mesh, grupo: &[u32]) -> bool {
    let antes_avesso = defect_count(mesh);
    let antes_forma = crate::quad_shape(mesh);
    let antes_abertas = open_edges(mesh);
    let antes_ilhas = components(mesh);
    let log = std::env::var("PH2D_UNTANGLE_LOG").is_ok();
    let Some(laco) = hole_loop(mesh, grupo) else {
        if log {
            eprintln!(
                "[aba] grupo de {} face(s): o bordo NAO e' um laco so'",
                grupo.len()
            );
        }
        return false;
    };
    if laco.len() < 4 || laco.len() % 2 != 0 {
        if log {
            eprintln!("[aba] laco de {} lados: curto ou impar", laco.len());
        }
        return false;
    }
    let pos = mesh.positions();
    let mut centro = [0.0f32; 3];
    #[allow(clippy::cast_precision_loss)]
    let inv = 1.0 / laco.len() as f32;
    for &v in &laco {
        let p = pos[v as usize];
        for k in 0..3 {
            centro[k] += p[k] * inv;
        }
    }
    let centro = ph2d_remesh_iso::project_onto(surface, centro, crate::finish::bbox_seed(surface));

    let mut novas_pos = pos.to_vec();
    let c = u32::try_from(novas_pos.len()).unwrap_or(0);
    novas_pos.push(centro);
    let fora: std::collections::BTreeSet<u32> = grupo.iter().copied().collect();
    let mut novas_faces: Vec<ph2d_mesh::Face> = mesh
        .faces()
        .iter()
        .enumerate()
        .filter(|(i, _)| !fora.contains(&u32::try_from(*i).unwrap_or(0)))
        .map(|(_, f)| *f)
        .collect();
    for par in 0..laco.len() / 2 {
        let a = laco[2 * par];
        let b = laco[2 * par + 1];
        let d = laco[(2 * par + 2) % laco.len()];
        novas_faces.push(ph2d_mesh::Face::quad(a, b, d, c));
    }
    // ⛔⛔ **Os vértices INTERIORES ao disco ficam órfãos, e órfão não é neutro:** o `χ` conta-os
    // (esta linha já pagou isso — «doze órfãos, doze unidades»). Compactar antes de montar.
    let (novas_pos, novas_faces) = compactar(novas_pos, novas_faces);
    let Ok(candidata) = ph2d_mesh::Mesh::from_parts(novas_pos, novas_faces) else {
        return false;
    };
    // ⛔ **As quatro colunas, e todas têm de dar** — ver o doc de [`remove_flaps`].
    let depois = crate::quad_shape(&candidata);
    if log {
        eprintln!(
            "[aba] laco de {} lados: avesso {antes_avesso} -> {} | abertas {antes_abertas} -> {} | ilhas {antes_ilhas} -> {} | >60 {} -> {}",
            laco.len(),
            defect_count(&candidata),
            open_edges(&candidata),
            components(&candidata),
            antes_forma.skew_over_60,
            depois.skew_over_60,
        );
    }
    // ⭐⭐⭐ **A GUARDA É DO QUE O ARTISTA VÊ, e o tecto das faces feias é o TAMANHO DO REMENDO.**
    //
    // ⛔⛔ Medido (2026-09-03): com `>60` a não poder subir de todo, o remendo que levava as faces
    // do avesso de `2` para **`0`** era recusado porque o leque acrescentava **uma** face com
    // canto pior que `60°`. *Uma dobra é uma fenda preta na foto dele; uma face enviesada é
    // invisível* — e quem pesa a beleza é o selector, uma camada acima, que vê a saída inteira.
    //
    // ⇒ o que se exige aqui é: **o resto da malha não piora** (o tecto é `L/2`, que é o número
    // de faces que o leque acrescenta — no pior caso todas elas são feias) e **nada do que se
    // vê aparece de novo** (faces do avesso, furos, ilhas).
    if defect_count(&candidata) >= antes_avesso
        || open_edges(&candidata) > antes_abertas
        || components(&candidata) > antes_ilhas
        || depois.skew_over_60 > antes_forma.skew_over_60 + laco.len() / 2
    {
        return false;
    }
    *mesh = candidata;
    true
}

/// O bordo do grupo, como **um** laço orientado para o preenchimento — `None` se não for um.
///
/// ⚠️ **A direcção vem da face de FORA**, e não das faces do grupo: elas estão do avesso, logo a
/// volta delas não diz de que lado fica o buraco.
fn hole_loop(mesh: &Mesh, grupo: &[u32]) -> Option<Vec<u32>> {
    let dentro: std::collections::BTreeSet<u32> = grupo.iter().copied().collect();
    let mut por_aresta: std::collections::BTreeMap<(u32, u32), Vec<u32>> =
        std::collections::BTreeMap::new();
    for (i, f) in mesh.faces().iter().enumerate() {
        let i = u32::try_from(i).unwrap_or(0);
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            por_aresta.entry((a.min(b), a.max(b))).or_default().push(i);
        }
    }
    let mut seguinte: std::collections::BTreeMap<u32, u32> = std::collections::BTreeMap::new();
    for (aresta, quem) in &por_aresta {
        let de_dentro = quem.iter().filter(|i| dentro.contains(i)).count();
        if de_dentro != 1 || quem.len() != 2 {
            continue;
        }
        let de_fora = *quem.iter().find(|i| !dentro.contains(i))?;
        let v = mesh.faces()[de_fora as usize].verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            if (a.min(b), a.max(b)) == *aresta {
                // ⭐ O preenchimento percorre ao CONTRÁRIO da face de fora.
                if seguinte.insert(b, a).is_some() {
                    return None;
                }
            }
        }
    }
    if seguinte.len() < 4 {
        return None;
    }
    let inicio = *seguinte.keys().next()?;
    let mut laco = vec![inicio];
    let mut v = *seguinte.get(&inicio)?;
    while v != inicio {
        if laco.len() > seguinte.len() {
            return None;
        }
        laco.push(v);
        v = *seguinte.get(&v)?;
    }
    // ⛔ **Um laço SÓ** — se sobraram arestas de bordo, a aba não é um disco.
    if laco.len() != seguinte.len() {
        return None;
    }
    Some(laco)
}

/// Arestas com uma face só **mais** as não-manifold — a mesma soma que o selector do produto usa.
fn open_edges(mesh: &Mesh) -> usize {
    let mut n: std::collections::BTreeMap<(u32, u32), usize> = std::collections::BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *n.entry((a.min(b), a.max(b))).or_default() += 1;
        }
    }
    n.values().filter(|c| **c != 2).count()
}

/// Quantas peças desligadas a malha tem.
fn components(mesh: &Mesh) -> usize {
    let mut viz: std::collections::BTreeMap<u32, Vec<u32>> = std::collections::BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            viz.entry(a).or_default().push(b);
            viz.entry(b).or_default().push(a);
        }
    }
    let mut vistos: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    let mut n = 0usize;
    for &s in viz.keys() {
        if vistos.contains(&s) {
            continue;
        }
        n += 1;
        let mut pilha = vec![s];
        vistos.insert(s);
        while let Some(u) = pilha.pop() {
            for &w in viz.get(&u).map(Vec::as_slice).unwrap_or(&[]) {
                if vistos.insert(w) {
                    pilha.push(w);
                }
            }
        }
    }
    n
}

/// Deita fora os vértices que nenhuma face usa, e renumera.
fn compactar(
    pos: Vec<[f32; 3]>,
    faces: Vec<ph2d_mesh::Face>,
) -> (Vec<[f32; 3]>, Vec<ph2d_mesh::Face>) {
    let mut mapa = vec![u32::MAX; pos.len()];
    let mut novas_pos: Vec<[f32; 3]> = Vec::with_capacity(pos.len());
    for f in &faces {
        for &v in f.verts() {
            if mapa[v as usize] == u32::MAX {
                mapa[v as usize] = u32::try_from(novas_pos.len()).unwrap_or(0);
                novas_pos.push(pos[v as usize]);
            }
        }
    }
    let novas_faces = faces
        .iter()
        .map(|f| {
            let v = f.verts();
            ph2d_mesh::Face::quad(
                mapa[v[0] as usize],
                mapa[v[1 % v.len()] as usize],
                mapa[v[2 % v.len()] as usize],
                mapa[v[3 % v.len()] as usize],
            )
        })
        .collect();
    (novas_pos, novas_faces)
}
