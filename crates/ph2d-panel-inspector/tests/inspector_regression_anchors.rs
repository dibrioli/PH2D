//! **A costura da §12 Sockets / Named Anchors** ([ADR-0072]) — escrita junto com a seção.
//!
//! Quarto ficheiro da família [`inspector_regression`]. As leis são as das ondas anteriores, mais
//! uma que só esta seção tem: **clicar numa linha da lista NÃO é uma edição da cena**, e por isso
//! não pode chegar ao barramento.
//!
//! [ADR-0072]: ../../../docs/architecture/decisions/0072-named-anchor-unification.md

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::{AnchorFieldEdit, InspectorAnchorInfo, InspectorAnchorRow};
use ph2d_editor_core::widget::CheckboxValue;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_anchor};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0xA0C0_1234;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 8000.0,
};

fn row(name: &str, bounds: Option<[f32; 4]>, center: Option<[f32; 4]>) -> InspectorAnchorRow {
    InspectorAnchorRow {
        name: name.into(),
        pos: [1.0, 2.0],
        rot_deg: 0.0,
        bounds,
        center,
    }
}

/// Três âncoras, e a **segunda é uma `Region`** — o estado em que todos os campos existem.
fn info() -> InspectorAnchorInfo {
    InspectorAnchorInfo {
        entity_bits: ENTITY,
        rows: vec![
            row("muzzle", None, None),
            row("face_box", Some([8.0, 4.0, 24.0, 24.0]), Some([2.0; 4])),
            row("foot", Some([0.0, 0.0, 8.0, 8.0]), None),
        ],
        present: true,
        selected_count: 1,
        mixed: false,
        // O pai oferece duas âncoras e este objeto não monta em nenhuma — o estado de partida do
        // seletor. Os gates da montagem, abaixo, variam estes dois campos.
        parent_anchors: vec!["hand_r".into(), "hand_l".into()],
        mount: None,
    }
}

fn fresh(selected: usize) -> (MockPanelHost, InspectorState) {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState {
        anchor_selected: selected,
        ..InspectorState::default()
    };
    host.settle_section_folds();
    host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let _ = host.drained_actions();
    (host, state)
}

fn edits(host: &mut MockPanelHost) -> Vec<AnchorFieldEdit> {
    host.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorAnchorEdit { entity_bits, edit } => {
                assert_eq!(entity_bits, ENTITY, "escreveu noutra sprite");
                Some(edit)
            }
            _ => None,
        })
        .collect()
}

/// **(1) Clicar numa linha muda a ficha aberta — e NÃO vai ao barramento.**
///
/// ⚠️ Qual âncora se está a editar é um facto da UI. Publicá-lo como ação obrigaria a shell a
/// saber dele, e faria toda troca de linha custar um quadro.
#[test]
fn selecting_a_row_changes_the_open_card_without_touching_the_bus() {
    set_current_inspector_anchor(Some(info()));
    let (mut host, mut state) = fresh(0);
    let outcome = host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::Click(ids::INSP_ANCHOR_ROW[2]),
    );
    assert!(
        matches!(
            outcome,
            ph2d_editor_core::panel::EventOutcome::Consumed
                | ph2d_editor_core::panel::EventOutcome::Observed
        ),
        "o clique na linha nao foi consumido"
    );
    assert_eq!(state.anchor_selected, 2, "a ficha aberta nao mudou");
    assert!(
        edits(&mut host).is_empty(),
        "escolher que ficha ver nao e' uma edicao da cena"
    );
}

/// **(2) Uma linha ALÉM do fim da lista não muda nada.**
///
/// As 64 linhas estão sempre registadas (o registo acontece no arranque); só as primeiras N são
/// pintadas. Um clique sintético nas outras não pode abrir uma ficha que não existe.
#[test]
fn a_row_past_the_end_of_the_list_selects_nothing() {
    set_current_inspector_anchor(Some(info()));
    let (mut host, mut state) = fresh(1);
    host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::Click(ids::INSP_ANCHOR_ROW[40]),
    );
    assert_eq!(state.anchor_selected, 1, "abriu uma ficha inexistente");
    assert!(edits(&mut host).is_empty());
}

