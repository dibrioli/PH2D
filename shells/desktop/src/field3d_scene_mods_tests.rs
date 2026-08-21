//! Os gates dos **modificadores** — a casca e o afastamento, do botão ao documento.
//!
//! ⚠️ Módulo-filho do arquivo de gates da autoria: `use super::*` traz as fixtures do pai, que
//! continuam a existir **uma vez**.

use super::*;

/// ⭐ **O botão de modificador é um INTERRUPTOR**: liga, e o segundo clique desliga.
///
/// ⚠️ O gate corre pelo caminho de produção inteiro — intent do painel → `ecs_bridge` → mundo —,
/// porque a metade que pode partir é a **costura**: uma ordem trocada no braço (acrescentar antes de
/// tirar) acrescentaria um segundo modificador e tiraria o primeiro no mesmo clique, e da tela isso
/// lê como *"não aconteceu nada"*.
#[test]
fn the_modifier_button_is_a_switch_not_a_stack_of_shells() {
    use ph2d_field::UnaryKind;
    let _ = ph2d_panel_model3d::drain_intents();
    let mut sim = a_world();
    crate::field3d_scene::sync_scene(&mut sim, Some(&scene(2)), 0.0);
    let root = the_root(&mut sim);

    let toggle = |sim: &mut SimWorld, slot: usize| {
        ph2d_panel_model3d::state::push_intent_for_test(
            ph2d_panel_model3d::ModelIntent::ToggleMod { slot },
        );
        crate::field3d_scene::sync_scene_and_birth(sim, None, &[root], 0.0);
    };
    let stack = |sim: &mut SimWorld| ph2d_field_ecs::mods_of(sim.world(), root);

    assert!(stack(&mut sim).is_empty(), "um nó nasce sem modificador");
    toggle(&mut sim, 0);
    assert_eq!(stack(&mut sim).len(), 1, "o primeiro clique acrescenta");
    assert_eq!(stack(&mut sim)[0].kind(), UnaryKind::Shell);
    toggle(&mut sim, 0);
    assert!(
        stack(&mut sim).is_empty(),
        "o segundo clique TIRA — senão o artista empilha cascas sem perceber"
    );

    // E as duas naturezas convivem: ligar uma não desliga a outra.
    toggle(&mut sim, 0);
    toggle(&mut sim, 1);
    let both = stack(&mut sim);
    assert_eq!(both.len(), 2, "casca e afastamento coexistem: {both:?}");
    assert_eq!(both[0].kind(), UnaryKind::Shell);
    assert_eq!(both[1].kind(), UnaryKind::Offset);
}

/// ⭐ **Uma casca nasce VISÍVEL** — a espessura vem do tamanho da peça, não de uma constante.
///
/// ⚠️ Um número absoluto seria invisível numa peça grande e engoliria uma pequena, e nos dois casos
/// o artista conclui que o botão não fez nada. O gate mede a razão em **duas peças de escalas
/// diferentes**: é a comparação que uma constante reprova e uma fração passa.
#[test]
fn a_shell_is_born_as_a_fraction_of_the_part_not_a_fixed_number() {
    use ph2d_field::{Primitive, UnaryKind};
    let born_on = |half: f32| -> f32 {
        let mut sim = a_world();
        let world = sim.world_mut();
        let doc = ph2d_field::FieldDoc::new(
            vec![ph2d_field::Node::new(
                ph2d_field::Xform::IDENTITY,
                ph2d_field::NodeKind::Leaf(Primitive::Box {
                    half: [half; 3],
                    round: 0.0,
                }),
            )],
            ph2d_field::NodeId(0),
        )
        .expect("caixa");
        let root = ph2d_field_ecs::spawn_doc(world, &doc, "Model");
        ph2d_field_ecs::add_mod(world, root, UnaryKind::Shell);
        ph2d_field_ecs::mods_of(world, root)[0].value()
    };
    let (small, big) = (born_on(0.1), born_on(1.0));
    assert!(
        (big / small - 10.0).abs() < 0.1,
        "a espessura tinha de acompanhar a peça (×10): {small} e {big}"
    );
    assert!(small > 0.0, "e uma casca de zero seria recusada pela porta");
}

/// ⭐ **O número do modificador chega ao painel e volta** — a linha, e a escrita nela.
///
/// ⚠️ A linha vem por **último** de propósito (ver `params_of`): primeiro o que a forma é, depois o
/// que se fez a ela. E o `Param::Mod` viaja por **posição na pilha**, não por natureza — duas cascas
/// no mesmo nó são duas linhas distintas, e uma chave por natureza escreveria as duas ao mesmo tempo.
#[test]
fn a_modifier_row_reaches_the_panel_and_takes_a_typed_number() {
    use ph2d_field::{Param, UnaryKind};
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(2), "Model");
    ph2d_field_ecs::add_mod(world, root, UnaryKind::Shell);

    let params = ph2d_field_ecs::params_of(world, root);
    let (param, dim) = *params.last().expect("a pilha entra no fim da lista");
    assert_eq!(param, Param::Mod(0));
    assert_eq!(dim.key, "field.mod.shell");

    ph2d_field_ecs::set_param(world, root, Param::Mod(0), 0.07).expect("escreve a espessura");
    assert!((ph2d_field_ecs::mods_of(world, root)[0].value() - 0.07).abs() < 1e-6);

    // ⛔ E uma espessura não-positiva é recusada, deixando o nó como estava.
    assert!(ph2d_field_ecs::set_param(world, root, Param::Mod(0), 0.0).is_err());
    assert!((ph2d_field_ecs::mods_of(world, root)[0].value() - 0.07).abs() < 1e-6);
}

/// ⚠️ **Tirar o último modificador TIRA o componente**, e não deixa uma pilha vazia.
///
/// O undo compara **bytes**: um componente presente e vazio não muda a forma e muda os bytes, então
/// acrescentar-e-tirar deixaria a peça diferente de si mesma e o desfazer teria um passo a mais do
/// que o artista fez.
#[test]
fn removing_the_last_modifier_removes_the_component_too() {
    use ph2d_field::UnaryKind;
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(2), "Model");
    let before = ph2d_field_ecs::cook(world, root)
        .expect("peça")
        .expect("válida");

    ph2d_field_ecs::add_mod(world, root, UnaryKind::Shell);
    assert!(ph2d_field_ecs::remove_mod(world, root, UnaryKind::Shell));
    assert!(
        !ph2d_field_ecs::remove_mod(world, root, UnaryKind::Shell),
        "tirar o que não está devolve false — é o que faz o interruptor ser honesto"
    );

    let after = ph2d_field_ecs::cook(world, root)
        .expect("peça")
        .expect("válida");
    assert_eq!(
        after, before,
        "a peça tem de voltar IDÊNTICA — o undo compara bytes"
    );
}
