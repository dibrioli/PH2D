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
        ph2d_field_ecs::mods_of(world, root)[0].dims()[0].value
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
    assert_eq!(param, Param::Mod { slot: 0, field: 0 });
    assert_eq!(dim.key, "field.mod.thickness");

    ph2d_field_ecs::set_param(world, root, Param::Mod { slot: 0, field: 0 }, 0.07)
        .expect("escreve a espessura");
    assert!((ph2d_field_ecs::mods_of(world, root)[0].dims()[0].value - 0.07).abs() < 1e-6);

    // ⛔ E uma espessura não-positiva é recusada, deixando o nó como estava.
    assert!(ph2d_field_ecs::set_param(world, root, Param::Mod { slot: 0, field: 0 }, 0.0).is_err());
    assert!((ph2d_field_ecs::mods_of(world, root)[0].dims()[0].value - 0.07).abs() < 1e-6);
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

/// ⭐ **Um modificador pode ter VÁRIOS números — ou nenhum.**
///
/// ⚠️ É o que a matriz forçou, e o gate mede as duas pontas na mesma corrida: o espelho não põe
/// linha nenhuma (o chip aceso já diz tudo), e a matriz põe duas. Um gate só sobre a matriz passaria
/// com um `flat_map` que inventasse uma linha vazia para o espelho.
#[test]
fn a_modifier_can_have_several_numbers_or_none_at_all() {
    use ph2d_field::{Param, UnaryKind};
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(2), "Model");

    let rows_of = |world: &bevy_ecs::world::World| -> Vec<Param> {
        ph2d_field_ecs::params_of(world, root)
            .into_iter()
            .map(|(p, _)| p)
            .filter(|p| matches!(p, Param::Mod { .. }))
            .collect()
    };

    ph2d_field_ecs::add_mod(world, root, UnaryKind::Mirror);
    assert!(
        rows_of(world).is_empty(),
        "o espelho não tem número nenhum — o chip aceso é a única coisa que há para dizer"
    );

    ph2d_field_ecs::add_mod(world, root, UnaryKind::Array);
    assert_eq!(
        rows_of(world),
        vec![
            Param::Mod { slot: 1, field: 0 },
            Param::Mod { slot: 1, field: 1 },
        ],
        "a matriz põe DUAS linhas, e no slot 1 — o espelho continua a ocupar o slot 0"
    );
}

/// ⭐ **A contagem de cópias é INTEIRA, e a linha do painel diz-se inteira.**
///
/// ⚠️ Três coisas dependem disto e nenhuma se deduz do valor: o passo do arrasto é **1**, o número
/// mostra-se **sem casas**, e o piso é **1**. Deduzir *"parece inteiro, logo é"* daria uma linha que
/// muda de comportamento quando o valor calha em `3,0`.
#[test]
fn the_copy_count_is_an_integer_row_with_a_floor_of_one() {
    use ph2d_field::{Param, UnaryKind};
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(2), "Model");
    ph2d_field_ecs::add_mod(world, root, UnaryKind::Array);
    const VIEW: f32 = 2.5;

    let count_row = |world: &bevy_ecs::world::World| {
        crate::field3d_scene::panel::param_rows(world, Some(root), VIEW)
            .into_iter()
            .find(|r| r.param == Param::Mod { slot: 0, field: 0 })
            .expect("a linha da contagem")
    };
    let row = count_row(world);
    assert!(row.integral, "a contagem é inteira");
    assert_eq!(
        row.lo, 1.0,
        "o piso é UMA cópia — zero é a peça a desaparecer"
    );
    assert!(row.bound.value() >= 2.0, "e há teto: {:?}", row.bound);

    // Escrever um fracionário arredonda; escrever abaixo de 1 é recusado.
    ph2d_field_ecs::set_param(world, root, Param::Mod { slot: 0, field: 0 }, 4.4)
        .expect("escreve a contagem");
    assert_eq!(count_row(world).value, 4.0, "4,4 cópias são 4 cópias");
    assert!(
        ph2d_field_ecs::set_param(world, root, Param::Mod { slot: 0, field: 0 }, 0.0).is_err(),
        "zero cópias é recusado — apagar já tem botão"
    );
    assert_eq!(count_row(world).value, 4.0, "e a recusa deixa como estava");
}
