//! **Os gestos da BOOLEANA VIVA** — armar, re-mirar e consolidar.
//!
//! O motor é o [`crate::bool_live`]; aqui mora o que é de DOCUMENTO (o grupo, o z, o undo) e a
//! decisão que os oito botões da seção Boolean tomam quando são clicados.
//!
//! # Um clique, três destinos — e a ordem deles é a lei
//!
//! 1. **A seleção já está dentro de um grupo booleano** ⇒ o clique RE-MIRA aquele grupo. É o que
//!    faz os oito botões serem também o seletor de operação de um grupo vivo: uma lista de oito
//!    verbos, e clicar num deles diz *"passe a ser este"*. Um dropdown separado seria uma segunda
//!    porta para a mesma pergunta, e ela divergiria no dia em que a nona operação entrasse só num
//!    dos dois.
//! 2. **O modo `Live` está ligado** ⇒ o clique CRIA um grupo com os selecionados.
//! 3. **Senão** ⇒ o caminho destrutivo de sempre, byte-idêntico.
//!
//! ⚠️ **(1) vem antes de (2) de propósito.** Sem essa ordem, re-mirar um grupo com o modo
//! desligado o CONSUMIRIA — o artista clicaria "Intersect" esperando trocar a operação e perderia
//! os operandos.

use ph2d_ecs::{Entity, SimWorld, VecBoolGroup};
use ph2d_vec_scene::{VecPathId, VecScene};

use crate::vec_entities::VecEntityMap;

/// O grupo booleano que contém a seleção, se houver.
///
/// Sobe a partir de cada caminho selecionado e devolve o **mais próximo** — com booleanas
/// aninhadas, quem o clique re-mira é o grupo imediato, que é o que o artista tem em mãos.
#[must_use]
pub(crate) fn group_of_selection(
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

/// **Arma** a booleana viva sobre a seleção — re-mirando um grupo existente, ou criando um.
/// Devolve `true` se alguma coisa mudou (e o `post_frame_undo` a capturará).
pub(crate) fn arm(
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
    let Some(bits) = crate::vec_entities::group_entities(sim, &closed, "Boolean".into()) else {
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
pub(crate) fn bake(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    pen: &mut ph2d_vec_edit::PenTool,
    plan: &crate::bool_live::Cooked,
    g: Entity,
) -> usize {
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
    // O grupo inteiro morre: os operandos já saíram do documento, e um grupo vazio na Hierarquia
    // é lixo que o artista teria de apagar à mão. `despawn` leva os descendentes junto.
    if let Ok(e) = sim.world_mut().get_entity_mut(g) {
        e.despawn();
    }
    pen.select_many(&new_ids);
    new_ids.len()
}

#[cfg(test)]
#[path = "bool_gesture_tests.rs"]
mod tests;
