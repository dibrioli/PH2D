//! ⭐⭐ **O que a MÃO do vetor segura neste quadro** — publicado em [`MaosDasFamilias`] (bug do dono,
//! 21/10: *«com timeline aberta os gizmos de ajustes das shapes não aceitam ajustes»*).
//!
//! Com o Auto Key armado (painel da timeline aberto), um arrasto que a timeline não conhecia era uma
//! «edição discreta» por quadro: o passe gravava a chave, a curva nascia e o apply do quadro
//! seguinte reescrevia a forma com ela — o dedo perdia para a curva. O gizmo e o osso já eram mãos
//! (`timeline_bridge::maos_do_quadro`); estes são os gestos vivos do vetor, a mesma cadeia do
//! «mover» da shell.

use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::{MaosDasFamilias, SimWorld};
use ph2d_vec_scene::VecPathId;

use crate::state::VecState;

/// Há um gesto vivo do vetor (o botão está premido sobre uma alça, um ponto, um gradiente…)?
#[must_use]
pub fn gesto_vivo(vec: &VecState) -> bool {
    vec.shape_handles.grab.is_some()
        || vec.grad_drag.is_some()
        || vec.envelope_drag.is_some()
        || vec.patternpath_handle.is_some()
        || vec.width_grab.is_some()
        || vec.textpath_handle_drag
        || vec.conn_handle.is_some()
        || vec.connect.is_some()
        || vec.pen.is_dragging()
        || vec.shape.is_active()
        || vec.pencil.is_active()
}

/// As entidades que a mão segura: as do alvo explícito do gesto e, porque metade deles age sobre a
/// selecção (o gradiente, o padrão, o texto no caminho), a selecção inteira. Sem gesto, nenhuma.
#[must_use]
pub fn maos(vec: &VecState) -> Vec<u64> {
    if !gesto_vivo(vec) {
        return Vec::new();
    }
    let mut ids: Vec<VecPathId> = vec.pen.selected_paths().to_vec();
    ids.extend(vec.shape_handles.grab.map(|g| g.id));
    ids.extend(vec.width_grab.as_ref().map(|g| g.path));
    ids.extend(vec.conn_handle.as_ref().map(|h| h.path));
    ids.extend(vec.connect.as_ref().map(|c| c.path));
    ids.extend(
        vec.pen
            .dragging_anchors()
            .unwrap_or_default()
            .into_iter()
            .map(|(id, _)| id),
    );
    let mut out: Vec<u64> = ids
        .iter()
        .filter_map(|id| vec.entities.get(id).copied())
        .collect();
    out.extend(vec.envelope_drag.map(|(bits, _)| bits));
    out.sort_unstable();
    out.dedup();
    out
}

/// Publica [`maos`] no mundo — uma vez por quadro, antes do apply da timeline.
pub fn publicar(vec: &VecState, sim: &mut SimWorld) {
    let m = maos(vec);
    let w = sim.world_mut();
    if let Some(mut r) = w.get_resource_mut::<MaosDasFamilias>() {
        r.publicar(VETOR, m);
    } else if !m.is_empty() {
        let mut r = MaosDasFamilias::default();
        r.publicar(VETOR, m);
        w.insert_resource(r);
    }
}

#[cfg(test)]
#[path = "maos_tests.rs"]
mod tests;
