//! ⭐ **F48d — o Offset Path é uma FERRAMENTA** (dono 02/11: «melhor transformar numa tools na barra
//! de tools»). O botão mora na coluna ①, as opções na fila ② e o `Apply`/`Remove` na ④; a secção do
//! Inspector saiu. Os ids e a porta (`event_expand`) são os mesmos da secção — cada gate pergunta
//! às BARRAS o que se vê e clica o id pela porta do painel, como o ponteiro sobre a barra faz.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{ToolBars, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::ToolRailEntry;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{EXPAND_KNOBS_AT_BIRTH, VectorPanel, bars, ids};
use ph2d_tool_vector::{DrawMode, VectorStyleSnapshot};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};

fn pintado(h: &mut MockPanelHost, id: NodeId) -> Option<Rect> {
    h.painted_rect::<VectorPanel>(&mut VectorPanelState, VIEWPORT, id)
}

fn host() -> MockPanelHost {
    ph2d_panel_vector::state::set_current_selection_count(1);
    ph2d_panel_vector::set_offset_knobs(EXPAND_KNOBS_AT_BIRTH);
    ph2d_panel_vector::set_offset_live_on(false);
    crate::host_aberto()
}

/// As barras com a ferramenta `mode` na mão.
fn barras(h: &MockPanelHost, mode: DrawMode) -> ToolBars {
    ph2d_panel_vector::set_current_vector_style(Some(VectorStyleSnapshot {
        mode,
        ..VectorStyleSnapshot::default()
    }));
    bars::tool_bars(h.store(), None, Vec::new())
}

fn fila(b: &ToolBars) -> Vec<NodeId> {
    b.row_lead
        .iter()
        .chain(&b.row_tail)
        .filter_map(ToolRailEntry::node_id)
        .collect()
}

fn acoes(b: &ToolBars) -> Vec<NodeId> {
    b.actions.iter().flatten().map(|x| x.id).collect()
}

/// O clique da barra: o id chega ao painel pela mesma porta (`apply_panel_event`).
fn clica_na_barra(h: &mut MockPanelHost, id: NodeId) {
    let out = h.apply_panel_event::<VectorPanel>(&mut VectorPanelState, WidgetEvent::Click(id));
    assert_eq!(
        out,
        EventOutcome::Consumed,
        "{id:?} na barra e o painel não o consome"
    );
}

fn chegou(h: &mut MockPanelHost, id: NodeId) -> bool {
    h.drained_actions()
        .into_iter()
        .any(|a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id))
}

/// ⭐ O botão **Offset Path** na coluna ① põe a ferramenta na mão, acende, e a dica ensina o gesto.
#[test]
fn o_botao_da_coluna_arma_a_ferramenta() {
    let mut h = host();
    let b = barras(&h, DrawMode::Select);
    assert!(
        b.column
            .iter()
            .flatten()
            .any(|x| x.id == ph2d_tool_vector::ids::VECTOR_MODE_OFFSET),
        "o Offset Path não tem botão na coluna ①"
    );
    let out = h.apply_panel_event::<VectorPanel>(
        &mut VectorPanelState,
        WidgetEvent::Click(ph2d_tool_vector::ids::VECTOR_MODE_OFFSET),
    );
    assert_eq!(out, EventOutcome::Consumed);
    assert!(
        chegou(&mut h, ph2d_tool_vector::ids::VECTOR_MODE_OFFSET),
        "o modo não chega à tool"
    );
    let b = barras(&h, DrawMode::Offset);
    assert!(b.hint.contains("grow"), "dica: {}", b.hint);
}

/// ⭐ A secção do Inspector SAIU: nenhum controlo do Offset Path se pinta no painel, e o `Outline
/// Stroke` (F50) mora só na ④. *Mutação: repor a `expand_section` no plano ⇒ vermelho.*
#[test]
fn a_seccao_do_inspector_saiu() {
    let mut h = host();
    ph2d_panel_vector::set_offset_live_on(true);
    for id in [
        ids::VECTOR_EXPAND_OFFSET,
        ph2d_tool_vector::ids::VECTOR_EXPAND_JOIN_MITER,
        ids::VECTOR_EXPAND_RESULT_BEHIND,
        ph2d_tool_vector::ids::VECTOR_EXPAND_OFFSET_PATH,
    ] {
        assert!(pintado(&mut h, id).is_none(), "{id:?} ainda no Inspector");
    }
    ph2d_panel_vector::set_offset_live_on(false);
    let id = ph2d_tool_vector::ids::VECTOR_EXPAND_OUTLINE_STROKE;
    assert!(
        pintado(&mut h, id).is_none(),
        "o Outline Stroke voltou ao painel"
    );
    let sel = Some(Rect::new(400.0, 300.0, 80.0, 60.0));
    let b = bars::tool_bars(h.store(), sel, Vec::new());
    assert!(
        b.actions.iter().flatten().any(|x| x.id == id),
        "o Outline Stroke saiu da ④"
    );
}

