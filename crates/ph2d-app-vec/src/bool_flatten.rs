//! **Flatten** (F50, dono 02/11: «faça como o Figma») — o comando de UMA vez que torna permanente
//! o que a seleção desenha: cada booleana viva é consolidada e o resto funde-se numa forma só, por
//! `Union` (uma fusão por `EvenOdd` furaria onde as formas se sobrepõem — a tela mudaria no
//! clique). O porquê do desenho: `docs/UI_New_and_Simple/F50_boolean_e_convert_a_figma.md` §4.

use ph2d_ecs::{Entity, SimWorld, VecBoolGroup};
use ph2d_vec_boolean::PathfinderOp;
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::{VecPathId, VecScene, VecXforms};

use crate::bool_live::BoolLive;

/// Achata a seleção do `pen`. Devolve quantos caminhos ficaram selecionados.
pub fn flatten(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    pen: &mut ph2d_vec_edit::PenTool,
    live: &BoolLive,
    xforms: &VecXforms,
) -> usize {
    let sel: Vec<VecPathId> = pen.selected_paths().to_vec();
    let mut groups: Vec<Entity> = Vec::new();
    for id in &sel {
        if let Some(g) = outermost_group(sim, map, live, *id)
            && !groups.contains(&g)
        {
            groups.push(g);
        }
    }
    let mut keep: Vec<VecPathId> = Vec::new();
    // Peças de receitas antigas (MinusBack/Trim/Crop/Merge, códigos ≥ 4): cada uma tem a sua cor,
    // e a Union fundi-las-ia numa só — ficam como estão.
    let mut pieces: Vec<VecPathId> = Vec::new();
    for g in groups {
        if let Some(plan) = live.plan(g) {
            let recipe = crate::bool_gesture::group_op(sim, Some(g)).is_some_and(|op| op >= 4);
            let baked = crate::bool_gesture::bake_ids(sim, scene, plan, g);
            if recipe { &mut pieces } else { &mut keep }.extend(baked);
        }
    }
    for id in sel {
        if !keep.contains(&id) && !pieces.contains(&id) && scene.paths().iter().any(|p| p.id == id)
        {
            keep.push(id);
        }
    }
    pen.select_many(&keep);
    if selected_closed_z(scene, pen).len() >= 2 {
        pathfinder_selection(scene, pen, xforms, PathfinderOp::Union);
    }
    let mut all = pen.selected_paths().to_vec();
    all.extend(pieces);
    pen.select_many(&all);
    all.len()
}

/// A booleana viva mais de FORA que contém `id` e TEM plano — a que a tela desenha (um grupo sem
/// plano não desenha nada próprio; a de dentro é que vale).
fn outermost_group(
    sim: &SimWorld,
    map: &VecEntityMap,
    live: &BoolLive,
    id: VecPathId,
) -> Option<Entity> {
    let w = sim.world();
    let mut cur = Entity::from_bits(*map.get(&id)?);
    let mut found = None;
    for _ in 0..64 {
        let Some(parent) = w.get::<ph2d_ecs::ChildOf>(cur).map(|c| c.parent()) else {
            break;
        };
        if w.get::<VecBoolGroup>(parent).is_some() && live.plan(parent).is_some() {
            found = Some(parent);
        }
        cur = parent;
    }
    found
}

/// Os índices de z (de trás para a frente) dos caminhos FECHADOS selecionados.
fn selected_closed_z(scene: &VecScene, pen: &ph2d_vec_edit::PenTool) -> Vec<usize> {
    let mut zs: Vec<usize> = pen
        .selected_paths()
        .iter()
        .filter_map(|id| scene.paths().iter().position(|p| p.id == *id && p.closed))
        .collect();
    zs.sort_unstable();
    zs.dedup();
    zs
}

/// A booleana DESTRUTIVA `op` sobre os caminhos fechados selecionados: consome os operandos, põe
/// o resultado na fatia de z da base (a de trás) e seleciona-o. Menos de dois fechados ou
/// resultado vazio ⇒ nada muda. Devolve se mudou.
pub fn pathfinder_selection(
    scene: &mut VecScene,
    pen: &mut ph2d_vec_edit::PenTool,
    xforms: &VecXforms,
    op: PathfinderOp,
) -> bool {
    let zs = selected_closed_z(scene, pen);
    if zs.len() < 2 {
        eprintln!("[ph2d-vec] boolean: selecione >= 2 regioes FECHADAS (Shift+clique)");
        return false;
    }
    // ADR-0111: cada operando é assado no MUNDO; o resultado nasce world-space.
    let operands: Vec<ph2d_vec_scene::VecPath> = zs
        .iter()
        .map(|&z| {
            let mut p = scene.paths()[z].clone();
            let x = ph2d_vec_scene::xform_of(xforms, p.id);
            ph2d_vec_scene::bake_xform(&mut p, &x);
            p
        })
        .collect();
    let refs: Vec<&ph2d_vec_scene::VecPath> = operands.iter().collect();
    let results = match ph2d_vec_boolean::pathfinder(&refs, op) {
        Ok(r) if r.is_empty() => {
            eprintln!("[ph2d-vec] pathfinder {op:?}: sem resultado (as formas nao se cruzam?)");
            return false;
        }
        Ok(r) => r,
        Err(e) => {
            eprintln!("[ph2d-vec] pathfinder {op:?}: o motor recusou -- {e}");
            return false;
        }
    };
    let at = zs[0];
    for p in &operands {
        scene.remove_path(p.id);
    }
    let new_ids: Vec<VecPathId> = results
        .into_iter()
        .enumerate()
        .map(|(k, r)| scene.insert_path(at + k, r))
        .collect();
    pen.select_many(&new_ids);
    true
}

#[cfg(test)]
#[path = "bool_flatten_tests.rs"]
mod tests;
