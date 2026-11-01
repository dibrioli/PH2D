//! ⭐⭐ **F43 — Effects e Filters no layout do Figma** (dono 01/11: *«Nada de todos botões à
//! mostra. Mas sim dropdown, lista de efeitos, Propriedade dos efeitos abre na lateral esquerda.»*).
//!
//! Cada gate CLICA pelo ponteiro (pintar → rect → Down/Up → dispatcher → painel): o «+» abre a
//! lista dos tipos PUBLICADOS pelo motor, cada efeito é uma linha, e a linha abre os campos num
//! painel à ESQUERDA do Inspector.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{FilterKindView, FilterRowView, VectorPanel, ids};
use ph2d_tool_vector::ids as tv;
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

/// O rect do painel do Vector (o slot do Inspector) neste layout.
fn painel(h: &mut MockPanelHost) -> Rect {
    let _ = pintado(h, tv::VECTOR_FX_ADD_MENU);
    h.store()
        .panel_rect(ph2d_editor_core::ids::VECTOR_PANEL)
        .expect("o painel publicou o rect")
}

fn efeitos(n: usize) {
    let row = |label: &'static str| ph2d_panel_vector::FxRowView {
        label,
        enabled: true,
        params: vec![
            ph2d_panel_vector::FxParamView {
                name: "Size",
                min: 0.0,
                max: 100.0,
                toggle: false,
                integer: false,
                value: 12.0,
            },
            ph2d_panel_vector::FxParamView {
                name: "Smooth",
                min: 0.0,
                max: 1.0,
                toggle: true,
                integer: false,
                value: 0.0,
            },
        ],
        falloff_role: ph2d_panel_vector::FalloffRole::NotFalloff,
    };
    let stack = ["Zig Zag", "Wave", "Twist"]
        .into_iter()
        .take(n)
        .map(row)
        .collect();
    ph2d_panel_vector::set_current_effects(
        true,
        false,
        ph2d_vec_scene::effect::PathEffect::KINDS,
        stack,
    );
}

fn filtros() {
    let row = FilterRowView {
        kind: 0,
        mode: 0,
        enabled: true,
        radius: 0.12,
        offx: 0.1,
        offy: -0.1,
        color: [0, 0, 0, 255],
        color_b: [255, 255, 255, 255],
        opacity: 1.0,
        blend: 0,
        scale: 0.25,
        detail: 3,
        seed: 0,
        grow: 0.0,
        hue: 0.0,
        sat: 0.0,
        bright: 0.0,
        stop_pos: [0.0; 8],
        stop_colors: [[0; 4]; 8],
        stop_count: 0,
        ramp_preview: [[128; 3]; ph2d_panel_vector::RAMP_PREVIEW_N],
    };
    let kind = FilterKindView {
        name: "Drop Shadow",
        radius_label: Some("Radius"),
        offset_labels: Some(("Offset X", "Offset Y")),
        color_label: Some("Color"),
        color_b_label: None,
        modes: &[],
        takes_blend: false,
        takes_ramp: false,
        noise_labels: None,
        grow_label: None,
        adjust_labels: None,
    };
    let blur = FilterKindView {
        name: "Blur",
        offset_labels: None,
        color_label: None,
        ..kind
    };
    ph2d_panel_vector::set_current_filter_can_add(true);
    ph2d_panel_vector::set_filter_kinds(vec![blur, kind]);
    ph2d_panel_vector::set_current_filters(vec![row, FilterRowView { kind: 1, ..row }]);
}

/// ⭐ **Fechado, nenhum «Add» nem campo à mostra** — só o «+» e as linhas (glifo, nome, olho, «−»).
/// O controlo: as mesmas ids aparecem depois de abrir o «+» e a linha.
#[test]
fn closed_the_sections_show_a_plus_and_rows_never_the_buttons() {
    efeitos(2);
    filtros();
    let mut h = crate::host_aberto();
    for id in [
        tv::VECTOR_FX_ADD_MENU,
        tv::vector_fx_row_id(0),
        tv::vector_fx_row_id(1),
        tv::vector_fx_hide_id(1),
        tv::vector_fx_remove_id(1),
        ids::FILTER_ADD_MENU,
        ids::filter_row_id(1),
        ids::filter_hide_id(1),
        ids::filter_remove_id(1),
    ] {
        assert!(pintado(&mut h, id).is_some(), "{id:?} não foi pintado");
    }
    for k in 0..tv::MAX_FX_KINDS {
        assert!(
            pintado(&mut h, tv::vector_fx_add_id(k)).is_none(),
            "Add {k} à mostra"
        );
    }
    assert!(
        pintado(&mut h, ids::filter_add_id(0)).is_none(),
        "Add Blur à mostra"
    );
    for id in [
        tv::vector_fx_param_id(0, 0),
        tv::vector_fx_toggle_id(0, 1),
        tv::vector_fx_up_id(1),
        ids::filter_radius_id(0),
    ] {
        assert!(
            pintado(&mut h, id).is_none(),
            "o campo {id:?} está à mostra"
        );
    }
    crate::clica(&mut h, tv::vector_fx_row_id(0));
    assert!(
        pintado(&mut h, tv::vector_fx_param_id(0, 0)).is_some(),
        "controlo: a linha aberta mostra os campos"
    );
}

