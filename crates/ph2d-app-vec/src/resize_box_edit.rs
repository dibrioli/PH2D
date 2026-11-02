//! **RESIZE BOX da seleção** — a projecção que o painel lê, e a porta que a edita (plano UI/UX
//! W3b, decisão do Enio 2026-08-03).
//!
//! O porquê e a história: `docs/Vector Module/50_cabecalhos_do_app_vec.md` § `resize_box_edit.rs`.

use ph2d_ecs::{Entity, SimWorld, VecResizeBox};
use ph2d_vec_scene::VecPathId;

use ph2d_vec_entities::entities::VecEntityMap;

/// O sujeito do checkbox: a ÚNICA forma selecionada.
///
/// ⚠️ **Uma seleção múltipla devolve `None`, e a linha não é pintada.** Um checkbox sobre N
/// objetos teria de responder *"e se metade estiver marcada?"* — o design system tem o
/// `Indeterminate` para isso, mas o clique nele precisaria de uma política (marca todos? inverte
/// cada um?) que ninguém pediu. Uma linha ausente é honesta; um tri-estado sem regra de clique é
/// um controlo que faz coisas diferentes em dias diferentes.
fn subject(map: &VecEntityMap, selected: &[VecPathId]) -> Option<Entity> {
    let [only] = selected else { return None };
    map.get(only).copied().map(Entity::from_bits)
}

/// O estado do checkbox para o painel — `None` = a linha não existe neste frame.
#[must_use]
pub fn selected_resize_box(
    sim: &SimWorld,
    map: &VecEntityMap,
    selected: &[VecPathId],
) -> Option<bool> {
    let e = subject(map, selected)?;
    let w = sim.world();
    w.get_entity(e).ok()?;
    Some(ph2d_ecs::resizes_box(w, e))
}

/// Aplica o clique. Devolve `true` se o mundo mudou — o `post_frame_undo` regista por diff, então
/// um no-op não custa passo de undo.
///
/// ⚠️ **O clique é um TOGGLE do que está na tela**, e não a escrita de um valor fixo: o checkbox
/// mostra a resposta EFETIVA (default composto com override), então inverter o que se vê é a
/// única leitura que não surpreende. Escrever um valor absoluto exigiria que o painel soubesse o
/// default — a segunda resposta que este módulo existe para evitar.
pub fn toggle_resize_box(sim: &mut SimWorld, map: &VecEntityMap, selected: &[VecPathId]) -> bool {
    let Some(e) = subject(map, selected) else {
        return false;
    };
    let w = sim.world();
    if w.get_entity(e).is_err() {
        return false;
    }
    let next = !ph2d_ecs::resizes_box(w, e);
    let default = ph2d_ecs::resize_box_default(w, e);
    let Ok(mut em) = sim.world_mut().get_entity_mut(e) else {
        return false;
    };
    if next == default {
        // O NEUTRO destaca. Ver o ⚠️ do cabeçalho: um `true` gravado por já ser filho de moldura
        // viajaria com a forma para fora dela.
        return em.take::<VecResizeBox>().is_some();
    }
    em.insert(VecResizeBox(next));
    true
}

#[cfg(test)]
#[path = "resize_box_edit_tests.rs"]
mod tests;
