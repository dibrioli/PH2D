//! ⭐⭐ **O painel Flip em cartões: nenhum controlo morreu.** Cada id é provado pelo gesto REAL, nos
//! rects pintados, modo a modo, e a costura termina na própria `FlipTool` quando o modo é dela.

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{dicas, dropdown, ferramenta};
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_editor_core::widget::key_mark::{KeyMark, key_mark_id, key_mark_target, set_key_marks};
use ph2d_painter_effects::MAX_BLEND_MODES;
use ph2d_panel_flip::ids::{self, FlipLayerWidget as W};
use ph2d_panel_flip::state::FlipPanelState;
use ph2d_panel_flip::{FlipLayerRow, FlipLayersSnapshot, FlipPanel, tool_button};
use ph2d_tool_flip::ids as t;
use ph2d_tool_flip::{EraseMode, FlipMode, FlipStyleSnapshot, FlipTool, StrokeTip};
use ph2d_ui_testkit::mesa::{Mesa, ultimo_valor};
use std::collections::{BTreeMap, BTreeSet};

const FUNDO: u64 = 3;
const CIMA: u64 = 4;
const PANEL: NodeId = ph2d_editor_core::ids::FLIP_PANEL;
const MODOS: [FlipMode; 7] = [
    FlipMode::Draw,
    FlipMode::Erase,
    FlipMode::Fill,
    FlipMode::Colorize,
    FlipMode::Trace,
    FlipMode::Edit,
    FlipMode::Reshape,
];

fn linha(id: u64, nome: &str) -> FlipLayerRow {
    FlipLayerRow {
        id,
        name: nome.to_owned(),
        blend: 0,
        opacity: 1.0,
        depth: 1.0,
        visible: true,
        locked: false,
    }
}

fn camadas() -> FlipLayersSnapshot {
    FlipLayersSnapshot {
        rows: vec![linha(FUNDO, "Back"), linha(CIMA, "Top")],
        active: Some(CIMA),
    }
}

fn mesa(s: FlipStyleSnapshot) -> Mesa<FlipPanel> {
    ferramenta::force_cards_on_this_thread(Some(true));
    ph2d_panel_flip::set_current_flip_style(Some(s));
    ph2d_panel_flip::set_current_flip_layers(camadas());
    set_key_marks(BTreeMap::new());
    let mut m = Mesa::<FlipPanel>::nova(FlipPanelState::default());
    m.host.drained_actions();
    m
}

fn em(mode: FlipMode) -> FlipStyleSnapshot {
    FlipStyleSnapshot {
        mode,
        ..FlipStyleSnapshot::default()
    }
}

/// O chip e a barra que ele avisa, os mesmos pares do `populate`.
const CAMPOS: &[(NodeId, NodeId)] = &[
    (ids::FLIP_SIZE_NUM, t::FLIP_SIZE),
    (ids::FLIP_HARDNESS_NUM, t::FLIP_HARDNESS),
    (ids::FLIP_OPACITY_NUM, t::FLIP_OPACITY),
    (ids::FLIP_SMOOTHING_NUM, t::FLIP_SMOOTHING),
    (ids::FLIP_PRESSURE_MIN_NUM, t::FLIP_PRESSURE_MIN),
    (ids::FLIP_PRESSURE_RESPONSE_NUM, t::FLIP_PRESSURE_RESPONSE),
    (ids::FLIP_DOT_SPACING_NUM, t::FLIP_DOT_SPACING),
    (ids::FLIP_ERASE_SIZE_NUM, t::FLIP_ERASE_SIZE),
    (ids::FLIP_ERASE_STRENGTH_NUM, t::FLIP_ERASE_STRENGTH),
    (ids::FLIP_GAP_NUM, t::FLIP_GAP),
    (ids::FLIP_TRAP_NUM, t::FLIP_TRAP),
    (ids::FLIP_GROW_NUM, t::FLIP_GROW),
    (ids::FLIP_PRECISION_NUM, t::FLIP_PRECISION),
    (ids::FLIP_COLORIZE_BLEED_NUM, t::FLIP_COLORIZE_BLEED),
];