/// ⭐ **O «+» lista TODOS os tipos do motor; escolher um chega ao bus e FECHA a lista.**
#[test]
fn the_plus_lists_every_engine_kind_and_a_pick_adds_and_closes() {
    efeitos(1);
    let mut h = crate::host_com_aberto(tv::VECTOR_FX_ADD_MENU);
    let n = ph2d_vec_scene::effect::PathEffect::KINDS.len();
    for k in 0..n {
        assert!(
            pintado(&mut h, tv::vector_fx_add_id(k)).is_some(),
            "o tipo {k} não está na lista do «+»"
        );
    }
    let mut st = VectorPanelState;
    let r = pintado(&mut h, tv::vector_fx_add_id(5)).expect("Arc Upper");
    let evs = h.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == tv::vector_fx_add_id(5))),
        "o clique na linha da lista não virou Click"
    );
    for ev in evs {
        h.apply_panel_event::<VectorPanel>(&mut st, ev);
    }
    assert!(
        h.drained_actions().into_iter().any(|a| matches!(a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == tv::vector_fx_add_id(5))),
        "o Add escolhido na lista não chegou ao bus"
    );
    assert!(
        pintado(&mut h, tv::vector_fx_add_id(5)).is_none(),
        "a lista do «+» não fechou depois da escolha"
    );
}

/// ⭐⭐ **A linha abre os campos num painel à ESQUERDA do Inspector**, com o topo na linha; o X
/// fecha. O mesmo para os filtros.
#[test]
fn a_row_opens_its_properties_to_the_left_of_the_inspector() {
    efeitos(2);
    filtros();
    for (linha, campos, close) in [
        (
            tv::vector_fx_row_id(1),
            vec![tv::vector_fx_param_id(1, 0), tv::vector_fx_toggle_id(1, 1)],
            tv::VECTOR_FX_PROPS_CLOSE,
        ),
        (
            ids::filter_row_id(1),
            vec![
                ids::filter_radius_id(1),
                ids::filter_offx_id(1),
                ids::filter_color_id(1),
                ids::filter_opacity_id(1),
            ],
            ids::FILTER_PROPS_CLOSE,
        ),
    ] {
        let mut h = crate::host_com_aberto(linha);
        let insp = painel(&mut h);
        let l = pintado(&mut h, linha).expect("a linha");
        for id in &campos {
            let r = pintado(&mut h, *id).unwrap_or_else(|| panic!("{id:?} fora do painel"));
            assert!(
                r.x + r.w <= insp.x + 0.5,
                "{id:?} não está à esquerda do Inspector: {r:?} × {insp:?}"
            );
            assert!(r.y >= l.y - 0.5, "{id:?} acima da linha que o abriu");
        }
        crate::clica(&mut h, close);
        assert!(
            pintado(&mut h, campos[0]).is_none(),
            "o X não fechou o painel"
        );
    }
}

/// ⭐ **↑ no painel move o efeito E o painel acompanha-o** (a linha aberta passa a ser a de cima);
/// e o «−» fecha (a linha aberta deixaria de ser o mesmo efeito).
#[test]
fn moving_the_open_effect_keeps_its_panel_and_removing_closes_it() {
    efeitos(3);
    let mut h = crate::host_com_aberto(tv::vector_fx_row_id(2));
    assert!(
        pintado(&mut h, tv::vector_fx_down_id(2)).is_none(),
        "↓ na última linha é inerte e não se pinta"
    );
    crate::clica(&mut h, tv::vector_fx_up_id(2));
    assert!(
        h.drained_actions().into_iter().any(|a| matches!(a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == tv::vector_fx_up_id(2))),
        "o ↑ do painel não chegou ao bus"
    );
    assert!(
        pintado(&mut h, tv::vector_fx_param_id(1, 0)).is_some(),
        "o painel não acompanhou o efeito para a linha 1"
    );
    assert!(pintado(&mut h, tv::vector_fx_param_id(2, 0)).is_none());
    crate::clica(&mut h, tv::vector_fx_remove_id(0));
    assert!(
        pintado(&mut h, tv::vector_fx_param_id(1, 0)).is_none(),
        "o «−» não fechou o painel"
    );
}

/// O «+» apaga-se com a pilha CHEIA (não regista clique) — e acende com espaço.
#[test]
fn the_plus_is_dead_when_the_stack_is_full() {
    efeitos(1);
    let mut h = crate::host_aberto();
    assert!(
        pintado(&mut h, tv::VECTOR_FX_ADD_MENU).is_some(),
        "controlo"
    );
    let row = ph2d_panel_vector::FxRowView {
        label: "Twist",
        enabled: true,
        params: Vec::new(),
        falloff_role: ph2d_panel_vector::FalloffRole::NotFalloff,
    };
    ph2d_panel_vector::set_current_effects(
        true,
        false,
        ph2d_vec_scene::effect::PathEffect::KINDS,
        vec![row; tv::MAX_FX_ROWS],
    );
    assert!(
        pintado(&mut h, tv::VECTOR_FX_ADD_MENU).is_none(),
        "o «+» aceitou clique com a pilha cheia"
    );
}
