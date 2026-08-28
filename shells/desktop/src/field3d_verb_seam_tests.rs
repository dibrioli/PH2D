//! ⭐⭐ **A COSTURA do verbo por forma e do raio de junção** (W97/W98) — do retrato publicado até ao
//! documento **cozido**.
//!
//! ⚠️ **Módulo FILHO do [`super`]**, e não um irmão de topo: ele usa as fixturas de lá (`a_world`,
//! `the_root`, `scene`) e um `use super::*` alcança-as. Duplicá-las aqui seria a segunda cena de
//! teste a envelhecer sozinha.
//!
//! ⚠️ Os gates da **lei** vivem na `ph2d-field-ecs` (materializa · preserva o carácter · não toca no
//! irmão). O que se mede aqui é o que **nenhum deles vê**: que a linha é de facto **publicada** no
//! retrato que o painel recebe. *Uma lei correcta que o painel não publica é um gesto que ninguém
//! alcança* — a quinta reincidência desta família neste módulo.

use super::*;

/// ⭐⭐⭐ **O RAIO DA JUNÇÃO viaja do RETRATO até ao documento COZIDO** (W98) — e a forma ao lado não
/// se mexe.
///
/// # ⚠️ Por que este gate é o da wave, e não os da crate
///
/// Os gates da `ph2d-field-ecs` provam a **lei** (materializa, preserva o carácter, não toca no
/// irmão). Este prova a **costura**: que a linha é de facto **publicada** no retrato que o painel
/// recebe, que o intent que ela cunha chega ao mundo, e que o **campo avaliado** muda por causa
/// disso. *Uma lei correcta que o painel não publica é um gesto que ninguém alcança* — é a quinta
/// reincidência desta família neste módulo, e a razão de o `field3d_reach_tests` existir.
#[test]
fn the_joint_radius_travels_from_the_snapshot_to_the_cooked_document() {
    let _ = ph2d_panel_model3d::drain_intents();
    let mut sim = a_world();
    // A cena 7 é a do verbo por forma: quatro irmãos, e o 3.º corta.
    sync_scene(&mut sim, Some(&scene(7)), 0.0);
    let root = the_root(&mut sim);
    let irmaos: Vec<_> = sim
        .world()
        .get::<bevy_ecs::hierarchy::Children>(root)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();
    assert_eq!(irmaos.len(), 4, "a cena 7 tem quatro irmãos");
    let (base, calado, corta) = (irmaos[0], irmaos[1], irmaos[2]);

    // ── O retrato oferece a linha a quem se junta, e não à base ──
    crate::field3d_scene::sync_scene_and_birth(
        &mut sim,
        None,
        &[corta],
        0.0,
        &crate::field3d_scene::no_drawing(),
    );
    let linha = ph2d_panel_model3d::state::current()
        .rows
        .into_iter()
        .find(|r| r.param == ph2d_field::Param::Joint)
        .expect("a forma que corta tem de oferecer o raio da junção dela");
    assert_eq!(linha.entity, corta.to_bits());

    crate::field3d_scene::sync_scene_and_birth(
        &mut sim,
        None,
        &[base],
        0.0,
        &crate::field3d_scene::no_drawing(),
    );
    assert!(
        !ph2d_panel_model3d::state::current()
            .rows
            .iter()
            .any(|r| r.param == ph2d_field::Param::Joint),
        "a BASE não se junta a nada — a linha ali seria a affordance que mente"
    );

    // ── E o intent daquela linha chega ao campo ──
    let antes = crate::field3d_scene::sync_scene(&mut sim, None, 0.0).expect("cozinha");
    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetParam {
        entity: linha.entity,
        param: ph2d_field::Param::Joint,
        value: 0.25,
    });
    let depois = crate::field3d_scene::sync_scene_and_birth(
        &mut sim,
        None,
        &[corta],
        0.0,
        &crate::field3d_scene::no_drawing(),
    )
    .0
    .expect("cozinha");
    assert_ne!(
        antes, depois,
        "o raio da junção não chegou ao documento — a linha é pintada e muda nada"
    );

    // ⭐ **O CONTROLO**: o irmão CALADO continua calado. Sem isto, uma escrita que fosse parar ao
    // grupo passaria no `assert_ne!` acima com o defeito que a wave inteira existe para curar.
    assert_eq!(
        ph2d_field_ecs::verb_of(sim.world(), calado),
        None,
        "o irmão calado ganhou um verbo — a escrita foi para o sítio errado"
    );
}