/// Os botões que seguem à ferramenta ou ao drain da shell como `Click`.
fn botoes() -> Vec<NodeId> {
    let mut v: Vec<NodeId> = MODOS.iter().map(|m| tool_button(*m).0).collect();
    v.extend([
        t::FLIP_SHAPE_LINE,
        t::FLIP_SHAPE_FILLED,
        t::FLIP_TIP_LINE,
        t::FLIP_TIP_DOTS,
        t::FLIP_TIP_SQUARES,
        t::FLIP_CAP_ROUND,
        t::FLIP_CAP_FLAT,
        t::FLIP_CAP_SQUARE,
        t::FLIP_SELF_OVERLAP,
        t::FLIP_AIRBRUSH,
        t::FLIP_ERASE_SOFT,
        t::FLIP_ERASE_HARD,
        t::FLIP_ERASE_STROKE,
        t::FLIP_LINK_SIZE,
        t::FLIP_LINK_STRENGTH,
        t::FLIP_FILL_PAINT,
        t::FLIP_FILL_BEHIND,
        t::FLIP_FILL_UNPAINT,
        t::FLIP_EDIT_DOM_STROKE,
        t::FLIP_EDIT_DOM_POINT,
        t::FLIP_EDIT_DOM_SEGMENT,
        ids::FLIP_EDIT_SELECT_ALL,
        ids::FLIP_EDIT_DESELECT,
        ids::FLIP_EDIT_DELETE,
        ids::FLIP_COLORIZE_APPLY,
        ids::FLIP_COLORIZE_CLEAR,
        ids::FLIP_TRACE_RESET,
        t::FLIP_LAYER_ADD,
        ids::FLIP_LAYER_DUPLICATE,
        ids::FLIP_LAYER_DELETE,
    ]);
    v.extend(t::FLIP_RESHAPE_KIND_IDS);
    v
}

/// Os estados que juntos mostram todo o painel: cada modo, mais as variantes dentro de um modo.
fn estados() -> Vec<FlipStyleSnapshot> {
    let mut v: Vec<FlipStyleSnapshot> = MODOS.iter().map(|m| em(*m)).collect();
    v.push(FlipStyleSnapshot {
        tip: StrokeTip::Dots,
        draw_filled: true,
        ..em(FlipMode::Draw)
    });
    v.push(FlipStyleSnapshot {
        erase: EraseMode::Soft,
        link_size: false,
        link_strength: false,
        ..em(FlipMode::Erase)
    });
    v.push(FlipStyleSnapshot {
        erase: EraseMode::Soft,
        link_size: true,
        link_strength: true,
        ..em(FlipMode::Erase)
    });
    v
}

/// O modo em que o chip aparece, ou nada: a prova segue-o até lá.
fn onde_aparece(chip: NodeId) -> FlipStyleSnapshot {
    estados()
        .into_iter()
        .find(|s| mesa(*s).rect(chip).is_some())
        .unwrap_or_else(|| panic!("{chip:?} não aparece em estado nenhum"))
}

/// ⭐ O inventário de antes: tudo o que se regista pinta-se em ALGUM estado, e as barras nunca.
#[test]
fn every_id_of_the_old_panel_is_painted_in_some_state_and_the_bars_never() {
    let mut pintados = BTreeSet::new();
    for s in estados() {
        pintados.extend(mesa(s).pintados());
    }
    let mortos: Vec<NodeId> = botoes()
        .into_iter()
        .chain(CAMPOS.iter().map(|c| c.0))
        .chain([
            ids::FLIP_STROKE_SWATCH,
            ids::FLIP_FILL_SWATCH,
            ids::FLIP_COLORIZE_SWATCH,
            ids::FLIP_CLOSE,
        ])
        .filter(|id| !pintados.contains(id))
        .collect();
    assert!(mortos.is_empty(), "nunca pintados: {mortos:?}");
    for (_, barra) in CAMPOS {
        assert!(
            !pintados.contains(barra),
            "a barra {barra:?} ainda se pinta"
        );
    }
}

