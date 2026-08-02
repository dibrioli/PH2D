//! **A moldura da SELEÇÃO** — a projeção que o painel lê, e o chip que a edita.
//!
//! Duas metades da mesma pergunta (*a seleção é uma moldura, e ela recorta?*), respondidas UMA vez
//! e no lugar onde o mundo mora. O painel não alcança o ECS — e não deve: se alcançasse, haveria
//! duas respostas para *"esta seleção é uma moldura?"*, e a que desenha a seção divergiria da que
//! honra o clique.
//!
//! ⚠️ A pergunta é feita ao caminho **selecionado**, não à entidade selecionada na Hierarquia: o
//! painel Vector fala em `VecPathId` em toda parte, e é essa a seleção que o artista vê realçada
//! no canvas.

use ph2d_ecs::{Entity, SimWorld, VecFrame};
use ph2d_vec_scene::VecPathId;

use crate::vec_entities::VecEntityMap;

/// A entidade do caminho selecionado, se houver exactamente um.
fn selected_entity(sim: &SimWorld, map: &VecEntityMap, selected: &[VecPathId]) -> Option<Entity> {
    // Com dois selecionados a pergunta *"esta moldura recorta?"* não tem UMA resposta, e um chip
    // que mostra a do primeiro editaria o que o artista não está a olhar.
    let [id] = selected else { return None };
    let &bits = map.get(id)?;
    let e = Entity::from_bits(bits);
    sim.world().get_entity(e).ok().map(|_| e)
}

/// A moldura da seleção deste frame: `None` = não é moldura.
#[must_use]
pub(crate) fn selected_frame_clip(
    sim: &SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<bool> {
    let e = selected_entity(sim, map, selected)?;
    sim.world().get::<VecFrame>(e).map(|f| f.clip)
}

/// Escreve o recorte na moldura selecionada. Devolve `true` se algo mudou — o `post_frame_undo`
/// captura por diff, então um no-op não custa passo de undo.
pub(crate) fn set_selected_frame_clip(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
    clip: bool,
) -> bool {
    let Some(e) = selected_entity(sim, map, selected) else {
        return false;
    };
    // ⚠️ Só escreve numa entidade que JÁ é moldura. O chip só é pintado sobre uma, então chegar
    // aqui sobre outra coisa seria um clique roteado errado — e criar a moldura em silêncio
    // transformaria um chip de opção num gesto de criação.
    if sim
        .world()
        .get::<VecFrame>(e)
        .is_none_or(|f| f.clip == clip)
    {
        return false;
    }
    if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
        em.insert(VecFrame { clip });
        return true;
    }
    false
}

#[cfg(test)]
#[path = "vec_frame_edit_tests.rs"]
mod tests;
