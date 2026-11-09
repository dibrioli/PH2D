//! **O grupo carrega o rig** (W-JG) — arrastar um corpo em repouso move o
//! conjunto articulado a que ele pertence.
//!
//! História e porquê: `git log` deste ficheiro.

use ph2d_ecs::{ChildOf, Entity, SimWorld, Transform};

use ph2d_editor_core::GroupDragSnapshot;

/// Preenche `out` (o `App::group_drag_starts`) para um gizmo drag que acabou de
/// abrir: os OUTROS selecionados e, se `carry_rig`, o rig articulado do
/// conjunto.
///
/// `selected` é a seleção inteira (o `primary_bits` pode estar dentro dela e é
/// pulado — o snapshot dele vive no `GizmoDragState`).
///
/// **Uma porta, dois sítios de Down** (a alça do gizmo e o pick de canvas):
/// duas cópias desta regra é como arrastar pela alça passaria a carregar o rig
/// e arrastar pelo corpo, não.
pub fn seed_group_drag_starts(
    out: &mut Vec<GroupDragSnapshot>,
    sim: &mut SimWorld,
    primary_bits: u64,
    selected: &[u64],
    reach: Option<ph2d_physics_ecs::DragReach>,
) {
    out.clear();
    // A regra de sempre: todo OUTRO selecionado que tenha `Transform`.
    for &sel in selected {
        if sel == primary_bits {
            continue;
        }
        if let Some(s) = snapshot_of(sim, sel) {
            out.push(s);
        }
    }
    let Some(reach) = reach else {
        return;
    };
    let mut seed: Vec<Entity> = Vec::with_capacity(out.len() + 1);
    seed.push(Entity::from_bits(primary_bits));
    seed.extend(out.iter().map(|s| Entity::from_bits(s.entity_bits)));
    // ⚠️ **A política vem de FORA e é resolvida numa porta só**
    // (`jointed_by`): `Whole` leva o rig inteiro, âncoras inclusas; `Dynamic`
    // deixa Static/Kinematic onde estão. Escolher aqui com um `match` seria a
    // terceira cópia da pergunta *quem conduz?*, e é ela que apodrece.
    let group = ph2d_physics_ecs::jointed_by(sim.world_mut(), &seed, reach);
    // O conjunto que VAI se mover, decidido ANTES de empurrar qualquer coisa —
    // sem isso a regra de parentesco dependeria da ordem em que os candidatos
    // aparecem, e a ordem de `jointed_rig` é por bits (id de alocação), que
    // não tem relação com a árvore.
    let candidates: Vec<u64> = seed
        .iter()
        .chain(group.iter())
        .map(|e| e.to_bits())
        .collect();
    for e in group {
        let bits = e.to_bits();
        if bits == primary_bits || out.iter().any(|s| s.entity_bits == bits) {
            continue;
        }
        if shares_a_chain(sim, e, &candidates) {
            continue;
        }
        if let Some(s) = snapshot_of(sim, bits) {
            out.push(s);
        }
    }
}

/// O snapshot de início que o `advance_gizmo_drag` consome, ou `None` se a
/// entidade não tem `Transform` (nada a mover).
fn snapshot_of(sim: &SimWorld, bits: u64) -> Option<GroupDragSnapshot> {
    let e = Entity::from_bits(bits);
    let t = sim.world().get::<Transform>(e)?;
    let start_transform = ph2d_editor_core::TransformSnapshot {
        translation: [t.translation.x, t.translation.y],
        rotation: t.rotation,
        scale: [t.scale.x, t.scale.y],
    };
    let pw = ph2d_ecs::parent_world_transform(sim.world(), e);
    Some(GroupDragSnapshot {
        entity_bits: bits,
        start_transform,
        parent_world: ph2d_editor_core::TransformSnapshot {
            translation: [pw.translation.x, pw.translation.y],
            rotation: pw.rotation,
            scale: [pw.scale.x, pw.scale.y],
        },
    })
}

/// `e` é parente (ancestral OU descendente) de algum outro membro de
/// `candidates`?
///
/// Simétrico de propósito: as duas direções são o MESMO defeito visto dos dois
/// lados (o de baixo anda duas vezes), então perguntar só para cima deixaria
/// passar o caso em que o corpo arrastado é o descendente.
fn shares_a_chain(sim: &SimWorld, e: Entity, candidates: &[u64]) -> bool {
    // Para cima, a partir de `e`.
    if let Some(a) = first_ancestor_in(sim, e, candidates) {
        debug_assert_ne!(a, e);
        return true;
    }
    // E para baixo: `e` é ancestral de algum outro candidato? Perguntado
    // subindo a partir de cada um deles — a árvore só tem ponteiro para cima,
    // e o número de candidatos é o tamanho de um rig.
    candidates.iter().any(|&other| {
        other != e.to_bits()
            && first_ancestor_in(sim, Entity::from_bits(other), &[e.to_bits()]).is_some()
    })
}

/// O primeiro ancestral ESTRITO de `e` que está em `set`, se houver.
fn first_ancestor_in(sim: &SimWorld, e: Entity, set: &[u64]) -> Option<Entity> {
    let mut cur = sim.world().get::<ChildOf>(e).map(ChildOf::parent);
    while let Some(p) = cur {
        if set.contains(&p.to_bits()) {
            return Some(p);
        }
        cur = sim.world().get::<ChildOf>(p).map(ChildOf::parent);
    }
    None
}

#[cfg(test)]
#[path = "joint_rig_drag_tests.rs"]
mod tests;