/// ⭐⭐ Cada número pelo SCRUB real, e o Esc devolve-o à ferramenta, não só à caixa. O valor chega à
/// própria `FlipTool`. *Mutação: tirar o chip de `numeros` ⇒ a pega não arrasta.*
#[test]
fn scrubbing_each_field_reaches_the_tool_and_escape_gives_it_back() {
    for &(chip, barra) in CAMPOS {
        let mut me = mesa(onde_aparece(chip));
        let antes = me.host.store().number_value(chip).expect("semeado");
        // Para dentro da faixa: o que nasce no tecto só desce.
        let valor = me.host.store().slider(barra).map_or(0.5, |(_, v)| v);
        let dx = if valor > 0.5 { -60.0 } else { 60.0 };
        let (arrastado, voltou) = me.prova_o_scrub(chip, barra, dx);
        assert!((arrastado - voltou).abs() > 1e-4, "{chip:?} não se moveu");
        let depois = me.host.store().number_value(chip).expect("semeado");
        assert!(
            (depois - antes).abs() < 1e-3,
            "{chip:?}: o Esc deixou {depois}, era {antes}"
        );
    }
}

/// O número escrito chega à ferramenta pelo caminho de sempre.
#[test]
fn typing_a_size_reaches_the_tool() {
    let mut me = mesa(em(FlipMode::Draw));
    let ev = me.host.type_into_number(ids::FLIP_SIZE_NUM, "17");
    let mut tool = FlipTool::default();
    for a in me.aplica(ev) {
        if let EditorAction::ToolPanelEvent(pe) = a {
            tool.handle_panel_event(pe);
        }
    }
    assert!((tool.width_px() - 17.0).abs() < 0.5, "{}", tool.width_px());
}

/// O campo mostra o que a ferramenta tem, mesmo que outro caminho (o atalho `[`) o tenha mudado.
#[test]
fn the_card_shows_what_the_tool_holds() {
    let me = mesa(FlipStyleSnapshot {
        width_px: 33.0,
        opacity: 0.25,
        ..em(FlipMode::Draw)
    });
    let store = me.host.store();
    assert_eq!(store.number_value(ids::FLIP_SIZE_NUM), Some(33.0));
    assert_eq!(store.number_value(ids::FLIP_OPACITY_NUM), Some(25.0));
}

/// Todo botão responde ao clique real no estado em que aparece. *Mutação: tirar `t::FLIP_AIRBRUSH`
/// de `caixas_aos_pares` ⇒ «não pintado».*
#[test]
fn every_button_answers_a_real_click_in_the_state_it_shows() {
    for id in botoes() {
        let s = onde_aparece(id);
        mesa(s).prova_o_clique(id);
    }
}

/// O clique no modo de verdade muda a `FlipTool`: a costura termina nela, não no bus.
#[test]
fn clicking_a_tool_changes_the_flip_tool() {
    let mut me = mesa(em(FlipMode::Draw));
    let mut tool = FlipTool::default();
    for a in me.clica(tool_button(FlipMode::Fill).0) {
        if let EditorAction::ToolPanelEvent(pe) = a {
            tool.handle_panel_event(pe);
        }
    }
    assert_eq!(tool.ui_snapshot().mode, FlipMode::Fill);
}

/// O Fill só no modo dele; a Strength só no Soft; o Spacing só com contas (a doutrina modal).
#[test]
fn each_mode_shows_only_its_own_cards() {
    let pinta = |s: FlipStyleSnapshot, id: NodeId| mesa(s).rect(id).is_some();
    assert!(pinta(em(FlipMode::Fill), ids::FLIP_SEC_FILL));
    assert!(!pinta(em(FlipMode::Draw), ids::FLIP_SEC_FILL));
    assert!(pinta(em(FlipMode::Colorize), ids::FLIP_SEC_COLORIZE));
    assert!(!pinta(em(FlipMode::Fill), ids::FLIP_SEC_BRUSH));
    assert!(pinta(em(FlipMode::Fill), ids::FLIP_SEC_LAYERS));
    let macia = FlipStyleSnapshot {
        erase: EraseMode::Soft,
        ..em(FlipMode::Erase)
    };
    assert!(pinta(macia, t::FLIP_LINK_STRENGTH));
    let dura = FlipStyleSnapshot {
        erase: EraseMode::Hard,
        ..em(FlipMode::Erase)
    };
    assert!(!pinta(dura, t::FLIP_LINK_STRENGTH));
    assert!(pinta(dura, t::FLIP_LINK_SIZE));
    assert!(!pinta(em(FlipMode::Draw), ids::FLIP_DOT_SPACING_NUM));
    assert!(!pinta(em(FlipMode::Reshape), ids::FLIP_SMOOTHING_NUM));
    assert!(!pinta(em(FlipMode::Edit), ids::FLIP_SMOOTHING_NUM));
}

