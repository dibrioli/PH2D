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
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue};
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
        riders: 0,
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
        mount_offset: [0.0, 0.0],
        vis_in_editor: false,
        vis_at_runtime: false,
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

/// **(14) Trocar de linha na lista RE-SEMEIA os campos do editor.**
///
/// ⚠️ **Este gate nasceu de um defeito medido, não de uma hipótese.** Os campos semeavam-se só
/// quando a **ENTIDADE** mudava (`entity_changed`), então clicar noutra âncora da mesma sprite
/// mudava a ficha aberta e deixava o nome e as caixas a mostrar a anterior. A sonda de 2026-08-23
/// mediu nome `""` e `Bounds` **desmarcada** sobre `face_box`, que tem área.
///
/// ⚠️ E a cura tinha de ser uma **ARESTA**, não uma reescrita por quadro: reescrever sempre faria
/// a caixa que o artista acabou de clicar voltar atrás antes de o commit da shell chegar. Por isso
/// o teste abaixo verifica as duas metades — semeia ao trocar, **e** não pisa o que o artista
/// mexeu sem trocar de linha.
#[test]
fn switching_rows_reseeds_the_editor_without_stomping_a_fresh_click() {
    set_current_inspector_anchor(Some(info()));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    host.settle_section_folds();
    host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    // Linha 0 = `muzzle`, um Socket sem área.
    assert_eq!(host.store().text(ids::INSP_ANCHOR_NAME), Some("muzzle"));
    assert_eq!(
        host.store()
            .checkbox(ids::INSP_ANCHOR_BOUNDS_ON)
            .map(|(_, v)| v),
        Some(CheckboxValue::Unchecked)
    );

    // Linha 1 = `face_box`, uma Region — nome e caixas TÊM de acompanhar.
    host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::Click(ids::INSP_ANCHOR_ROW[1]),
    );
    host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    assert_eq!(
        host.store().text(ids::INSP_ANCHOR_NAME),
        Some("face_box"),
        "o nome ficou na ancora anterior"
    );
    assert_eq!(
        host.store()
            .checkbox(ids::INSP_ANCHOR_BOUNDS_ON)
            .map(|(_, v)| v),
        Some(CheckboxValue::Checked),
        "a caixa de area ficou na ancora anterior"
    );

    // ⚠️ A OUTRA metade: sem trocar de linha, o que o artista acabou de clicar **fica**. O commit
    // da shell demora um quadro, e uma reescrita por quadro desfá-lo-ia antes de ele chegar.
    host.set_checkbox_value(ids::INSP_ANCHOR_BOUNDS_ON, CheckboxValue::Unchecked);
    host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    assert_eq!(
        host.store()
            .checkbox(ids::INSP_ANCHOR_BOUNDS_ON)
            .map(|(_, v)| v),
        Some(CheckboxValue::Unchecked),
        "o sync pisou o clique antes de o commit chegar"
    );
}

/// **(15) O botão «Reset to Anchor» chega ao barramento — e só quando tem o que fazer.**
///
/// ⚠️ A segunda metade não é zelo: os 64+ ids da seção são registados no ARRANQUE, e um clique
/// sintético alcança um botão que não foi pintado. Sem a guarda no despacho, ele escreveria uma
/// pose zerada sobre um objeto que não monta em nada.
#[test]
fn the_reset_button_only_fires_when_the_object_is_off_anchor() {
    let off = InspectorAnchorInfo {
        mount: Some("hand_r".into()),
        mount_offset: [12.0, -4.0],
        ..info()
    };
    set_current_inspector_anchor(Some(off));
    let (mut host, mut state) = fresh(0);
    host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::Click(ids::INSP_MOUNT_SNAP));
    assert_eq!(edits(&mut host), vec![AnchorFieldEdit::SnapToAnchor]);

    // Já em cima da âncora: o botão não é pintado, e o clique sintético não escreve nada.
    let on = InspectorAnchorInfo {
        mount: Some("hand_r".into()),
        mount_offset: [0.0, 0.0],
        ..info()
    };
    set_current_inspector_anchor(Some(on));
    let (mut host, mut state) = fresh(0);
    host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::Click(ids::INSP_MOUNT_SNAP));
    assert!(
        edits(&mut host).is_empty(),
        "o botao disparou sobre um objeto que ja' esta' na ancora"
    );
}

