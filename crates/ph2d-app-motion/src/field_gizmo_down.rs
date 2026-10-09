//! **O Down de um arrasto de campo** — a semente inteira no espaço LOCAL do objecto Motion em edição.
//!
//! Os params (`center_*`, `rotation`, tamanho) são do espaço do objecto (doc ObjectsAndComponents 01
//! §13). Sob uma escala NÃO uniforme do dono, um campo rodado é um paralelogramo no mundo, e nenhuma
//! pose `(T, R, S)` de pai o exprime — a 1.ª versão punha a pose do dono no `parent_world` e o motor
//! do gizmo media as razões de escala nos eixos ERRADOS. ⇒ o cursor entra no espaço do objecto pela
//! inversa da afim ([`GizmoCamera::screen_in_model`]), o pai é a identidade, e o motor canónico
//! ([`ph2d_editor_core::compute_gizmo_transform`]) corre exacto. Identidade = o caminho de antes.

use ph2d_editor_core::{
    GizmoCamera, GizmoDragKind, GizmoDragState, GizmoTarget, TransformSnapshot,
};

use super::{FieldGizmoDrag, seed_start, selected_field};
use crate::motion_state::MotionState;

/// O arrasto aberto por um Down na alça `kind` em `px` (pixels da CENA, a `cam` do sub-retângulo do
/// split), ou `None` sem campo espacial seleccionado. `ctrl` = escala pelo centro.
#[must_use]
pub fn drag_do_down(
    motion: &MotionState,
    kind: GizmoDragKind,
    cam: &GizmoCamera,
    px: (f32, f32),
    ctrl: bool,
) -> Option<FieldGizmoDrag> {
    let (nid, spec) = selected_field(motion)?;
    let p = |name: &str| crate::motion_bridge::params::param_value(motion, nid, name);
    let intrinsic_half = spec.size.half(p);
    let start = seed_start(
        p(spec.center_x),
        p(spec.center_y),
        spec.rotation.map_or(0.0, p),
    );
    let model = crate::motion_objeto::afim_em_edicao(motion);
    let local_px = cam.screen_in_model(&model, px);
    // Rotate pivota no centro; scale, no canto/borda OPOSTOS (ou no centro com Ctrl) — a política do
    // sprite. `anchor = [0, 0]`: o `start` de um campo É o centro dele.
    let pivot = ph2d_editor_core::anchor_pivot_world(kind, [0.0, 0.0], intrinsic_half, start, ctrl);
    Some(FieldGizmoDrag {
        drag: GizmoDragState {
            kind,
            // Sentinela: o writeback é por-param, então este drag NUNCA lê nem escreve uma entidade.
            entity_bits: 0,
            start_screen: local_px,
            cursor_screen: local_px,
            start_transform: start,
            pivot_world: pivot,
            start_cursor_world: cam.screen_to_world(local_px),
            sprite_half_intrinsic: intrinsic_half,
            anchor_is_center: ctrl,
            target: GizmoTarget::MotionField,
            parent_world: TransformSnapshot::IDENTITY,
            turns: 0,
        },
        node: nid,
        spec,
        intrinsic_half,
        model,
    })
}

#[cfg(test)]
#[path = "field_gizmo_down_tests.rs"]
mod tests;
