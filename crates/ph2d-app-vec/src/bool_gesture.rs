//! **Os gestos da BOOLEANA VIVA** — armar, re-mirar e consolidar.

use ph2d_ecs::{Entity, SimWorld, VecBoolGroup};
use ph2d_vec_scene::{VecPathId, VecScene};

use ph2d_vec_entities::entities::VecEntityMap;

/// O grupo booleano que contém a seleção, se houver.
///
/// Sobe a partir de cada caminho selecionado e devolve o **mais próximo** — com booleanas
/// aninhadas, quem o clique re-mira é o grupo imediato, que é o que o artista tem em mãos.
#[must_use]
pub fn group_of_selection(
    sim: &SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<Entity> {
    let w = sim.world();
    for id in selected {
        let Some(&bits) = map.get(id) else { continue };
        let mut cur = Entity::from_bits(bits);
        for _ in 0..64 {
            let Some(parent) = w.get::<ph2d_ecs::ChildOf>(cur).map(|c| c.parent()) else {
                break;
            };
            if w.get::<VecBoolGroup>(parent).is_some() {
                return Some(parent);
            }
            cur = parent;
        }
    }
    None
}

/// A operação do grupo booleano `group` (a ④ acende o botão dela); `None` sem grupo.
#[must_use]
pub fn group_op(sim: &SimWorld, group: Option<Entity>) -> Option<u8> {
    group.and_then(|g| sim.world().get::<VecBoolGroup>(g).map(|b| b.op))
}

/// **Arma** a booleana viva sobre a seleção — re-mirando um grupo existente, ou criando um.
/// Devolve `true` se alguma coisa mudou (e o `post_frame_undo` a capturará).
pub fn arm(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    selected: &[VecPathId],
    op: u8,
) -> bool {
    if let Some(g) = group_of_selection(sim, map, selected) {
        // Re-mira. O componente é `Copy`, então re-inserir é a edição inteira.
        sim.world_mut().entity_mut(g).insert(VecBoolGroup { op });
        return true;
    }
    // ⚠️ A triagem é a MESMA da booleana destrutiva (`selected_closed_z`): só regiões FECHADAS
    // são operandos, e menos de duas não é uma booleana. Uma segunda regra aqui faria o botão
    // aceitar no modo vivo o que recusa no destrutivo.
    let closed: Vec<u64> = selected
        .iter()
        .filter(|id| scene.path(**id).is_some_and(|p| p.closed))
        .filter_map(|id| map.get(id).copied())
        .collect();
    if closed.len() < 2 {
        eprintln!("[ph2d-vec] boolean live: selecione >= 2 regioes FECHADAS");
        return false;
    }
    let Some(bits) = ph2d_vec_entities::entities::group_entities(sim, &closed, "Boolean".into())
    else {
        return false;
    };
    sim.world_mut()
        .entity_mut(Entity::from_bits(bits))
        .insert(VecBoolGroup { op });
    true
}

/// **Consolida** a booleana viva do grupo `g`: o que está na tela vira caminhos comuns, no z da
/// base, e o grupo morre.
///
/// ⚠️ Ele materializa o `plan` — *a geometria que o produtor já cozinhou neste frame* — e **não**
/// re-pergunta ao motor. É o que garante que o clique não move um pixel: com duas portas, o Apply
/// responderia a partir de um estado que pode ter mudado desde o último desenho.
///
/// Devolve quantos caminhos nasceram.
pub fn bake(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    pen: &mut ph2d_vec_edit::PenTool,
    plan: &crate::bool_live::Cooked,
    g: Entity,
) -> usize {
    let new_ids = bake_ids(sim, scene, plan, g);
    pen.select_many(&new_ids);
    new_ids.len()
}

/// O [`bake`] sem tocar na seleção — o Flatten junta o que nasce de vários grupos antes de a pôr.
pub(crate) fn bake_ids(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    plan: &crate::bool_live::Cooked,
    g: Entity,
) -> Vec<VecPathId> {
    // O z da base, ANTES de remover seja o que for — é a fatia que o resultado ocupa, a mesma
    // regra da booleana destrutiva (*"o resultado ocupa a fatia de z dela, não salta pro topo"*).
    let at = scene
        .paths()
        .iter()
        .position(|p| p.id == plan.base)
        .unwrap_or(0);
    for id in &plan.operands {
        scene.remove_path(*id);
    }
    let new_ids: Vec<VecPathId> = plan
        .out
        .iter()
        .enumerate()
        .map(|(k, r)| scene.insert_path(at + k, r.clone()))
        .collect();
    rehang_non_operands(sim, plan, g);
    // O grupo morre: os operandos já saíram do documento, e um grupo vazio na Hierarquia é lixo
    // que o artista teria de apagar à mão. `despawn` leva os descendentes que restam.
    if let Ok(e) = sim.world_mut().get_entity_mut(g) {
        e.despawn();
    }
    new_ids
}

/// Os filhos de `g` que não são operandos (traços abertos) mudam-se para o pai do grupo — ou para a
/// raiz — com a pose do grupo somada, como no Ungroup; senão o `despawn` apagá-los-ia.
fn rehang_non_operands(sim: &mut SimWorld, plan: &crate::bool_live::Cooked, g: Entity) {
    let w = sim.world();
    let parent = w.get::<ph2d_ecs::ChildOf>(g).map(|c| c.parent());
    let pose = w.get::<ph2d_ecs::Transform>(g).map(|t| t.translation);
    let kids: Vec<Entity> = w
        .get::<ph2d_ecs::Children>(g)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();
    let loose: Vec<Entity> = kids
        .into_iter()
        .filter(|k| {
            w.get::<ph2d_ecs::VecPathRef>(*k)
                .is_some_and(|r| !plan.operands.contains(&r.0))
        })
        .collect();
    for k in loose {
        if let (Some(p), Some(mut tr)) = (pose, sim.world_mut().get_mut::<ph2d_ecs::Transform>(k)) {
            tr.translation += p;
        }
        let order = ph2d_vec_entities::entities::next_root_order(sim);
        if let Ok(mut e) = sim.world_mut().get_entity_mut(k) {
            match parent {
                Some(p) => {
                    e.insert(ph2d_ecs::ChildOf(p));
                }
                None => {
                    e.remove::<ph2d_ecs::ChildOf>();
                    e.insert(ph2d_ecs::RootOrder(order));
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "bool_gesture_tests.rs"]
mod tests;
