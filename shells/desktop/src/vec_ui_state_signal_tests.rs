//! Gates da **TABELA SINAL → PAPEL** (item 4 do estudo dos contêineres) — a PORTA de autoria.
//!
//! Irmão do [`super::tests`] pelo teto de 600 LOC da shell, e o corte é por ASSUNTO: ali mora
//! *que poses uma forma tem e quem as grava*, aqui *o que a faz mudar de pose sozinha*.

use super::*;

/// Um mundo VAZIO: estes gates falam da PORTA, não da árvore. Uma seleção de uma forma é
/// hospedeira de si própria — o caso degenerado da lei do hospedeiro.
fn bare() -> (SimWorld, VecScene, VecEntityMap) {
    (SimWorld::new(), VecScene::default(), VecEntityMap::new())
}

/// **Cada id da tabela endereça o gesto certo** — e nada mais endereça gesto nenhum.
#[test]
fn every_table_id_addresses_its_own_gesture() {
    use crate::vec_ui_state_edit::{SignalEdit, signal_edit_for_id, signal_name_row};

    assert_eq!(
        signal_edit_for_id(ph2d_editor::ids::VECTOR_STATE_SIGNAL_ADD),
        Some(SignalEdit::Add)
    );
    for i in 0..ph2d_editor::ids::MAX_SIGNAL_BINDINGS {
        assert_eq!(
            signal_edit_for_id(ph2d_editor::ids::vector_state_signal_remove_id(i)),
            Some(SignalEdit::Remove(i)),
            "a lixeira da linha {i} nao endereca a propria linha"
        );
        for (r, role) in StateRole::ALL.iter().enumerate() {
            assert_eq!(
                signal_edit_for_id(ph2d_editor::ids::vector_state_signal_role_id(i, r)),
                Some(SignalEdit::Role(i, *role)),
                "o chip ({i}, {r}) nao endereca o proprio papel"
            );
        }
        assert_eq!(
            signal_name_row(ph2d_editor::ids::vector_state_signal_name_id(i)),
            Some(i)
        );
    }
    // ⚠️ O CONTROLE: um id vizinho da MESMA seção não pode cair na tabela — sem ele, uma porta
    // que dissesse `Some` para tudo passaria em todas as asserções acima.
    assert_eq!(
        signal_edit_for_id(ph2d_editor::ids::VECTOR_STATE_SPRING),
        None
    );
    assert_eq!(
        signal_name_row(ph2d_editor::ids::VECTOR_STATE_DURATION),
        None
    );
}

/// **Os três gestos escrevem o documento**, e o `Add` **para no teto do painel**.
///
/// ⚠️ A guarda do teto mora nos DOIS lados de propósito: o painel esconde o botão (a metade
/// visível) e a porta o honra (a metade que decide). Sem esta, um clique que chegasse por outra
/// rota cresceria a lista além do que a UI sabe mostrar — e as linhas extra ficariam invisíveis
/// no documento.
#[test]
fn the_three_gestures_write_the_table_and_add_stops_at_the_pool() {
    let (sim, scene, map) = bare();
    use crate::vec_ui_state_edit::{SignalEdit, apply_signal_edit};

    let host: VecPathId = 1;
    let sel = [host];
    let mut states = StateSets::default();

    assert!(apply_signal_edit(
        &sim,
        &scene,
        &map,
        &mut states,
        &sel,
        SignalEdit::Add
    ));
    assert_eq!(states.bindings(host).len(), 1);
    apply_signal_edit(
        &sim,
        &scene,
        &map,
        &mut states,
        &sel,
        SignalEdit::Role(0, StateRole::Pressed),
    );
    states.set_binding_name(host, 0, "open".into());
    assert_eq!(
        states.targets("open").collect::<Vec<_>>(),
        vec![(host, StateRole::Pressed)]
    );

    while states.bindings(host).len() < ph2d_editor::ids::MAX_SIGNAL_BINDINGS {
        assert!(apply_signal_edit(
            &sim,
            &scene,
            &map,
            &mut states,
            &sel,
            SignalEdit::Add
        ));
    }
    assert!(
        !apply_signal_edit(&sim, &scene, &map, &mut states, &sel, SignalEdit::Add),
        "o Add passou do pool que o painel sabe mostrar — as linhas extra ficariam invisiveis"
    );
    assert_eq!(
        states.bindings(host).len(),
        ph2d_editor::ids::MAX_SIGNAL_BINDINGS
    );

    apply_signal_edit(&sim, &scene, &map, &mut states, &sel, SignalEdit::Remove(0));
    assert_eq!(states.targets("open").count(), 0);
}

/// **Sem HOSPEDEIRO a porta recusa** — a tabela é por hospedeiro, e carimbar a mesma ligação em
/// vários seria um gesto cujo alcance o artista não vê.
///
/// ⚠️ **O que ela recusa deixou de ser *"mais de uma forma"*** (auditoria de 2026-08-23): uma
/// seleção múltipla governada por uma forma TEM hospedeiro, e é ela. O que continua sem resposta é
/// uma seleção que **nenhuma** forma governa — aqui, dois ids que o mundo vazio não relaciona.
#[test]
fn the_table_refuses_a_selection_with_no_host() {
    let (sim, scene, map) = bare();
    use crate::vec_ui_state_edit::{SignalEdit, apply_signal_edit};

    let mut states = StateSets::default();
    assert!(!apply_signal_edit(
        &sim,
        &scene,
        &map,
        &mut states,
        &[1, 2],
        SignalEdit::Add
    ));
    assert!(!apply_signal_edit(
        &sim,
        &scene,
        &map,
        &mut states,
        &[],
        SignalEdit::Add
    ));
    assert!(states.is_empty());
}
