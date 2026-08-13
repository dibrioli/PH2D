//! **Quem PUBLICA o número do arrasto de gizmo** (o estudo da UI viva, C3).
//!
//! A lei do número e o desenho da ficha vivem os dois em `ph2d-editor-core`
//! ([`ph2d_editor::gizmo::gizmo_readout`] + [`ph2d_editor::readout`]), onde os gates os medem sem
//! janela. O que só a shell pode fazer é o meio: ler o `Transform` **VIVO** da entidade — o número
//! que ela própria acabou de escrever — e entregá-lo já formatado.
//!
//! # Porque é o `Transform` vivo, e não o cursor
//!
//! O arrasto é aplicado por vários braços (`advance_gizmo_drag`: translate, scale, rotate, pivot,
//! multi-selecção, moldura), cada um com o seu encaixe: o Ctrl quantiza a POSIÇÃO, o Shift quantiza
//! o ÂNGULO e tranca a proporção. Ler de volta o que foi escrito é **uma** porta que cobre os
//! braços todos — incluindo os que ainda não existem — sem uma lista de sítios de publicação a
//! apodrecer. Re-derivar do cursor daria `12,03` com a forma pousada em `12,00`.
//!
//! # Só os alvos cujo resultado É o `Transform` da entidade
//!
//! `FlipPose`, `FlipSelection` e `MotionField` escrevem noutro sítio (a pose de uma chave, a
//! geometria seleccionada, os params de um nó), então para eles o `Transform` da entidade **não se
//! move** — e a ficha diria `+0,0` com a mão a arrastar. ⚠️ Um número errado apresentado como certo
//! é pior que número nenhum: eles ficam sem ficha, e há gate a fixá-lo. Dar-lhes uma ficha é uma
//! wave própria, porque cada um precisa que o seu dono publique o que aplicou.

use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_editor::gizmo::{GizmoTarget, TransformSnapshot, gizmo_readout};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

use crate::Transform;

/// Publica (ou apaga) o texto da ficha do arrasto em curso. Uma vez por quadro.
pub(super) fn publish(
    hero: &mut HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
) {
    hero.gizmo.readout = readout_text(hero, sim, camera, window_size);
}

/// O texto, ou `None` — separado do escritor para o arch-gate poder falar de uma coisa só.
fn readout_text(
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
) -> Option<String> {
    let drag = hero.gizmo.drag?;
    if !matches!(
        drag.target,
        GizmoTarget::PrimaryIndividual | GizmoTarget::ExtraIndividual(_) | GizmoTarget::Global
    ) {
        return None;
    }
    let entity = ph2d_ecs::Entity::from_bits(drag.entity_bits);
    let t = sim.world().get::<Transform>(entity)?;
    let now = TransformSnapshot {
        translation: [t.translation.x, t.translation.y],
        rotation: t.rotation,
        scale: [t.scale.x, t.scale.y],
    };
    let r = gizmo_readout(drag.kind, &drag.start_transform, &now);
    if r.is_idle() {
        return None;
    }
    // O zoom VIVO: as casas decimais seguem a resolução que a tela de facto mostra, a mesma regra
    // (e a mesma porta) do rótulo do smart guide.
    let px_per_world = if camera.height_world > 0.0 {
        f64::from(window_size.height as f32 / camera.height_world)
    } else {
        1.0
    };
    Some(r.text(ph2d_editor::LengthDisplay::of(&hero.project), px_per_world))
}
