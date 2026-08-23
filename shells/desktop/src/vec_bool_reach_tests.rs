//! **A BOOLEANA VIVA ALCANÇA OS ESTADOS** — os gates de ROTA da auditoria de 2026-08-23.
//!
//! # A pergunta que estes gates fazem
//!
//! *Com uma booleana em mãos, o artista consegue chegar às poses dela?*
//!
//! ⚠️ Ela não é a mesma que *"o widget existe, é pintado e o clique chega ao barramento"* — essas
//! três estavam **verdes** enquanto a resposta a esta era **não**. Tocar um operando seleciona o
//! GRUPO inteiro (lei deliberada do `object_selection_for`), o `publish` exigia forma ÚNICA, e a
//! seção STATES — com o interruptor **Preview** dentro dela — não era sequer pintada. Não dimmed,
//! não vazia: ausente, sem uma palavra a dizer o que faltava.
//!
//! É a família do [`crate::field3d_reach_tests`]: o que se mede aqui é a SEQUÊNCIA.

use crate::vec_entities::VecEntityMap;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, VecBoolGroup};
use ph2d_ui_state::StateSets;
use ph2d_vec_scene::{VecPathId, VecScene, rectangle};

/// A disposição que a cena `=74` monta: um CHIP com uma booleana viva pendurada nele.
///
/// Devolve `(sim, scene, map, chip, [operando de baixo, operando de cima])`.
fn chip_with_a_live_boolean() -> (SimWorld, VecScene, VecEntityMap, VecPathId, [VecPathId; 2]) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let chip = scene.push_path(rectangle([-2.0, -2.0], [22.0, 22.0]));
    let outer = scene.push_path(rectangle([0.0, 0.0], [20.0, 20.0]));
    let inner = scene.push_path(rectangle([6.0, 6.0], [14.0, 14.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    for (id, name) in [(chip, "Chip"), (outer, "Big"), (inner, "Hole")] {
        sim.world_mut()
            .entity_mut(Entity::from_bits(map[&id]))
            .insert(Name::new(name));
    }
    let g = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&outer], map[&inner]], "Bool".into())
            .unwrap(),
    );
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op: 0 });
    crate::vec_transform::reparent_keeping_world(&mut sim, g, Entity::from_bits(map[&chip]));
    let _ = Transform::IDENTITY;
    (sim, scene, map, chip, [outer, inner])
}

/// ⭐ **TOCAR A BOOLEANA CHEGA À SEÇÃO STATES, e ela NOMEIA o chip.**
///
/// ⚠️ A seleção que um clique de facto produz é a do `object_selection_for` — o grupo inteiro —, e
/// é essa que entra aqui. Um gate que passasse `&[chip]` à mão mediria uma seleção que **nenhum
/// gesto do editor produz**, que foi exactamente como os quatro chips do verbo por forma shiparam
/// mortos em 22/08.
#[test]
fn touching_a_live_boolean_reaches_the_states_section() {
    let (sim, scene, map, chip, ops) = chip_with_a_live_boolean();
    let selection = crate::vec_entities::object_selection_for(&sim, &scene, &map, ops[1]);
    assert!(
        selection.len() > 1,
        "a fixture não reproduz o clique: tocar um operando tem de acender o GRUPO, e acendeu {:?}",
        selection
    );

    let published = crate::vec_ui_state_edit::publish(
        &sim,
        &scene,
        &map,
        &selection,
        &StateSets::default(),
        None,
        false,
        false,
    )
    .expect("a seccao STATES tem de EXISTIR com uma booleana em maos");
    assert_eq!(
        published.host.as_deref(),
        Some("Chip"),
        "a seccao apareceu sem dizer de QUEM sao as poses"
    );
    assert_eq!(
        crate::vec_ui_state_edit::host_of_selection(&sim, &scene, &map, &selection),
        Some(chip),
        "o hospedeiro derivado nao e' a forma que contem a booleana"
    );
}