/// **(16) A caixa de visibilidade VIVA chega ao barramento na variante dela.**
///
/// ⚠️ Trocá-la pela irmã compila. É o mesmo risco do par `Bounds`/`Center`.
///
/// ⛔⛔ **A «Show anchors at runtime» saiu desta lista em 2026-08-30** — ver
/// [`the_runtime_box_is_parked_until_a_game_runtime_exists`], que agora afirma o CONTRÁRIO sobre
/// ela e diz porquê.
#[test]
fn the_live_visibility_box_reaches_the_bus_as_itself() {
    for (on, expect) in [
        (true, AnchorFieldEdit::VisibilityInEditor(true)),
        (false, AnchorFieldEdit::VisibilityInEditor(false)),
    ] {
        set_current_inspector_anchor(Some(info()));
        let (mut host, mut state) = fresh(0);
        host.set_checkbox_value(
            ids::INSP_ANCHOR_VIS_EDITOR,
            if on {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            },
        );
        host.apply_panel_event::<InspectorPanel>(
            &mut state,
            WidgetEvent::Toggled(ids::INSP_ANCHOR_VIS_EDITOR),
        );
        assert_eq!(edits(&mut host), vec![expect]);
    }
}

/// ⛔⛔⛔ **(16-bis) A caixa «Show anchors at runtime» está PARADA, e o bloqueador tem nome.**
///
/// Ela gravava `AnchorVisibility::at_runtime` no `.ph2dproj` e **não tinha um único leitor** —
/// porque não existe modo de jogo (`shells/game` / Runtime R1, adiado por decisão do dono do
/// produto). Um controlo que promete e não entrega é pior que um ausente: o ausente não é
/// acreditado.
///
/// O gate mede as **três** metades da cura, e a quarta — a irmã VIVA — no fim:
///
/// 1. ela continua **pintada** (o artista vê que a capacidade existe e está parada);
/// 2. o registo é `Disabled` ⇒ `is_focusable` recusa-a, então o dedo não a alterna;
/// 3. um `Toggled` **sintético** (a porta que salta o `is_focusable`) não levanta edição nenhuma;
/// 4. ⚠️ a irmã «Always show anchors» continua a levantar a dela — parar as duas por simetria
///    apagaria uma feature que funciona.
///
/// **Mutação:** repor o `INSP_ANCHOR_VIS_RUNTIME` no `matches!` do `event_anchor.rs` ⇒ RED em (3);
/// registá-la `Normal` no `populate_anchor.rs` ⇒ RED em (2).
#[test]
fn the_runtime_box_is_parked_until_a_game_runtime_exists() {
    set_current_inspector_anchor(Some(info()));
    let (mut host, mut state) = fresh(0);
    let painted = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    assert!(
        painted
            .iter()
            .any(|(id, _)| *id == ids::INSP_ANCHOR_VIS_RUNTIME),
        "a caixa parada deixou de ser pintada — retirá-la da tela em silêncio é a outra metade do \
         mesmo defeito: ninguém fica a saber que a capacidade existe e está bloqueada"
    );
    assert_eq!(
        host.store()
            .checkbox(ids::INSP_ANCHOR_VIS_RUNTIME)
            .map(|(s, _)| s),
        Some(CheckboxState::Disabled),
        "a caixa parada esta' registada como alcancavel: o `is_focusable` deixa o dedo alterna-la \
         e o cinzento passa a ser decoracao"
    );

    set_current_inspector_anchor(Some(info()));
    let (mut host, mut state) = fresh(0);
    host.set_checkbox_value(ids::INSP_ANCHOR_VIS_RUNTIME, CheckboxValue::Checked);
    host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::Toggled(ids::INSP_ANCHOR_VIS_RUNTIME),
    );
    assert!(
        edits(&mut host).is_empty(),
        "um Toggled sintetico ainda escreve `at_runtime` — a recusa tem de viver no braco tambem, \
         nao so' no registo"
    );

    set_current_inspector_anchor(Some(info()));
    let (mut host, mut state) = fresh(0);
    host.set_checkbox_value(ids::INSP_ANCHOR_VIS_EDITOR, CheckboxValue::Checked);
    host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::Toggled(ids::INSP_ANCHOR_VIS_EDITOR),
    );
    assert_eq!(
        edits(&mut host),
        vec![AnchorFieldEdit::VisibilityInEditor(true)],
        "a caixa IRMA' — que tem consumidor vivo no `anchor_overlay` — foi parada junto"
    );
}