/// ⭐ Na fila ②, só com a ferramenta na mão: `Distance (px)` é um CAMPO, as três quinas, o
/// `Holes` sem o `Inner` que só movia furos, e o `Result` com as três escolhas.
#[test]
fn a_fila_mostra_as_opcoes_so_com_a_ferramenta() {
    let h = host();
    let t = |m| fila(&barras(&h, m));
    let todas = [
        ids::VECTOR_EXPAND_OFFSET,
        ph2d_tool_vector::ids::VECTOR_EXPAND_JOIN_MITER,
        ph2d_tool_vector::ids::VECTOR_EXPAND_JOIN_ROUND,
        ph2d_tool_vector::ids::VECTOR_EXPAND_JOIN_BEVEL,
        ids::VECTOR_EXPAND_MITER,
        // `Holes ▾` e `Result ▾`: uma lista cada (as escolhas são as LINHAS dela).
        ids::VECTOR_BAR_HOLES,
        ids::VECTOR_BAR_RESULT,
    ];
    let com = t(DrawMode::Offset);
    for id in todas {
        assert!(com.contains(&id), "{id:?} falta na fila ②");
        assert!(
            !t(DrawMode::Select).contains(&id),
            "{id:?} fora da ferramenta"
        );
    }
    assert!(!com.contains(&ph2d_tool_vector::ids::VECTOR_EXPAND_SIDE_INNER));
    assert!(
        barras(&h, DrawMode::Offset).row_lead.iter().any(
            |e| matches!(e, ToolRailEntry::Number { id, .. } if *id == ids::VECTOR_EXPAND_OFFSET)
        ),
        "a distância tem de ser um campo numérico"
    );
    assert_eq!(h.store().number_value(ids::VECTOR_EXPAND_OFFSET), Some(0.0));
}

/// ⭐ O `Result` nasce atrás e o clique grava-o; a escolha acende. *Mutação: o braço do `Result`
/// no `event_expand` gravar sempre o mesmo ⇒ vermelho.*
#[test]
fn o_result_grava_o_que_se_clica() {
    let mut h = host();
    let result = || ph2d_panel_vector::offset_knobs().result;
    // A face do `Result ▾` é a escolha de agora.
    let face = |h: &MockPanelHost| {
        barras(h, DrawMode::Offset)
            .row_lead
            .iter()
            .find_map(|e| match e {
                ToolRailEntry::Compound { id, face, .. } if *id == ids::VECTOR_BAR_RESULT => {
                    Some(face.clone())
                }
                _ => None,
            })
            .expect("o Result ▾ na fila")
    };
    assert_eq!(result(), 1, "nasce atrás");
    assert_eq!(face(&h), "Behind");
    clica_na_barra(&mut h, ids::VECTOR_EXPAND_RESULT_REPLACE);
    assert_eq!(result(), 0);
    assert_eq!(face(&h), "Replace");
    clica_na_barra(&mut h, ids::VECTOR_EXPAND_RESULT_FRONT);
    assert_eq!(result(), 2, "pela frente (dono 02/11)");
    clica_na_barra(&mut h, ids::VECTOR_EXPAND_RESULT_BEHIND);
    assert_eq!(result(), 1);
}

/// ⭐ O `Miter limit` só existe com a quina Miter, e a quina clicada acende. *Mutação: tirar o
/// `if k.join == 0` da fila ⇒ vermelho.*
#[test]
fn o_limite_so_aparece_no_miter() {
    let mut h = host();
    assert!(fila(&barras(&h, DrawMode::Offset)).contains(&ids::VECTOR_EXPAND_MITER));
    clica_na_barra(&mut h, ph2d_tool_vector::ids::VECTOR_EXPAND_JOIN_ROUND);
    assert_eq!(ph2d_panel_vector::expand_join(), 1);
    let b = barras(&h, DrawMode::Offset);
    assert!(!fila(&b).contains(&ids::VECTOR_EXPAND_MITER));
    assert!(b.row_lead.iter().any(|e| matches!(
        e,
        ToolRailEntry::Icon { id, active: true, .. }
            if *id == ph2d_tool_vector::ids::VECTOR_EXPAND_JOIN_ROUND
    )));
    ph2d_panel_vector::set_offset_knobs(EXPAND_KNOBS_AT_BIRTH);
}