/// ⭐ **E O QUE O `Rec` GRAVA É A BOOLEANA INTEIRA** — a outra metade da rota.
///
/// ⚠️ Sem ela, a seção poderia aparecer e gravar as poses **erradas**: o hospedeiro é derivado, e
/// o que ele governa tem de conter os operandos, senão o artista anima um chip vazio e não percebe
/// porquê.
#[test]
fn recording_from_a_boolean_selection_captures_the_operands() {
    let (mut sim, mut scene, map, _chip, ops) = chip_with_a_live_boolean();
    let selection = crate::vec_entities::object_selection_for(&sim, &scene, &map, ops[0]);
    let mut states = StateSets::default();
    crate::vec_ui_state_edit::apply(
        &mut sim,
        &mut scene,
        &map,
        &selection,
        &mut states,
        crate::vec_ui_state_edit::UiStateEdit::Record(ph2d_ui_state::StateRole::Default),
    );
    let recorded: Vec<VecPathId> = states
        .role(_chip, ph2d_ui_state::StateRole::Default)
        .expect("o Rec nao gravou nada")
        .objects
        .iter()
        .map(|o| o.id)
        .collect();
    for id in ops {
        assert!(
            recorded.contains(&id),
            "o operando {id} ficou de fora da pose: o estado animaria um chip vazio"
        );
    }
    // ⭐ E as poses dos operandos trazem a operação do grupo — senão trocá-la num estado não anima.
    let op = states
        .role(_chip, ph2d_ui_state::StateRole::Default)
        .and_then(|s| {
            s.objects
                .iter()
                .find(|o| o.id == ops[1])
                .map(|o| o.bool_group_op)
        });
    assert_eq!(
        op,
        Some(Some(0)),
        "a pose do operando nao carrega a operacao do grupo"
    );
}

/// ⭐⭐ **AS DUAS SUPERFÍCIES ESTÃO NA TELA AO MESMO TEMPO.**
///
/// ⚠️ Autorar *"no Hover esta forma subtrai"* precisa das duas: a fileira **This Shape** (que
/// exige que o PRIMÁRIO seja um operando) e a seção **STATES** (que exige um hospedeiro). Enquanto
/// o hospedeiro foi *"a forma única selecionada"*, as duas condições eram **mutuamente exclusivas
/// por construção** — o artista tinha de trocar de seleção no meio de um único ato de autoria, e
/// nada na tela dizia isso.
#[test]
fn the_verb_row_and_the_states_section_coexist_on_one_selection() {
    let (sim, scene, map, _chip, ops) = chip_with_a_live_boolean();
    let selection = crate::vec_entities::object_selection_for(&sim, &scene, &map, ops[1]);

    // O plano do quadro: é dele que o papel de cada forma sai.
    let mut live = ph2d_vec_render::LiveGeometry::new();
    let mut bl = crate::bool_live::BoolLive::default();
    bl.recook(
        &scene,
        &sim,
        &map,
        &ph2d_vec_scene::VecXforms::default(),
        &[],
        &mut live,
    );

    let row =
        crate::vec_bool_shape::shape_row_of_selection(&sim, &map, &bl, &selection, Some(ops[1]));
    assert!(
        row.is_some(),
        "a fileira do verbo por forma nao aparece sobre a selecao que o clique produz"
    );
    // ⚠️ **A barra é ter HOSPEDEIRO, e não apenas `Some`.** A face vazia também é um `Some` — um
    // gate que só perguntasse *"a seção existe?"* ficaria verde sobre a seção a dizer *"nenhuma
    // forma governa esta seleção"*, que é exactamente o estado que esta wave veio curar. Medido:
    // um mutante que devolvia `None` no braço da seleção múltipla **sobreviveu** à primeira versão.
    let published = crate::vec_ui_state_edit::publish(
        &sim,
        &scene,
        &map,
        &selection,
        &StateSets::default(),
        None,
        false,
        false,
    )
    .expect("a seccao STATES nao aparece sobre a MESMA selecao");
    assert!(
        published.host.is_some(),
        "a seccao apareceu com a FACE VAZIA: autorar um verbo por estado exigiria trocar de \
         seleccao no meio do gesto"
    );
}
