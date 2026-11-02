//! **O RECORTE da seleção** — a projeção que o painel lê, e o chip que a edita.
//!
//! O porquê e a história: `docs/Vector Module/50_cabecalhos_do_app_vec.md` § `vec_clip_edit.rs`.

use ph2d_ecs::{Entity, SimWorld, VecClipContent};
use ph2d_vec_scene::{VecPathId, VecScene};

use crate::vec_frame_edit::{entity_of, is_within};
use ph2d_vec_entities::entities::VecEntityMap;

/// O caminho `id` é FECHADO? Um id sem caminho na cena descreve um mundo que já não existe, e ali
/// a resposta honesta é "não" — nunca um recorte oferecido sobre geometria que ninguém tem.
fn is_closed(scene: &VecScene, id: VecPathId) -> bool {
    scene.paths().iter().any(|p| p.id == id && p.closed)
}

/// **A forma que recorta nesta seleção** — a fechada que CONTÉM tudo o que está selecionado.
/// `None` = a seleção não oferece o controlo.
#[must_use]
pub(crate) fn clip_subject_of_selection(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<Entity> {
    if selected.is_empty() {
        return None;
    }
    let members: Vec<Entity> = selected
        .iter()
        .filter_map(|id| entity_of(sim, map, *id))
        .collect();
    // ⚠️ A herança literal do `frame_of_selection`: um caminho selecionado sem entidade viva faria
    // *"contém tudo"* deixar de significar o que diz — a forma pareceria conter uma seleção cuja
    // metade ausente ninguém conferiu.
    if members.len() != selected.len() {
        return None;
    }
    selected.iter().zip(members.iter()).find_map(|(&id, &e)| {
        (is_closed(scene, id) && members.iter().all(|&m| is_within(sim, e, m))).then_some(e)
    })
}

/// O recorte da seleção deste frame: `None` = a seleção não oferece o controlo; `Some(false)` =
/// oferece e está desligado.
#[must_use]
pub fn selected_clip(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<bool> {
    let e = clip_subject_of_selection(sim, scene, map, selected)?;
    Some(sim.world().get::<VecClipContent>(e).is_some())
}

/// Liga/desliga o recorte na forma da seleção. Devolve `true` se algo mudou — o
/// `post_frame_undo` captura por diff, então um no-op não custa passo de undo.
pub fn set_selected_clip(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    selected: &[VecPathId],
    on: bool,
) -> bool {
    let Some(e) = clip_subject_of_selection(sim, scene, map, selected) else {
        return false;
    };
    if sim.world().get::<VecClipContent>(e).is_some() == on {
        return false;
    }
    let Ok(mut em) = sim.world_mut().get_entity_mut(e) else {
        return false;
    };
    // ⚠️ **Desligar REMOVE o componente**, não o põe em falso: a ausência é o estado "não
    // recorta" (é o que faz um documento sem recortes ser byte-idêntico ao de antes da feature),
    // e um componente presente-e-inerte daria ao undo um passo que a tela não distingue.
    if on {
        em.insert(VecClipContent);
    } else {
        em.remove::<VecClipContent>();
    }
    true
}

#[cfg(test)]
#[path = "vec_clip_edit_tests.rs"]
mod tests;
