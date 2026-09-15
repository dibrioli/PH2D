//! Os gates da autoria da peça — a **segunda metade** (ver [`super::scene_edit_tests`]).
//!
//! ⚠️ **A partição é por TETO DE LINHAS e não por assunto**, e dizê-lo é mais honesto do que
//! inventar uma fronteira: as fixtures moram no pai e os dois ficheiros exercitam o mesmo gesto de
//! autoria. *Quem acrescentar um gate escolhe o ficheiro pelo tamanho, e o pai é onde a família
//! começa.*

use super::*;

/// ⭐ **Apagar leva o que está debaixo junto**, e a peça continua válida.
#[test]
fn deleting_a_group_takes_its_children_with_it() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let kids: Vec<bevy_ecs::entity::Entity> = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .collect();
    let group = ph2d_field_ecs::wrap_in_op(
        world,
        &kids[..2],
        ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
    )
    .expect("embrulha");

    assert!(ph2d_field_ecs::remove(world, group));
    assert!(world.get_entity(group).is_err(), "o grupo saiu");
    for k in &kids[..2] {
        assert!(world.get_entity(*k).is_err(), "os filhos foram com ele");
    }
    // Sobra o terceiro cilindro, e a peça é válida.
    assert_eq!(ph2d_field_ecs::walk(world, root).len(), 2);
    assert!(
        ph2d_field_ecs::cook(world, root)
            .expect("não vazia")
            .is_ok()
    );
}

/// ⭐ **Apagar a peça na Hierarquia apaga-a de VERDADE** — ela não volta no quadro seguinte.
///
/// ⚠️ Era um bug, e o comentário do código afirmava o contrário do que o código fazia. A ponte
/// oferecia o documento **cozido** como semente («a peça inicial»), e o comentário dizia que ele
/// *"deixa de existir"* — o que nunca foi verdade: ele é reescrito a cada quadro. Apagar a raiz
/// deixava a ponte sem raiz, e ela **replantava o que tinha acabado de cozer**.
///
/// *Uma semente usa-se uma vez.*
/// ⚠️ **Passa pelo `ecs_bridge`**, e não pela metade de baixo: a decisão que estava errada era
/// *o que a ponte oferece como semente*, e um gate que passasse `None` à mão nunca lhe chegaria.
/// (Foi assim que a primeira versão deste gate ficou verde com o bug reposto.)
#[test]
fn deleting_the_part_does_not_replant_it_next_frame() {
    let _ = ph2d_panel_model3d::drain_intents();
    crate::smoke::set_armed_by_panel(true);
    let mut sim = a_world();

    // Quadro 1: a ponte planta a semente.
    crate::scene::ecs_bridge(&mut sim, None, &[], &crate::scene::no_drawing());
    let root = {
        let world = sim.world_mut();
        let mut q = world.query::<(bevy_ecs::entity::Entity, &FieldObject)>();
        q.iter(world).next().map(|(e, _)| e).expect("a peça nasceu")
    };

    // A Hierarquia apaga a peça (cascata: a raiz leva os filhos).
    sim.world_mut().despawn(root);

    // Quadro 2: a ponte corre outra vez — e **não replanta**.
    crate::scene::ecs_bridge(&mut sim, None, &[], &crate::scene::no_drawing());
    let world = sim.world_mut();
    let mut q = world.query::<&FieldObject>();
    assert_eq!(
        q.iter(world).count(),
        0,
        "a peça voltou — a ponte replantou o que tinha acabado de cozer"
    );
}

