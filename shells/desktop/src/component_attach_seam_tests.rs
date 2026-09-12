//! **A costura do `+`: anexar um componente faz a secção do Inspector APARECER.**
//!
//! ⛔⛔ **Estes dois gates não puderam ir com a família, e a razão é estrutural.** Eles perguntam
//! ao `render_loop::inspector_presence_probe`, que chama um builder **privado do `render_loop`** —
//! e **uma crate nunca pode chamar o `bin`** (HOWTO §4: a seta aponta sempre shell → família). O
//! resto do ficheiro donte eles saíram (`component_attach_tests`) mudou-se para a
//! [`ph2d_app_components`] em 2026-09-12; estes ficaram porque o **sujeito** deles é meio chrome.
//!
//! ⚠️ *Os testes seguem o SUJEITO, não o ficheiro* (HOWTO §1.2) — o piloto partiu 5 ficheiros de
//! teste 4/1 pela mesma régua.
//!
//! ⏳ **Quando eles voltam para lá:** quando a família da **Sprite** sair da shell. O 9-Slice é
//! dela, e é o `inspector_slice` dela que este probe alcança.
//!
//! ⚠️ O arnês (`registo`, `image`) vem do `ph2d_app_components::test_support`, atrás da feature
//! `test-support`: um `#[cfg(test)]` da família é **falso** quando é ela a ser compilada como
//! dependência (HOWTO §2.5).

use ph2d_app_components::test_support::{image, registo as registry};
use ph2d_ecs::{SimWorld, Transform};
use ph2d_editor_core::{HeroScreen, NodeId};

/// ⭐ **E o pick deixa o componente na CENA** — a ponta final da sequência.
///
/// ⚠️ **Pelo `route_pick`, que é o dreno CONDICIONAL:** o canal de pick tem três consumidores (a
/// biblioteca de nós do Motion, o `Ctrl+K` e este), e um `take` incondicional faria quem recebe o
/// pick ser *a ordem dos drenos no quadro*.
#[test]
fn picking_an_item_leaves_the_component_in_the_scene() {
    let mut sim = SimWorld::new();
    let bits = image(&mut sim);
    let reg = registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let mut target: Option<u64> = None;
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    ph2d_app_components::component_attach::open_palette_if_asked(
        &mut hero,
        &sim,
        &reg,
        Some(bits),
        &mut target,
    );

    hero.store
        .set_command_pick(ph2d_app_components::component_palette::item_id(
            "ph2d::ecs::SliceNine",
        ));
    let picked = ph2d_app_components::component_attach::route_pick(&mut hero, &mut target);
    assert!(picked.is_some(), "o dreno nao reconheceu o proprio id");
    ph2d_app_components::component_attach::attach_picked(
        picked.as_ref(),
        &mut sim,
        &reg,
        ph2d_app_physics::physics_seed::COMPONENT_SEEDS,
        &mut toasts,
    );

    let e = ph2d_ecs::Entity::from_bits(bits);
    assert!(
        sim.world().get::<ph2d_ecs::SliceNine>(e).is_some(),
        "o componente escolhido nao chegou a` cena"
    );
    // ⭐ E a seção da §5 passa a existir — que é a razão de tudo isto.
    assert!(
        crate::render_loop::inspector_presence_probe::slice(sim.world(), bits),
        "o artista anexou o 9-Slice e a seccao nao apareceu"
    );
    assert_eq!(target, None, "o alvo tem de ser limpo depois do pick");
}

/// ⭐ **Anexar é UM passo de desfazer, e desfazer FECHA a seção** (ADR-0166 / F3).
///
/// ⚠️ **Medido pela captura, que é a unidade do undo** (`ProjectState` = `WorldSnapshot` +
/// `VecScene`; o passo nasce de um DIFF no fim do quadro). O que este gate afirma são as três
/// coisas que o artista vê: a captura **muda** (senão não haveria passo nenhum e o Ctrl+Z saltaria
/// por cima), repor a captura anterior **tira o componente**, e com ele fora a **seção some**.
///
/// ⛔ Ele não encena o `App::post_frame_undo` — aquele pede a janela inteira. O que ele mede é a
/// propriedade de que o passo depende.
#[test]
fn attaching_is_one_undo_step_and_undoing_closes_the_section() {
    let mut sim = SimWorld::new();
    let bits = image(&mut sim);
    let reg = registry();

    let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
    let mut work = ph2d_ecs::WorklistBuf::default();
    let mut snap = |sim: &mut SimWorld| {
        let mut out = ph2d_ecs::scene::WorldSnapshot::default();
        ph2d_ecs::scene::world_to_snapshot(sim.world_mut(), &mut prop, &mut work, &reg, &mut out)
            .expect("captura");
        out
    };

    let before = snap(&mut sim);
    ph2d_app_components::component_attach::attach_by_name(
        &mut sim,
        &reg,
        ph2d_app_physics::physics_seed::COMPONENT_SEEDS,
        bits,
        "ph2d::ecs::SliceNine",
    )
    .expect("anexa");
    let after = snap(&mut sim);
    assert_ne!(
        before, after,
        "anexar nao mudou a captura — nao haveria passo de undo nenhum"
    );

    // O Ctrl+Z repõe a captura anterior.
    //
    // ⚠️ **DESPAWNA primeiro, como o `undo::restore` faz** — e a 1.ª versão deste gate não o fazia
    // e ficou vermelha por isso. O `snapshot_to_world` declara no doc que *"`world` não precisa de
    // estar vazio"*: ele SOMA linhas, não substitui o mundo. Sem a limpeza, o gate reencontrava a
    // entidade ORIGINAL (com o componente) e acusava o produto de um defeito do arnês.
    let editable: Vec<ph2d_ecs::Entity> = {
        let mut q = sim
            .world_mut()
            .query_filtered::<ph2d_ecs::Entity, bevy_ecs::query::With<Transform>>();
        q.iter(sim.world()).collect()
    };
    for e in editable {
        let _ = sim.world_mut().despawn(e);
    }
    ph2d_ecs::scene::snapshot_to_world(sim.world_mut(), &before, &reg).expect("restore");
    // ⚠️ **O restore RE-SPAWNA tudo com bits novos**, então a entidade tem de ser reencontrada
    // pela identidade que sobrevive — que é a razão de o `StableId` existir.
    let again = sim
        .world_mut()
        .query::<(ph2d_ecs::Entity, &ph2d_render::Sprite)>()
        .iter(sim.world())
        .map(|(e, _)| e)
        .next()
        .expect("a sprite volta");
    assert!(
        sim.world().get::<ph2d_ecs::SliceNine>(again).is_none(),
        "desfazer nao tirou o componente"
    );
    assert!(
        !crate::render_loop::inspector_presence_probe::slice(sim.world(), again.to_bits()),
        "o componente saiu e a seccao ficou"
    );
}