/// As amostras abrem o seletor: a pressão nelas é a do despacho genérico.
#[test]
fn the_swatches_are_picker_swatches_in_their_modes() {
    for (s, id) in [
        (em(FlipMode::Draw), ids::FLIP_STROKE_SWATCH),
        (em(FlipMode::Edit), ids::FLIP_FILL_SWATCH),
        (em(FlipMode::Fill), ids::FLIP_FILL_SWATCH),
        (em(FlipMode::Colorize), ids::FLIP_COLORIZE_SWATCH),
    ] {
        let me = mesa(s);
        assert!(me.rect(id).is_some(), "{id:?} não pintada");
        assert!(me.host.store().is_picker_swatch(id));
    }
}

fn da_camada(k: W) -> NodeId {
    ids::flip_layer_widget_id(CIMA, k)
}

/// ⭐ A linha da camada: olho, cadeado, seleção e a seta que pode andar seguem à shell; a seta que
/// não pode andar pinta-se e não regista alvo.
#[test]
fn the_layer_rows_forward_and_a_blocked_arrow_is_not_a_target() {
    let mut me = mesa(em(FlipMode::Draw));
    for k in [W::Visibility, W::Lock, W::Row, W::MoveDown] {
        me.prova_o_clique(da_camada(k));
    }
    assert!(
        me.rect(da_camada(W::MoveUp)).is_none(),
        "a de cima não sobe"
    );
    assert!(
        me.rect(ids::flip_layer_widget_id(FUNDO, W::MoveDown))
            .is_none(),
        "a do fundo não desce"
    );
    me.prova_o_clique(ids::flip_layer_widget_id(FUNDO, W::MoveUp));
}

/// A opacidade e a profundidade por scrub: a mão arrasta o chip, a barra da camada avisa a shell.
#[test]
fn scrubbing_layer_opacity_and_depth_reaches_the_shell() {
    for (chip, barra) in [(W::OpacityNum, W::Opacity), (W::DepthNum, W::Depth)] {
        let mut me = mesa(em(FlipMode::Draw));
        let ev = me.arrasta_a_pega(da_camada(chip), -60.0);
        let a = me.aplica(ev);
        let v = ultimo_valor(&a, da_camada(barra)).expect("o scrub avisou");
        assert!(v < 1.0, "{chip:?}: {v}");
    }
}

/// ⭐ A mistura abre pelo clique, tem os 22 modos, e a opção escolhida segue à shell; a camada do fundo
/// não a tem. *Mutação: tirar o `dropdown::declare` ⇒ a caixa não abre.*
#[test]
fn the_blend_box_opens_and_its_option_reaches_the_shell() {
    let mut me = mesa(em(FlipMode::Draw));
    let caixa = da_camada(W::Blend);
    assert!(
        me.rect(ids::flip_layer_widget_id(FUNDO, W::Blend))
            .is_none()
    );
    assert!(dropdown::on_click(caixa), "a caixa não foi declarada");
    me.assenta();
    for modo in 0..MAX_BLEND_MODES {
        assert!(
            me.rect(ids::flip_layer_blend_option_id(CIMA, modo))
                .is_some()
        );
    }
    let opcao = ids::flip_layer_blend_option_id(CIMA, 3);
    let a = me.clica(opcao);
    assert!(
        a.iter().any(|x| matches!(x, EditorAction::ToolPanelEvent(PanelEvent::SelectOption(id, v)) if *id == caixa && v == "3")),
        "{a:?}"
    );
    assert!(!dropdown::on_click(opcao));
}