/// ⭐ **Duplicar pela Hierarquia e pelo painel é a MESMA porta.**
///
/// ⚠️ O braço genérico da Hierarquia copia `Transform` + `Sprite` + `Name`. Um nó de campo **não tem
/// nenhum dos dois** — sairia uma linha na Hierarchy sobre geometria nenhuma, invisível para o
/// traçado. É o mesmo defeito que a nota vetorial daquele bloco já descreve, no módulo seguinte.
///
/// O gate mede o que a cópia **é**, e não que ela existe: sem `FieldNode` ela é o sósia.
#[test]
fn a_duplicate_is_a_real_node_not_a_nameless_twin() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let first = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .next()
        .expect("o primeiro");

    let bits =
        crate::scene::duplicate_with_view(world, first, &a_view().0, a_view().1).expect("duplica");
    let copy = bevy_ecs::entity::Entity::from_bits(bits);

    assert!(
        world.get::<FieldNode>(copy).is_some(),
        "a cópia tem de SER um nó — sem isto ela é uma linha na Hierarchy sobre nada"
    );
    assert!(world.get::<FieldPose>(copy).is_some(), "e ter pose própria");
    assert_eq!(world.get::<ChildOf>(copy).map(|c| c.0), Some(root));
    // E ela entra na peça: o cozimento tem de a conter.
    let cooked = ph2d_field_ecs::cook(world, root)
        .expect("não vazia")
        .expect("válida");
    assert_eq!(
        cooked.nodes().len(),
        5,
        "três cilindros + a cópia + a união"
    );
}

/// ⚠️ **A porta da Hierarquia é a MESMA do painel**, e o gate prende-o: as duas cópias saem no mesmo
/// sítio relativo.
///
/// Duas contas para *"onde vai a cópia?"* divergiriam no primeiro ajuste — com o artista a ver o
/// mesmo gesto fazer duas coisas conforme por onde o pediu.
#[test]
fn both_doors_put_the_copy_in_the_same_place() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    let kids: Vec<bevy_ecs::entity::Entity> = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .collect();

    let offset_of = |world: &mut bevy_ecs::world::World, src: bevy_ecs::entity::Entity| {
        let before = ph2d_field_ecs::world_xform(world, src).translation;
        let bits = crate::scene::duplicate_with_view(world, src, &a_view().0, a_view().1)
            .expect("duplica");
        let after = ph2d_field_ecs::world_xform(world, bevy_ecs::entity::Entity::from_bits(bits))
            .translation;
        [
            after[0] - before[0],
            after[1] - before[1],
            after[2] - before[2],
        ]
    };
    let a = offset_of(world, kids[0]);
    let b = offset_of(world, kids[1]);
    for k in 0..3 {
        assert!(
            (a[k] - b[k]).abs() < 1e-6,
            "as duas cópias saíram em sítios diferentes: {a:?} e {b:?}"
        );
    }
    // E o deslocamento não é zero — senão o gate passaria com as duas portas a não fazer nada.
    assert!(
        a.iter().any(|v| v.abs() > 1e-6),
        "a cópia ficou em cima do original"
    );
}

/// ⭐ **Digitar uma posição move o objeto** — o par da W10, que deu o tamanho e não a pose.
///
/// ⚠️ A posição é **LOCAL**, e é a convenção da casa: o Inspector dela mostra o `Transform`, que é
/// local, e o readout do gizmo 2D diz por extenso que o delta é local *"porque é isso que o
/// Inspector mostra"*. Um painel que mostrasse mundo contradiria o número ao lado no dia em que
/// alguém agrupasse — e o gate prova-o com um pai deslocado.
#[test]
fn typing_a_position_moves_the_node_in_its_parents_frame() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    // O grupo sai do zero: se as linhas mostrassem MUNDO, elas passariam a incluir este offset.
    ph2d_field_ecs::set_param(world, root, ph2d_field::Param::Pos(0), 1.0)
        .expect("a raiz tem pose");

    let child = world
        .get::<Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .next()
        .expect("o primeiro");
    ph2d_field_ecs::set_param(world, child, ph2d_field::Param::Pos(2), 0.4).expect("escreve Z");

    let local = world
        .get::<FieldPose>(child)
        .expect("pose")
        .xform
        .translation;
    assert!((local[2] - 0.4).abs() < 1e-6, "o número escrito é o local");

    // E o painel mostra o mesmo número — não o de mundo, que aqui é outro.
    let shown = ph2d_field_ecs::params_of(world, child)
        .into_iter()
        .find(|(p, _)| *p == ph2d_field::Param::Pos(0))
        .map(|(_, d)| d.value)
        .expect("a linha de X");
    assert!(
        (shown - local[0]).abs() < 1e-6,
        "o painel mostra {shown} e a pose local é {}",
        local[0]
    );
    let world_x = ph2d_field_ecs::world_xform(world, child).translation[0];
    assert!(
        (world_x - local[0]).abs() > 0.5,
        "a fixture tem de ter mundo != local, senão o gate não distingue os dois"
    );
}

