//! **A MÃO NO OSSO é um arrasto** — o irmão do [`super::tests`] pelo teto de 600 LOC, cortado por
//! RESPONSABILIDADE: ali mora o diff da pose, aqui quem o quadro considera estar a arrastar.
use super::*;

/// ⭐⭐⭐ **POSAR UM OSSO É UM ARRASTO** — sem isso cada quadro do gesto abre um passo de undo seu.
/// ⛔ Mede a COSTURA (`run`, não o `apply_samples`): é ela que lê os dois estados.
#[test]
fn posing_a_bone_opens_one_drag_bracket_like_the_gizmo() {
    use ph2d_app_skeleton::state::SkeletonState;
    let hero = ph2d_editor_core::HeroScreen::new(ph2d_editor_core::NodeId(1));
    let world = ph2d_ecs::World::new();
    let drive = ph2d_preview_drive::PreviewDrive::default();
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.pause();
    let posando = SkeletonState {
        bone_pose: Some((7, ph2d_skeleton_render::BonePart::Body)),
        ..Default::default()
    };
    for (skeleton, esperado, porque) in [
        (&posando, true, "a mao no osso e' um arrasto"),
        (
            &SkeletonState::default(),
            false,
            "controlo: sem gesto nenhum nao ha' arrasto (senao o gate mede um `true` constante)",
        ),
    ] {
        let mut st = TimelineState::new();
        st.flags.auto_key = true;
        let mut ak = AutokeyState::default();
        super::run(
            &mut st,
            &ph,
            &mut ak,
            &mut ph2d_editor_core::ToastQueue::new(),
            &hero,
            &world,
            &drive,
            skeleton,
        );
        assert_eq!(ak.drag_active, esperado, "{porque}");
    }
}