/// ⭐ O duplo clique abre o campo por cima do nome; o Enter pede o nome novo à shell.
#[test]
fn double_clicking_a_layer_name_renames_it_on_top_of_the_row() {
    let mut me = mesa(em(FlipMode::Draw));
    let linha = da_camada(W::Row);
    let aberto = me.aplica(vec![WidgetEvent::DoubleClick(linha)]);
    assert!(aberto.is_empty(), "{aberto:?}");
    me.assenta();
    assert!(
        me.rect(ids::FLIP_LAYER_RENAME_INPUT).is_some(),
        "o campo não abriu"
    );
    assert_eq!(
        me.host.store().focus_id(),
        Some(ids::FLIP_LAYER_RENAME_INPUT)
    );
    let ev = me
        .host
        .type_keys_then_enter(ids::FLIP_LAYER_RENAME_INPUT, "X");
    let a = me.aplica(ev);
    assert!(
        a.iter().any(|x| matches!(x, EditorAction::ToolPanelEvent(PanelEvent::SelectOption(id, n)) if *id == linha && n == "TopX")),
        "{a:?}"
    );
    me.assenta();
    assert!(me.rect(ids::FLIP_LAYER_RENAME_INPUT).is_none());
}

/// ⭐ Cada canal da camada tem o losango dele, no cartão; o cadeado não é canal.
#[test]
fn each_layer_channel_keeps_its_mark_in_the_card() {
    ferramenta::force_cards_on_this_thread(Some(true));
    ph2d_panel_flip::set_current_flip_style(Some(em(FlipMode::Draw)));
    ph2d_panel_flip::set_current_flip_layers(camadas());
    set_key_marks(BTreeMap::from([
        (da_camada(W::Visibility), KeyMark::Chave),
        (da_camada(W::Blend), KeyMark::Animado),
        (da_camada(W::Opacity), KeyMark::Mudado),
        (da_camada(W::OpacityNum), KeyMark::Mudado),
        (da_camada(W::Depth), KeyMark::Livre),
        (da_camada(W::DepthNum), KeyMark::Livre),
    ]));
    let me = Mesa::<FlipPanel>::nova(FlipPanelState::default());
    let alvo = |c| me.rect(key_mark_id(c)).is_some();
    assert!(alvo(da_camada(W::Visibility)), "o olho");
    assert!(alvo(da_camada(W::Blend)), "a mistura");
    assert!(alvo(da_camada(W::Opacity)), "a opacidade");
    assert!(alvo(da_camada(W::Depth)), "a profundidade");
    assert_eq!(
        key_mark_target(key_mark_id(da_camada(W::Opacity))).map(|c| c.fields),
        Some(vec![da_camada(W::Opacity), da_camada(W::OpacityNum)]),
        "a barra e o chip são UMA linha, uma marca"
    );
    assert!(!alvo(da_camada(W::Lock)), "o cadeado não é canal");
    assert!(
        !alvo(ids::flip_layer_widget_id(FUNDO, W::Visibility)),
        "a camada que a shell não declarou não é alvo"
    );
    set_key_marks(BTreeMap::new());
}

/// Sem camada ativa, Duplicate e Delete pintam-se apagados e sem alvo; o Add continua.
#[test]
fn without_an_active_layer_duplicate_and_delete_are_off() {
    ferramenta::force_cards_on_this_thread(Some(true));
    ph2d_panel_flip::set_current_flip_style(Some(em(FlipMode::Draw)));
    ph2d_panel_flip::set_current_flip_layers(FlipLayersSnapshot::default());
    let me = Mesa::<FlipPanel>::nova(FlipPanelState::default());
    assert!(me.rect(t::FLIP_LAYER_ADD).is_some());
    assert!(me.rect(ids::FLIP_LAYER_DUPLICATE).is_none());
    assert!(me.rect(ids::FLIP_LAYER_DELETE).is_none());
}