/// ⛔ **Uma escala não-positiva é recusada pela porta do painel**, e o nó fica como estava.
#[test]
fn typing_a_non_positive_scale_is_refused() {
    let mut sim = a_world();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &scene(1), "Model");
    for bad in [0.0f32, -2.0, f32::NAN] {
        assert!(ph2d_field_ecs::set_param(world, root, ph2d_field::Param::Scale, bad).is_err());
    }
    assert!((world.get::<FieldPose>(root).expect("pose").xform.scale - 1.0).abs() < 1e-6);
}

#[path = "scene_mods_tests.rs"]
mod mods;

/// ⭐⭐⭐ **UM VERBO SEM SUJEITO NÃO SE OFERECE** (`docs/Render3d/05` §28).
///
/// Uma lâmpada de ponto não tem orientação nem tamanho: com ela escolhida, *Rotate* e *Scale* eram
/// alças desenhadas que não moviam número nenhum.
#[test]
fn uma_luz_nao_oferece_rodar_nem_escalar() {
    use crate::gizmo::Mode;
    let mut world = bevy_ecs::world::World::new();
    let luz = ph2d_field_ecs::add_light(
        &mut world,
        [0.0, 1.0, 0.0],
        ph2d_field_ecs::FieldLight {
            intensity: 1.0,
            color: [1.0; 3],
        },
    );
    assert_eq!(
        crate::scene::offered_verbs(&world, &[luz]),
        vec![Mode::Move],
        "com uma luz escolhida só MOVER tem sujeito"
    );

    // ⭐ O controlo: uma FORMA oferece os três — senão o gate acima passaria por esconder tudo.
    let raiz = ph2d_field_ecs::spawn_doc(&mut world, &crate::smoke::scene(1), "Model");
    let forma = *world
        .get::<bevy_ecs::hierarchy::Children>(raiz)
        .expect("a peça tem filhos")
        .first()
        .expect("a peça tem pelo menos uma forma");
    assert_eq!(
        crate::scene::offered_verbs(&world, &[forma]),
        Mode::ALL.to_vec(),
        "uma forma tem orientação e tamanho: os três verbos têm sujeito"
    );

    // ⚠️ **A regra é sobre a selecção INTEIRA.** Com a luz E a forma escolhidas, rodar tem sujeito
    // (a forma), e o gizmo pousa no meio das duas — tirar o verbo ali seria tirar um gesto legítimo
    // por causa de um acompanhante.
    assert_eq!(
        crate::scene::offered_verbs(&world, &[luz, forma]),
        Mode::ALL.to_vec(),
        "uma luz ao lado de uma forma não pode apagar o verbo da forma"
    );

    // ⚠️ E sem selecção nenhuma a fileira não pisca.
    assert_eq!(crate::scene::offered_verbs(&world, &[]), Mode::ALL.to_vec());
}