/// ⭐ `Apply` e `Remove` na ④ só com um offset VIVO; o `Remove` zera a distância e não vai ao bus;
/// o `Apply` chega ao bus. *Mutação: tirar o `offset_live_on` do `offset_actions` ⇒ vermelho.*
#[test]
fn apply_e_remove_so_com_offset_vivo() {
    let mut h = host();
    let apply = ph2d_tool_vector::ids::VECTOR_EXPAND_OFFSET_PATH;
    assert!(!acoes(&barras(&h, DrawMode::Offset)).contains(&apply));
    ph2d_panel_vector::set_offset_live_on(true);
    let a = acoes(&barras(&h, DrawMode::Offset));
    assert!(a.contains(&apply) && a.contains(&ids::VECTOR_EXPAND_REMOVE));
    h.store_mut()
        .set_number_value(ids::VECTOR_EXPAND_OFFSET, 12.0);
    clica_na_barra(&mut h, ids::VECTOR_EXPAND_REMOVE);
    assert_eq!(h.store().number_value(ids::VECTOR_EXPAND_OFFSET), Some(0.0));
    assert!(!chegou(&mut h, ids::VECTOR_EXPAND_REMOVE));
    clica_na_barra(&mut h, apply);
    assert!(chegou(&mut h, apply));
    // O Offset é PRECISO: nem com duas formas a ② ganha a fila de baixo (alinhar, booleanas…).
    // *Mutação: tirar o Offset do `PRECISE` ⇒ vermelho.*
    ph2d_panel_vector::state::set_current_selection_count(2);
    assert!(
        barras(&h, DrawMode::Offset).actions_below.is_empty(),
        "o Offset ganhou a fila de baixo da ④"
    );
    ph2d_panel_vector::state::set_current_selection_count(1);
    ph2d_panel_vector::set_offset_live_on(false);
}

/// ⭐ Escolher uma linha do `Holes ▾` ou do `Result ▾` FECHA a lista dela. *Mutação: tirar
/// `[HOLES, RESULT]` do `close_list_on_pick` ⇒ vermelho.*
#[test]
fn escolher_na_lista_fecha_a_lista() {
    use ph2d_editor_core::interaction::InteractiveState;
    let mut h = host();
    let _ = barras(&h, DrawMode::Offset);
    for (lista, linha) in [
        (ids::VECTOR_BAR_RESULT, ids::VECTOR_EXPAND_RESULT_FRONT),
        (
            ids::VECTOR_BAR_HOLES,
            ph2d_tool_vector::ids::VECTOR_EXPAND_SIDE_BOTH,
        ),
    ] {
        let Some(InteractiveState::Dropdown { open, .. }) = h.store_mut().get_mut(lista) else {
            panic!("{lista:?} não é uma lista no store");
        };
        *open = true;
        clica_na_barra(&mut h, linha);
        assert!(
            !matches!(
                h.store().get(lista),
                Some(InteractiveState::Dropdown { open: true, .. })
            ),
            "{lista:?} ficou aberta depois da escolha"
        );
    }
    ph2d_panel_vector::set_offset_knobs(EXPAND_KNOBS_AT_BIRTH);
}

/// ⭐ O perfil de largura (os quatro W e o `Apply Power Stroke`) mora no «Stroke settings».
#[test]
fn o_perfil_de_largura_mora_no_stroke_settings() {
    let mut h = host();
    let power = ph2d_tool_vector::ids::VECTOR_EXPAND_POWER_STROKE;
    assert!(pintado(&mut h, ids::VECTOR_EXPAND_W_START).is_none());
    crate::clica(&mut h, ids::VECTOR_STROKE_SETTINGS);
    assert!(pintado(&mut h, ids::VECTOR_EXPAND_W_START).is_some());
    assert!(pintado(&mut h, power).is_some());
    crate::clica(&mut h, power);
    assert!(chegou(&mut h, power), "pintado e morto");
    ph2d_panel_vector::state::close_open_layer();
}