/// **(3) Cada campo edita a âncora ABERTA, e só o seu eixo.**
#[test]
fn every_field_edits_the_open_anchor_and_only_its_own_axis() {
    // A ficha 1 (`face_box`) é a `Region` — só nela os oito campos existem.
    let cases: Vec<(&str, NodeId, WidgetEvent, AnchorFieldEdit)> = vec![
        (
            "Pos Y",
            ids::INSP_ANCHOR_POS[1],
            WidgetEvent::ValueChanged(ids::INSP_ANCHOR_POS[1]),
            AnchorFieldEdit::Pos(1, 1, 33.0),
        ),
        (
            "Rot",
            ids::INSP_ANCHOR_ROT,
            WidgetEvent::ValueChanged(ids::INSP_ANCHOR_ROT),
            AnchorFieldEdit::Rot(1, 33.0),
        ),
        (
            "Bounds W (indice 2)",
            ids::INSP_ANCHOR_BOUNDS[2],
            WidgetEvent::ValueChanged(ids::INSP_ANCHOR_BOUNDS[2]),
            AnchorFieldEdit::Bounds(1, 2, 33.0),
        ),
        (
            "Center H (indice 3)",
            ids::INSP_ANCHOR_CENTER[3],
            WidgetEvent::ValueChanged(ids::INSP_ANCHOR_CENTER[3]),
            AnchorFieldEdit::Center(1, 3, 33.0),
        ),
    ];
    for (what, id, ev, expect) in cases {
        set_current_inspector_anchor(Some(info()));
        let (mut host, mut state) = fresh(1);
        host.set_number_value(id, 33.0);
        host.apply_panel_event::<InspectorPanel>(&mut state, ev);
        assert_eq!(edits(&mut host), vec![expect], "'{what}' despachou errado");
    }
}

/// **(4) As duas caixas e os dois botões.**
#[test]
fn the_toggles_and_the_buttons_reach_the_bus() {
    for (what, ev, prep_on, expect) in [
        (
            "Bounds on",
            WidgetEvent::Toggled(ids::INSP_ANCHOR_BOUNDS_ON),
            Some((ids::INSP_ANCHOR_BOUNDS_ON, true)),
            AnchorFieldEdit::BoundsOn(1, true),
        ),
        (
            "Center off",
            WidgetEvent::Toggled(ids::INSP_ANCHOR_CENTER_ON),
            Some((ids::INSP_ANCHOR_CENTER_ON, false)),
            AnchorFieldEdit::CenterOn(1, false),
        ),
        (
            "Add",
            WidgetEvent::Click(ids::INSP_ANCHOR_ADD),
            None,
            AnchorFieldEdit::Add,
        ),
        (
            "Remove",
            WidgetEvent::Click(ids::INSP_ANCHOR_REMOVE),
            None,
            AnchorFieldEdit::Remove(1),
        ),
    ] {
        set_current_inspector_anchor(Some(info()));
        let (mut host, mut state) = fresh(1);
        if let Some((id, on)) = prep_on {
            host.set_checkbox_value(
                id,
                if on {
                    CheckboxValue::Checked
                } else {
                    CheckboxValue::Unchecked
                },
            );
        }
        host.apply_panel_event::<InspectorPanel>(&mut state, ev);
        assert_eq!(edits(&mut host), vec![expect], "'{what}' despachou errado");
    }
}

/// **(5) Escrever o nome leva o texto ATUAL da caixa.**
#[test]
fn typing_a_name_carries_the_current_text() {
    set_current_inspector_anchor(Some(info()));
    let (mut host, mut state) = fresh(2);
    host.set_text(ids::INSP_ANCHOR_NAME, "left_foot");
    host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::TextChanged(ids::INSP_ANCHOR_NAME),
    );
    assert_eq!(
        edits(&mut host),
        vec![AnchorFieldEdit::Rename(2, "left_foot".into())]
    );
}

