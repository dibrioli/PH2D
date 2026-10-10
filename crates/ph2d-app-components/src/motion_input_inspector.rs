//! **O instantâneo e o dreno da secção MOTION INPUT** (ciclo 19 do Motion).
//!
//! A ponte `u8` ⇄ `MotionSource` vive AQUI e só aqui (o painel não vê a `ph2d-ecs`).

use ph2d_ecs::{MotionInput, MotionSource, SimWorld};
use ph2d_editor_core::motion_input_edits::{
    InspectorMotionInputInfo, MOTION_INPUT_SOURCES, MotionInputFieldEdit as E,
};

const _: () = assert!(MotionSource::ALL.len() == MOTION_INPUT_SOURCES);

/// O índice que atravessa a fronteira ⇄ a fonte. Fora da faixa ⇒ `None` (edição inerte).
#[must_use]
pub fn source_de_u8(v: u8) -> Option<MotionSource> {
    MotionSource::ALL.get(usize::from(v)).copied()
}

/// ⭐ **Constrói o instantâneo** da entidade escolhida, ou `None` se ela não tem o componente.
///
/// `nomes_do_grafo` só corre quando o componente existe — é a varredura do grafo do Motion, que esta
/// crate não vê (quem a passa é a shell).
#[must_use]
pub fn build_info(
    sim: &SimWorld,
    bits: u64,
    selected_count: usize,
    nomes_do_grafo: impl FnOnce() -> Vec<String>,
) -> Option<InspectorMotionInputInfo> {
    let e = ph2d_ecs::Entity::from_bits(bits);
    let world = sim.world();
    let m = world.get::<MotionInput>(e)?;
    Some(InspectorMotionInputInfo {
        entity_bits: bits,
        name: m.name.clone(),
        source: m.source.index(),
        value: m.value,
        color: m.color,
        nomes_do_grafo: nomes_do_grafo(),
        // ⚠️ A MESMA porta que a ponte lê — senão o painel diria «tem vida» sobre uma que ela ignora.
        tem_vida: crate::health_bar_bridge::vida_da_barra(world, e).is_some(),
        selected_count,
    })
}

/// Aplica uma edição. `true` = o mundo mudou (escrever o MESMO valor não é mudança: o undo é por
/// diff).
pub fn apply(sim: &mut SimWorld, bits: u64, edit: &E) -> bool {
    let e = ph2d_ecs::Entity::from_bits(bits);
    let Some(mut m) = sim.world_mut().get_mut::<MotionInput>(e) else {
        return false;
    };
    match edit {
        E::Name(n) => {
            if m.name == *n {
                return false;
            }
            m.name.clone_from(n);
        }
        E::Source(i) => {
            let Some(s) = source_de_u8(*i) else {
                return false;
            };
            if m.source == s {
                return false;
            }
            m.source = s;
        }
        E::Value(v) => {
            if !v.is_finite() || m.value.to_bits() == v.to_bits() {
                return false;
            }
            m.value = *v;
        }
        E::Color(c) => {
            if m.color == *c {
                return false;
            }
            m.color = *c;
        }
    }
    true
}

/// Aplica todas as edições do quadro. `true` = alguma coisa mudou (`|`: todas correm).
pub fn apply_all(sim: &mut SimWorld, edits: &[(u64, E)]) -> bool {
    edits
        .iter()
        .fold(false, |mudou, (bits, edit)| apply(sim, *bits, edit) | mudou)
}

#[cfg(test)]
#[path = "motion_input_inspector_tests.rs"]
mod tests;
