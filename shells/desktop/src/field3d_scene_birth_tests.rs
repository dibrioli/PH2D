//! ⭐ **O QUE A PEÇA JÁ TEM QUANDO NASCE** — a selecção que aparece sem ninguém adivinhar o gesto.
//!
//! # Por que um arquivo irmão
//!
//! O [`super::field3d_scene_tests`] responde *«o que a cena faz quando alguém mexe»*; este responde
//! *«o que ela já é antes de alguém mexer»*. O arquivo cruzou as `600` linhas do gate de LOC do
//! shell nesta wave (`599 → 607`). ⛔ *Split, nunca allowlist.*

use super::*;

/// ⭐ **A peça nasce com um objeto selecionado** — as setas aparecem sem ninguém adivinhar o gesto.
///
/// ⚠️ E o selecionado é um **filho**, não a raiz: a raiz é o grupo inteiro, e um gizmo em cima dela
/// move a peça toda. Quem abre o módulo pela primeira vez quer ver o que uma seta faz a **uma**
/// forma.
///
/// ⚠️ **Uma vez, e só nessa.** Re-selecionar todo quadro tiraria da mão do artista o direito de
/// escolher outro objeto — o mesmo defeito que o painel de modelagem já pagou ao reabrir sozinho.
#[test]
fn the_part_is_born_with_an_object_selected_once_and_only_once() {
    let _ = ph2d_panel_model3d::drain_intents();
    let mut sim = a_world();
    let (_, born) = crate::field3d_scene::sync_scene_and_birth(
        &mut sim,
        Some(&scene(1)),
        &[],
        0.0,
        &crate::field3d_scene::no_drawing(),
    );
    let crate::field3d_scene::SelectRequest::Entity(bits) =
        born.expect("nascer tem de pedir uma seleção")
    else {
        panic!("nascer pede uma ENTIDADE, não uma limpeza");
    };

    let root = the_root(&mut sim);
    let world = sim.world_mut();
    let e = bevy_ecs::entity::Entity::from_bits(bits);
    assert!(world.get::<FieldNode>(e).is_some(), "o selecionado é um nó");
    assert_ne!(e, root, "e não é a raiz — a raiz é o grupo inteiro");
    assert_eq!(
        world.get::<ChildOf>(e).map(|c| c.0),
        Some(root),
        "é um filho direto da peça"
    );

    let (_, again) = crate::field3d_scene::sync_scene_and_birth(
        &mut sim,
        None,
        &[],
        0.0,
        &crate::field3d_scene::no_drawing(),
    );
    assert_eq!(
        again, None,
        "o quadro seguinte não volta a mandar selecionar"
    );
}