/// **(6) Sem snapshot publicado, nada age.**
#[test]
fn no_anchor_control_acts_without_its_snapshot() {
    for (what, ev) in [
        ("Add", WidgetEvent::Click(ids::INSP_ANCHOR_ADD)),
        ("Remove", WidgetEvent::Click(ids::INSP_ANCHOR_REMOVE)),
        ("row", WidgetEvent::Click(ids::INSP_ANCHOR_ROW[0])),
        ("Pos X", WidgetEvent::ValueChanged(ids::INSP_ANCHOR_POS[0])),
        ("name", WidgetEvent::TextChanged(ids::INSP_ANCHOR_NAME)),
    ] {
        set_current_inspector_anchor(None);
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        host.settle_section_folds();
        let _ = host.drained_actions();
        host.apply_panel_event::<InspectorPanel>(&mut state, ev);
        assert!(
            edits(&mut host).is_empty(),
            "'{what}' despachou sem snapshot"
        );
    }
}

/// **(7) Com o componente ANEXADO e VAZIO, só o «+ Add» age.**
///
/// ⚠️ Os campos do editor não existem sobre uma lista vazia — não há âncora para editar, e um
/// campo que despachasse escreveria no índice 0 de uma lista sem índice 0.
#[test]
fn on_an_empty_list_only_add_acts() {
    let empty = InspectorAnchorInfo {
        rows: Vec::new(),
        ..info()
    };
    set_current_inspector_anchor(Some(empty.clone()));
    let (mut host, mut state) = fresh(0);
    host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::ValueChanged(ids::INSP_ANCHOR_POS[0]),
    );
    assert!(
        edits(&mut host).is_empty(),
        "um campo agiu sobre a lista vazia"
    );

    set_current_inspector_anchor(Some(empty));
    let (mut host, mut state) = fresh(0);
    host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::Click(ids::INSP_ANCHOR_ADD));
    assert_eq!(edits(&mut host), vec![AnchorFieldEdit::Add]);
}

/// **(8) O editor só pinta os campos que a FORMA da âncora tem.**
///
/// Um Socket não mostra os oito números de área/miolo: eles não vão a lado nenhum.
#[test]
fn the_editor_paints_only_the_fields_the_shape_has() {
    set_current_inspector_anchor(Some(info()));
    // Ficha 0 = `muzzle`, um Socket.
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    host.settle_section_folds();
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let painted = |id: NodeId| rects.iter().any(|(pid, _)| *pid == id);
    assert!(painted(ids::INSP_ANCHOR_POS[0]), "um Socket tem posicao");
    assert!(
        painted(ids::INSP_ANCHOR_BOUNDS_ON),
        "e a caixa que lhe da' area"
    );
    for id in ids::INSP_ANCHOR_BOUNDS {
        assert!(!painted(id), "um Socket nao tem campos de area");
    }
    for id in ids::INSP_ANCHOR_CENTER {
        assert!(!painted(id), "um Socket nao tem campos de miolo");
    }
    // Ficha 1 = `face_box`, uma Region: tem tudo.
    let mut state = InspectorState {
        anchor_selected: 1,
        ..InspectorState::default()
    };
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let painted = |id: NodeId| rects.iter().any(|(pid, _)| *pid == id);
    for id in ids::INSP_ANCHOR_BOUNDS
        .iter()
        .chain(ids::INSP_ANCHOR_CENTER.iter())
    {
        assert!(painted(*id), "uma Region tem os oito campos");
    }
}

/// **(9) Apagar a última âncora não deixa a ficha aberta a apontar para o vazio.**
///
/// ⚠️ O índice é saturado na pintura. Sem isso, o editor ficaria aberto sobre uma linha que já
/// não existe — e o próximo campo editado escreveria na âncora errada.
#[test]
fn the_open_card_is_clamped_when_the_list_shrinks() {
    set_current_inspector_anchor(Some(info()));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState {
        anchor_selected: 2,
        ..InspectorState::default()
    };
    host.settle_section_folds();
    host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    assert_eq!(state.anchor_selected, 2);
    // A lista encolhe para uma.
    let one = InspectorAnchorInfo {
        rows: vec![row("only", None, None)],
        ..info()
    };
    set_current_inspector_anchor(Some(one));
    host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    assert_eq!(state.anchor_selected, 0, "o indice nao foi saturado");
}