/// ⭐⭐⭐ **O `slot` do clique indexa a lista OFERECIDA, e não a `Mode::ALL`.**
///
/// ⛔ Este gate existe porque as duas **coincidem hoje** por acidente: com uma luz escolhida sobra
/// só o `Move`, que já é o índice `0` de ambas. *Uma correspondência que se mantém por o primeiro
/// elemento não mudar não é uma lei — é uma coincidência à espera de um verbo novo.*
#[test]
fn o_slot_do_verbo_le_a_lista_que_foi_oferecida() {
    use crate::gizmo::Mode;
    let mut world = bevy_ecs::world::World::new();
    let raiz = ph2d_field_ecs::spawn_doc(&mut world, &crate::smoke::scene(1), "Model");
    let forma = *world
        .get::<bevy_ecs::hierarchy::Children>(raiz)
        .expect("filhos")
        .first()
        .expect("uma forma");
    let luz = ph2d_field_ecs::add_light(
        &mut world,
        [0.0, 1.0, 0.0],
        ph2d_field_ecs::FieldLight {
            intensity: 1.0,
            color: [1.0; 3],
        },
    );

    // Com a FORMA, o chip na posição 1 é o que o painel chama de `rotate`.
    let com_forma = crate::scene::offered_verbs(&world, &[forma]);
    assert_eq!(com_forma.get(1).copied(), Some(Mode::Rotate));
    assert_eq!(com_forma[1].key(), "panel.model3d.mode.rotate");

    // Com a LUZ, a posição 1 **não existe** — e é isso que impede o clique de pôr um verbo que a
    // fileira nunca mostrou.
    let com_luz = crate::scene::offered_verbs(&world, &[luz]);
    assert_eq!(com_luz.len(), 1);
    assert_eq!(com_luz.get(1).copied(), None);

    // ⛔⛔⛔ **E AQUI CORRE O CONSUMIDOR, porque sem isto a mutação SOBREVIVE.**
    //
    // A 1.ª redacção deste gate parava nas três linhas acima — ele media a função pura e nunca o
    // braço que a lê. Repor o `Mode::ALL.get(slot)` no dreno deixava-o **verde**, que é a costura
    // não-testada da `DIRETIVA_IMPLEMENTACAO` §1 outra vez. *Um gate que só mede a porta não prova
    // que alguém a usa.*
    //
    // O clique que a fileira NUNCA pintou: com uma luz escolhida ela tem um chip só, logo o `slot`
    // `1` não é alcançável por gesto nenhum — e é precisamente por isso que ele é o discriminador:
    // com a lista certa não acontece nada; com a `Mode::ALL` ele põe **Rotate**.
    // ⚠️ **O módulo tem de estar ARMADO**: fora do pill o `with_smoke` devolve `None` e todo
    // gancho é inerte — de propósito (W42). Sem isto o gate lia `None` nos dois lados e passava
    // por não medir nada.
    crate::smoke::set_armed_by_panel(true);
    crate::smoke::with_smoke(|sm| sm.gizmo_mode = Mode::Move);
    ph2d_panel_model3d::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetGizmoMode {
        slot: 1,
    });
    crate::scene::apply_intents_for_test(&mut world, &[luz]);
    assert_eq!(
        crate::smoke::with_smoke(|sm| sm.gizmo_mode),
        Some(Mode::Move),
        "o clique num slot que a fileira não ofereceu armou um verbo — o dreno está a indexar uma \
         SEGUNDA lista"
    );

    // ⭐ O controlo: com a FORMA escolhida o MESMO slot `1` arma o Rotate, senão este gate passaria
    // por o dreno estar simplesmente morto.
    ph2d_panel_model3d::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetGizmoMode {
        slot: 1,
    });
    crate::scene::apply_intents_for_test(&mut world, &[forma]);
    assert_eq!(
        crate::smoke::with_smoke(|sm| sm.gizmo_mode),
        Some(Mode::Rotate),
        "com uma forma o slot 1 É o Rotate — se isto falhar o dreno não está a fazer nada"
    );

    // Deixa o módulo como o encontrou — os gates deste processo partilham o `thread_local`.
    crate::smoke::set_armed_by_panel(false);
    let _ = crate::smoke::with_smoke(|_| ());

    // ⭐ E a chave de cada posição oferecida é a que o painel publica — é por ela que o trilho
    // (`area_bar::rail_verb_slot`) reencontra o chip, e é o que liga as duas pontas.
    for (i, m) in com_forma.iter().enumerate() {
        assert_eq!(
            m.key(),
            Mode::ALL[i].key(),
            "com uma forma a oferta é a lista inteira, na ordem dela"
        );
    }
}
