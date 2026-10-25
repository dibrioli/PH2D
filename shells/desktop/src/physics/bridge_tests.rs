//! **The transport's Physics toggle, at the seam the product actually runs.**
//!
//! `crates/ph2d-physics-ecs/tests/it/hold.rs` proves `PhysicsBridge::hold` behaves.
//! That is worth nothing on its own: a `hold` nobody calls is unit-green and
//! dead in the product ([[feedback_tool_unit_green_integration_dead]]). These
//! gates drive `physics_bridge::dispatch` — the function the frame loop calls,
//! with the flag in the argument the frame loop passes — so the claim under
//! test is "the checkbox decides", not "a method exists".

use ph2d_core::Playhead;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::{InputTape, PhysicsBridge, PlayerInput};
use ph2d_timeline::TimelineDoc;

use ph2d_app_physics::bridge::dispatch::dispatch;
use ph2d_app_physics::physics_tests::{DT, FRAMES, falling_scene, height};

// Os gates que só medem o `dispatch` mudaram-se para `ph2d_app_physics::bridge::dispatch_tests`.

/// **UMA BOLA A CAIR NÃO É UMA EDIÇÃO** — a outra metade do pedido do Enio de 2026-08-23
/// (*«precisamos corrigir o CtrlZ para ambas»*), medida na porta que o produto corre.
///
/// ⚠️ **O passo de undo nasce por CLIQUE, não por quadro** (o `any_input_this_frame` não é
/// levantado por mover o cursor). Enquanto o solver escrevia o `Transform`, cada clique dado
/// durante uma corrida empilhava um passo cujo conteúdo era só a pose de uma bola a cair — e o
/// Ctrl+Z do artista gastava-se a desfazer nada. A cura é a folha [`ph2d_preview_drive`]: a pose que o
/// solver escreve é **pré-visualização**, e a captura fotografa o documento.
///
/// ⚠️ **As duas metades juntas, de propósito.** «Não muda durante a corrida» sozinho fica verde
/// numa ponte que nunca corre; «muda quando pára» sozinho fica verde sem cura nenhuma. E o
/// CONTROLO POSITIVO no meio (a bola caiu de facto) é o que impede o gate de medir o vazio.
///
/// **Mutação que deve sangrar:** tirar o `declare_solver_writes` do fim do `dispatch`.
#[test]
fn a_falling_ball_is_not_an_undo_step() {
    use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components};

    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    ph2d_render::register_render_components(&mut reg);
    let shot = |sim: &mut SimWorld, drive: &ph2d_preview_drive::PreviewDrive| {
        crate::undo::ProjectState::capture(
            drive,
            sim,
            &ph2d_vec_scene::VecScene::new(),
            &ph2d_flip::FlipDoc::new(),
            &ph2d_guides::GuideSet::default(),
            &ph2d_ui_state::StateSets::default(),
            &crate::project_library::LibraryDoc::default(),
            &[],
            &reg,
            &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
            None,
        )
    };

    let (mut sim, ball) = falling_scene();
    let mut bridge = PhysicsBridge::new();
    let mut doc = TimelineDoc::new();
    let mut playhead = Playhead::new(DT);
    playhead.play();
    let mut drive = ph2d_preview_drive::PreviewDrive::default();
    let at_rest = shot(&mut sim, &drive);
    let started_at = height(&sim, ball);

    for _ in 0..FRAMES {
        playhead.advance();
        dispatch(
            &mut bridge,
            &mut sim,
            &playhead,
            DT,
            &mut doc,
            true,
            PlayerInput::default(),
            &mut InputTape::new(),
            &mut drive,
        );
        drive.settle(); // o que o `post_frame_undo` faz no fim de cada quadro
        assert_eq!(
            shot(&mut sim, &drive),
            at_rest,
            "a bola a cair mudou a captura — cada clique viraria um passo de Ctrl+Z vazio"
        );
    }
    // CONTROLO POSITIVO: houve corrida.
    assert!(
        height(&sim, ball) < started_at - 1.0,
        "a bola nao caiu ({} -> {}) — o gate acima nao mediu nada",
        started_at,
        height(&sim, ball)
    );

    // E DESARMAR deixa UM passo: a pose caída passa a ser documento, e o Ctrl+Z desfaz a corrida.
    drive.settle();
    assert!(
        drive.is_empty(),
        "a `settle` tinha de esquecer o solver que largou"
    );
    assert_ne!(
        shot(&mut sim, &drive),
        at_rest,
        "parar a simulacao tem de deixar UM passo — a corrida"
    );
}