// ── «Rides Parent Anchor» — o consumidor de uma âncora (ADR-0072 §2.6) ────────────────────────

/// **(10) Escolher uma âncora do pai publica o NOME, nunca o índice.**
///
/// ⚠️ O índice é do widget e vale só neste quadro; o vínculo tem de sobreviver a reordenar a
/// lista do pai e a reabrir o projeto. Este gate falha se alguém trocar a edição por um `u8`.
#[test]
fn picking_a_parent_anchor_publishes_the_name() {
    set_current_inspector_anchor(Some(info()));
    let (mut host, mut state) = fresh(0);
    host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::Click(ids::INSP_MOUNT_OPT[1]),
    );
    assert_eq!(
        edits(&mut host),
        vec![AnchorFieldEdit::Mount(Some("hand_l".into()))],
        "a opcao 1 do pai e' `hand_l`"
    );
}

/// **(11) A opção «—» desfaz o vínculo — e funciona mesmo sobre um pai SEM âncoras.**
///
/// ⚠️ É este o caso que impede um estado preso: renomeada a âncora, a lista do pai pode nem
/// conter o nome montado, e sem esta saída o artista não teria gesto nenhum para o largar.
#[test]
fn the_dash_option_always_clears_the_mount_even_with_no_parent_anchors() {
    let trapped = InspectorAnchorInfo {
        parent_anchors: Vec::new(),
        mount: Some("gone".into()),
        ..info()
    };
    set_current_inspector_anchor(Some(trapped));
    let (mut host, mut state) = fresh(0);
    host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::Click(ids::INSP_MOUNT_NONE_OPT),
    );
    assert_eq!(edits(&mut host), vec![AnchorFieldEdit::Mount(None)]);
}

/// **(12) Uma opção ALÉM do que o pai oferece não escolhe nada.**
///
/// Os 64 ids estão sempre registados; só os que a lista do pai alcança são pintados. Um clique
/// sintético nos outros escolheria uma âncora inexistente — e o vínculo nasceria pendurado.
#[test]
fn a_mount_option_past_the_parents_list_picks_nothing() {
    set_current_inspector_anchor(Some(info()));
    let (mut host, mut state) = fresh(0);
    host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::Click(ids::INSP_MOUNT_OPT[40]),
    );
    assert!(
        edits(&mut host).is_empty(),
        "escolheu uma ancora que o pai nao tem"
    );
}

/// **(13) O seletor só se pinta quando há o que escolher — e um vínculo pendurado aparece sempre.**
///
/// ⛔ Um controlo com uma opção só é a afordância a mentir que o botão `Simple` do 9-slice era
/// (Enio, 2026-08-22). Mas escondê-lo sobre um vínculo pendurado prenderia o estado.
#[test]
fn the_mount_picker_appears_exactly_when_it_is_useful() {
    let painted_pick = |info: InspectorAnchorInfo| {
        set_current_inspector_anchor(Some(info));
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        host.settle_section_folds();
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        rects.iter().any(|(id, _)| *id == ids::INSP_MOUNT_PICK)
    };

    assert!(
        painted_pick(info()),
        "o pai tem ancoras — o seletor tem de existir"
    );
    assert!(
        !painted_pick(InspectorAnchorInfo {
            parent_anchors: Vec::new(),
            mount: None,
            ..info()
        }),
        "sem pai e sem vinculo, o seletor nao tem o que oferecer"
    );
    assert!(
        painted_pick(InspectorAnchorInfo {
            parent_anchors: Vec::new(),
            mount: Some("gone".into()),
            ..info()
        }),
        "um vinculo pendurado sem a linha e' um estado preso"
    );
}