/// O «×» da lista cancela a ferramenta, como o de antes.
#[test]
fn the_list_close_cancels_the_tool() {
    mesa(em(FlipMode::Draw)).prova_o_cancelar(ids::FLIP_CLOSE);
}

/// O cabeçalho dobra sem pedir nada à ferramenta.
#[test]
fn every_header_folds_its_card() {
    let dentro = [
        (ids::FLIP_SEC_TOOL, tool_button(FlipMode::Draw).0),
        (ids::FLIP_SEC_BRUSH, ids::FLIP_SIZE_NUM),
        (ids::FLIP_SEC_COLOR, ids::FLIP_STROKE_SWATCH),
        (ids::FLIP_SEC_LAYERS, t::FLIP_LAYER_ADD),
    ];
    let mut me = mesa(em(FlipMode::Draw));
    for (secao, de_dentro) in dentro {
        me.prova_a_dobra(secao, de_dentro);
    }
}

/// Arrastar o cabeçalho reordena (a mesma pilha do Kanban) e grava a ordem.
#[test]
fn dragging_a_card_by_its_header_reorders_the_list() {
    use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
    let mut me = mesa(em(FlipMode::Draw));
    let a = me.rect(ids::FLIP_SEC_TOOL).expect("Tool");
    let b = me.rect(ids::FLIP_SEC_COLOR).expect("Color");
    let (x, y0) = (a.x + a.w * 0.4, a.y + m::CARD_HEAD_H * 0.5);
    let ev = |kind, y| PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: 0,
    };
    let _ = me.host.dispatch_pointer_event(ev(PointerKind::Down, y0));
    let _ = me
        .host
        .dispatch_pointer_event(ev(PointerKind::Move, y0 + m::CARD_HEAD_H));
    me.assenta();
    let alvo = b.y + b.h;
    let _ = me.host.dispatch_pointer_event(ev(PointerKind::Move, alvo));
    me.assenta();
    let _ = me.host.dispatch_pointer_event(ev(PointerKind::Up, alvo));
    me.assenta();
    let ordem = me.host.store().section_order().to_vec();
    let pos = |id| ordem.iter().position(|&k| k == id);
    assert!(
        pos(ids::FLIP_SEC_TOOL) > pos(ids::FLIP_SEC_BRUSH),
        "o Tool não desceu: {ordem:?}"
    );
}

/// Todo ícone mudo tem dica; o campo diz a unidade na dele.
#[test]
fn every_mute_control_has_a_tooltip() {
    let me = mesa(FlipStyleSnapshot {
        erase: EraseMode::Soft,
        ..em(FlipMode::Erase)
    });
    let store = me.host.store();
    let mudos: Vec<NodeId> = dicas::icon_only_ids(PANEL)
        .into_iter()
        .chain([
            t::FLIP_LINK_SIZE,
            ids::FLIP_CLOSE,
            ph2d_editor_core::panel::cards::list_menu_of(PANEL),
        ])
        .filter(|id| store.tooltip_for(*id).is_none())
        .collect();
    assert!(mudos.is_empty(), "sem dica: {mudos:?}");
    let dica = dicas::field_hint(store, ids::FLIP_SIZE_NUM).expect("campo marcado");
    assert!(dica.contains("(px)"), "{dica}");
}

/// `PH2D_CARDS=0`: o corpo de antes, com as barras pintadas.
#[test]
fn cards_off_paints_the_old_body() {
    ferramenta::force_cards_on_this_thread(Some(false));
    ph2d_panel_flip::set_current_flip_style(Some(em(FlipMode::Draw)));
    ph2d_panel_flip::set_current_flip_layers(camadas());
    let me = Mesa::<FlipPanel>::nova(FlipPanelState::default());
    assert!(me.rect(t::FLIP_SIZE).is_some());
    assert!(me.rect(ids::FLIP_SEC_TOOL).is_none());
    assert!(me.host.store().card_field(ids::FLIP_SIZE_NUM).is_none());
}
