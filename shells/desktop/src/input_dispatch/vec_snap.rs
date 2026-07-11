//! Snap de objeto vetorial em TEMPO REAL durante um arraste de gizmo (Enio
//! 2026-07-10). Chamado a cada `CursorMoved`, logo depois de `advance_gizmo_drag`
//! escrever a pose deste move: se a entidade arrastada é uma forma vetorial, ela
//! desliza para se ALINHAR ao vizinho — bordas/centros/vértices, eixos X e Y
//! independentes (o snap completo de editor vetorial). O snap acontece DURANTE o
//! arraste, não só no release; a fusão das pontas de formas abertas continua no
//! release (`weld_new_shape`).
//!
//! Módulo irmão de `gizmo_drag` (que está no teto de LOC): só a chamada mora no
//! hub de eventos; toda a lógica vive aqui.

use ph2d_ecs::SimWorld;

use crate::{App, Transform};

/// Desliza a entidade `bits` por um delta de MUNDO, convertido para o frame local do
/// pai e somado à `translation`. No-op se o delta é nulo. Recebe só o `SimWorld` (não
/// o `AppGfx` inteiro) para não colidir com um borrow vivo de `hero_screen` no chamador
/// do release. Compartilhado pelo snap em tempo real e pelo snap no release.
pub(crate) fn slide_entity_world(sim: &mut SimWorld, bits: u64, delta: [f64; 2]) {
    if delta == [0.0, 0.0] {
        return;
    }
    let entity = ph2d_ecs::Entity::from_bits(bits);
    let pw = ph2d_ecs::parent_world_transform(sim.world(), entity);
    let parent = ph2d_editor::TransformSnapshot {
        translation: [pw.translation.x, pw.translation.y],
        rotation: pw.rotation,
        scale: [pw.scale.x, pw.scale.y],
    };
    let [dx, dy] = ph2d_editor::world_delta_to_local(parent, delta[0] as f32, delta[1] as f32);
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity) {
        t.translation.x += dx;
        t.translation.y += dy;
    }
}

impl App {
    /// Aplica o snap de alinhamento à entidade que o gizmo está arrastando AGORA.
    /// No-op se não há drag, se é um `MovePivot` (mexe no pivô, não na forma), se o
    /// drag é de grupo (o snap da primária desalinharia os extras), se a entidade
    /// não é uma forma vetorial, ou se nada alinha. Só desliza o `Transform`; o undo
    /// global captura a mudança de pose.
    pub(crate) fn snap_dragged_vec_during_drag(&mut self) {
        let drag = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.gizmo.drag);
        let Some(drag) = drag else {
            return;
        };
        if matches!(drag.kind, ph2d_editor::GizmoDragKind::MovePivot)
            || !self.group_drag_starts.is_empty()
        {
            return;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let entity = ph2d_ecs::Entity::from_bits(drag.entity_bits);
        let Some(id) = gfx
            .sim
            .world()
            .get::<ph2d_ecs::VecPathRef>(entity)
            .map(|v| v.0)
        else {
            return;
        };
        let win = gfx.surface.size();
        let tol = crate::vec_gizmo_view::stroke_hit_r(&gfx.camera, win) * 1.5;
        let xforms = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
        let delta = gfx.vec_scene.align_snap_delta(id, &xforms, tol);
        slide_entity_world(&mut gfx.sim, drag.entity_bits, delta);
    }
}
