//! ⭐⭐⭐ **PRENDER COZE OS EFEITOS** (ordem do dono, 2026-10-03: *«ao aplicar os bones, os efeitos são
//! cozidos antes. E uma vez com bones, o vetor não pode receber efeitos»*) — num irmão pelo tecto de
//! LOC do `skin_live_tests.rs`.

use super::bind;
use super::tests::palco;
use crate::test_support::{pior_desvio_do_desenho, quadro};
use ph2d_skeleton_ecs::SkinBind;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};

fn twist() -> PathEffect {
    PathEffect::Twist(ph2d_vec_scene::fx_twist::TwistSpec { angle: 40.0 })
}

/// ⭐⭐⭐ **GATE — o Bind coze o efeito no desenho, a pilha sai vazia na cena E na fonte guardada, e
/// prender não move um pixel.**
///
/// ⛔ **O CONTROLO:** o efeito muda MESMO o desenho — senão «não moveu» mediria uma forma igual.
#[test]
fn binding_bakes_the_effects_into_the_drawing_and_moves_nothing() {
    let (mut sim, mut scene, map, id, _) = palco();
    let liso = scene.paths()[0].clone();
    scene.path_mut(id).expect("path").effects = vec![FxEntry::new(twist())];
    let visto = scene.paths()[0].cooked().into_owned();
    assert!(
        pior_desvio_do_desenho(&liso, &visto) > 0.5,
        "o CONTROLO: o Twist não muda o desenho — a fixtura perdeu o efeito"
    );
    assert_eq!(bind(&mut sim, &mut scene, &map, &[id], None), 1);
    assert!(
        scene.path(id).expect("path").effects.is_empty(),
        "a pilha sobreviveu ao Bind na cena"
    );
    let e = ph2d_ecs::Entity::from_bits(map[&id]);
    let fonte = crate::skinned_mesh::le(&sim.world().get::<SkinBind>(e).expect("pele").source)
        .expect("a fonte lê-se");
    assert!(fonte.path.effects.is_empty(), "a fonte guardada leva a pilha");
    let depois = quadro(&sim, &mut scene, id);
    let pior = pior_desvio_do_desenho(&visto, &depois);
    assert!(pior < 1e-9, "prender com efeito moveu o desenho em {pior}");
}

/// ⭐⭐ **GATE — um efeito DESLIGADO sai com a pilha e não coze nada** (o olho fechado não é desenho).
#[test]
fn binding_drops_a_disabled_effect_without_baking_it() {
    let (mut sim, mut scene, map, id, _) = palco();
    let liso = scene.paths()[0].clone();
    scene.path_mut(id).expect("path").effects = vec![FxEntry {
        enabled: false,
        ..FxEntry::new(twist())
    }];
    assert_eq!(bind(&mut sim, &mut scene, &map, &[id], None), 1);
    assert!(scene.path(id).expect("path").effects.is_empty());
    let depois = quadro(&sim, &mut scene, id);
    let pior = pior_desvio_do_desenho(&liso, &depois);
    assert!(pior < 1e-9, "o efeito desligado foi cozido ({pior})");
}