/// ⭐⭐⭐ **O BLOQUEADOR, escrito como um teste que falha no dia em que ele cai.**
///
/// A caixa acima está parada por UMA razão nomeada: `shells/game` não existe. No dia em que
/// alguém o criar, este gate fica vermelho e obriga quem o criou a voltar aqui e decidir — ligar
/// a caixa, ou reescrever a razão.
///
/// ⚠️ **A pergunta é feita ao DISCO, não a uma nota.** Uma nota a dizer «bloqueado por R1»
/// envelhece em silêncio; um `Path::exists` não.
#[test]
fn the_parked_box_comes_back_the_day_the_game_shell_exists() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root");
    let game = repo.join("shells").join("game");
    assert!(
        !game.exists(),
        "`{}` passou a existir. O bloqueador da caixa «Show anchors at runtime» era exactamente \
         *«nao ha' modo de jogo»* — reveja `populate_anchor.rs`, `event_anchor.rs` e o \
         `RUNTIME_BOX_LABEL` do `sections/anchor_mount_row.rs` antes de apagar esta linha.",
        game.display()
    );
    assert!(
        repo.join("shells").join("desktop").is_dir(),
        "a sonda nao esta' a olhar para a raiz da workspace — sem esta metade ela responderia \
         «nao existe» para sempre, sobre qualquer caminho"
    );
}

/// **(17) O botão de reset e as duas caixas pintam-se exatamente quando devem.**
#[test]
fn the_new_controls_appear_only_where_they_belong() {
    let painted = |info: InspectorAnchorInfo, id: NodeId| {
        set_current_inspector_anchor(Some(info));
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        host.settle_section_folds();
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        rects.iter().any(|(pid, _)| *pid == id)
    };
    let off = InspectorAnchorInfo {
        mount: Some("hand_r".into()),
        mount_offset: [12.0, -4.0],
        ..info()
    };
    assert!(painted(off.clone(), ids::INSP_MOUNT_SNAP), "deslocado");
    assert!(
        !painted(info(), ids::INSP_MOUNT_SNAP),
        "sem montagem nao ha' o que repor"
    );
    assert!(
        !painted(
            InspectorAnchorInfo {
                mount_offset: [0.0, 0.0],
                ..off
            },
            ids::INSP_MOUNT_SNAP
        ),
        "em cima da ancora o botao nao tem o que fazer"
    );
    // ⛔ As caixas são do DONO das âncoras: sem âncoras, não há o que manter visível.
    assert!(painted(info(), ids::INSP_ANCHOR_VIS_EDITOR));
    assert!(painted(info(), ids::INSP_ANCHOR_VIS_RUNTIME));
    assert!(!painted(
        InspectorAnchorInfo {
            rows: Vec::new(),
            ..info()
        },
        ids::INSP_ANCHOR_VIS_EDITOR
    ));
}
