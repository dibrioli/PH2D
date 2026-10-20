//! **O losango ao lado de um campo do painel do vetor (plano 35 §5) é pintado, registado, e um clique
//! REAL nele chega à shell** — como pedido de chave, sem mexer no campo.
//!
//! A shell publica o mapa (`set_key_marks`); a pintura do painel regista o losango no índice de alvos
//! pelo id derivado (`key_mark_id`); o «Down» do despacho entrega o pedido (`take_key_mark_click`).
//! O `architecture_panel_wiring_parity` não vê ids derivados — este teste é o DoD.

use std::collections::BTreeMap;

use ph2d_editor_core::widget::key_mark::{
    KeyMark, key_mark_id, set_key_marks, take_key_mark_click,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::VectorPanel;
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_tool_vector::ids::{VECTOR_FILL_SWATCH, VECTOR_WIDTH, VECTOR_WIDTH_NUM};

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};

fn ev(kind: PointerKind, r: Rect, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x: r.x + r.w * 0.5,
        y: r.y + r.h * 0.5,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

/// O losango da largura do traço existe sob o rato quando a shell diz que o canal tem chave, e o
/// clique nele é um pedido com os campos da linha (o slider e o número) — e não um arrasto do
/// slider. (Mutação: tirar o braço do losango no `dispatch_down` ⇒ nenhum pedido; não registar no
/// `paint_decorator_for` ⇒ o `expect` dispara.)
#[test]
fn clicar_no_losango_da_largura_pede_a_chave_do_canal() {
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    set_key_marks(BTreeMap::from([(VECTOR_WIDTH, KeyMark::Chave)]));
    let _ = take_key_mark_click();
    let mut host = crate::host_aberto();
    let mut st = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, key_mark_id(VECTOR_WIDTH))
        .expect("o losango da largura foi pintado mas não registado (morto sob o rato)");
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, key_mark_id(VECTOR_FILL_SWATCH))
            .is_none(),
        "um campo que a shell não declarou não é alvo"
    );

    let _ = host.dispatch_pointer_event(ev(PointerKind::Down, r, 1_000_000_000));
    let _ = host.dispatch_pointer_event(ev(PointerKind::Up, r, 1_010_000_000));

    let c = take_key_mark_click().expect("o clique no losango não chegou à shell");
    assert_eq!(c.fields, vec![VECTOR_WIDTH, VECTOR_WIDTH_NUM]);
    set_key_marks(BTreeMap::new());
}

/// O clique REAL num losango e o pedido que chega à shell.
fn clica_no_losango(primeiro: ph2d_a11y::NodeId, o_que: &str) -> Vec<ph2d_a11y::NodeId> {
    let _ = take_key_mark_click();
    let mut host = crate::host_aberto();
    let mut st = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, key_mark_id(primeiro))
        .unwrap_or_else(|| panic!("o losango de {o_que} foi pintado mas não registado"));
    let _ = host.dispatch_pointer_event(ev(PointerKind::Down, r, 1_000_000_000));
    let _ = host.dispatch_pointer_event(ev(PointerKind::Up, r, 1_010_000_000));
    take_key_mark_click()
        .unwrap_or_else(|| panic!("o clique no losango de {o_que} não chegou à shell"))
        .fields
}

/// ⭐ A linha de ESCOLHA (Cap) tem losango: os três segmentos são os campos dela, e o clique pede
/// a chave sem escolher um segmento. (Mutação: o `decorador` de `paint_choice_row` de volta ao ponto
/// ⇒ o `expect` dispara.)
#[test]
fn a_linha_de_escolha_tem_losango() {
    use ph2d_tool_vector::ids::{VECTOR_CAP_BUTT, VECTOR_CAP_ROUND, VECTOR_CAP_SQUARE};
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    set_key_marks(BTreeMap::from([(VECTOR_CAP_BUTT, KeyMark::Animado)]));
    assert_eq!(
        clica_no_losango(VECTOR_CAP_BUTT, "Cap"),
        vec![VECTOR_CAP_BUTT, VECTOR_CAP_ROUND, VECTOR_CAP_SQUARE]
    );
    set_key_marks(BTreeMap::new());
}

/// ⭐ A linha de cada entrada da pilha de pinturas tem o losango da TINTA dela, na coluna do
/// decorator — e a coluna é a mesma das outras linhas (os verbos começam antes dela).
#[test]
fn a_entrada_da_pilha_tem_o_losango_da_tinta() {
    use ph2d_panel_vector::state::{self, Appearance, PaintRow};
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    let linha = PaintRow {
        is_fill: true,
        color: [0xCC, 0x33, 0x33, 0xFF],
        width: 0.0,
        enabled: true,
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        offset: [0.0, 0.0],
        dilate: 0.0,
        dilate_join: 1,
        mixed: Default::default(),
    };
    state::set_current_appearance(Some(Appearance {
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        layers: vec![linha],
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
    }));
    let sw = ph2d_panel_vector::ids::vector_paint_swatch_id(0);
    set_key_marks(BTreeMap::from([(sw, KeyMark::Chave)]));
    let mut host = crate::host_aberto();
    let mut st = VectorPanelState;
    let losango = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, key_mark_id(sw))
        .expect("o losango da entrada não foi registado");
    let apagar = host
        .painted_rect::<VectorPanel>(
            &mut st,
            VIEWPORT,
            ph2d_panel_vector::ids::vector_paint_del_id(0),
        )
        .expect("o lixo da entrada");
    assert!(
        apagar.x + apagar.w <= losango.x + 0.5,
        "os verbos começam ANTES da coluna do losango: {apagar:?} × {losango:?}"
    );
    assert_eq!(clica_no_losango(sw, "a entrada"), vec![sw]);
    set_key_marks(BTreeMap::new());
    state::set_current_appearance(None);
}
