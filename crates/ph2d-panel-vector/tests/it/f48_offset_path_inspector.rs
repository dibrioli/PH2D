//! ⭐ **F48 — o Offset Path no padrão do Inspector** (dono 02/11: «expand não é intuitivo… faça tudo
//! como no inspector»). Cada gate CLICA pelo ponteiro: o `Distance (px)` é um campo, o `Holes` não
//! oferece o `Inner` que só movia furos, o `Result` e o `Remove` gravam onde a shell lê, o `Limit`
//! só existe no Miter, e o perfil de largura mora no «Stroke settings».

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{EXPAND_KNOBS_AT_BIRTH, VectorPanel, ids};
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

fn chegou(h: &mut MockPanelHost, id: NodeId) -> bool {
    h.drained_actions()
        .into_iter()
        .any(|a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id))
}

/// ⭐ `Distance (px)` é um CAMPO do Inspector (número com setas), não um slider em %. *Mutação:
/// voltar o `slider_chip` no `populate` ⇒ vermelho (o store não tem número).*
#[test]
fn a_distancia_e_um_campo_numerico() {
    let mut h = host();
    assert!(pintado(&mut h, ids::VECTOR_EXPAND_OFFSET).is_some());
    assert_eq!(h.store().number_value(ids::VECTOR_EXPAND_OFFSET), Some(0.0));
}

/// ⛔ O `Inner` movia só os FUROS — numa forma sem furo não fazia nada. O `Holes` oferece os dois
/// que fazem sentido. *Mutação: repor o chip `Inner` na secção ⇒ vermelho.*
#[test]
fn o_inner_que_nao_fazia_nada_saiu() {
    let mut h = host();
    assert!(pintado(&mut h, ph2d_tool_vector::ids::VECTOR_EXPAND_SIDE_INNER).is_none());
    assert!(pintado(&mut h, ph2d_tool_vector::ids::VECTOR_EXPAND_SIDE_BOTH).is_some());
    assert!(pintado(&mut h, ph2d_tool_vector::ids::VECTOR_EXPAND_SIDE_OUTER).is_some());
}

/// ⭐ O `Result` nasce `New shape behind` e o clique grava-o. *Mutação: o braço do `Result` no
/// `event.rs` gravar sempre `true` ⇒ vermelho.*
#[test]
fn o_result_grava_o_que_se_clica() {
    let mut h = host();
    let result = || ph2d_panel_vector::offset_knobs().result;
    assert_eq!(result(), 1, "nasce atrás");
    crate::clica(&mut h, ids::VECTOR_EXPAND_RESULT_REPLACE);
    assert_eq!(result(), 0);
    crate::clica(&mut h, ids::VECTOR_EXPAND_RESULT_FRONT);
    assert_eq!(result(), 2, "pela frente (dono 02/11)");
    crate::clica(&mut h, ids::VECTOR_EXPAND_RESULT_BEHIND);
    assert_eq!(result(), 1);
}

/// ⭐ O `Miter limit` só existe com a quina Miter. *Mutação: tirar o `if k.join == 0` ⇒ vermelho.*
#[test]
fn o_limite_so_aparece_no_miter() {
    let mut h = host();
    assert!(pintado(&mut h, ids::VECTOR_EXPAND_MITER).is_some());
    crate::clica(&mut h, ph2d_tool_vector::ids::VECTOR_EXPAND_JOIN_ROUND);
    assert_eq!(ph2d_panel_vector::expand_join(), 1);
    assert!(pintado(&mut h, ids::VECTOR_EXPAND_MITER).is_none());
    ph2d_panel_vector::set_offset_knobs(EXPAND_KNOBS_AT_BIRTH);
}

/// ⭐ `Apply` e `Remove` só com um offset VIVO; o `Remove` zera a distância (a shell desarma pela
/// mesma porta de quem digita `0`) e não vai ao bus. *Mutação: tirar o `set_number_value` do
/// braço do `Remove` ⇒ vermelho.*
#[test]
fn apply_e_remove_so_com_offset_vivo() {
    let mut h = host();
    assert!(pintado(&mut h, ph2d_tool_vector::ids::VECTOR_EXPAND_OFFSET_PATH).is_none());
    assert!(pintado(&mut h, ids::VECTOR_EXPAND_REMOVE).is_none());
    ph2d_panel_vector::set_offset_live_on(true);
    h.store_mut()
        .set_number_value(ids::VECTOR_EXPAND_OFFSET, 12.0);
    crate::clica(&mut h, ids::VECTOR_EXPAND_REMOVE);
    assert_eq!(h.store().number_value(ids::VECTOR_EXPAND_OFFSET), Some(0.0));
    assert!(!chegou(&mut h, ids::VECTOR_EXPAND_REMOVE));
    crate::clica(&mut h, ph2d_tool_vector::ids::VECTOR_EXPAND_OFFSET_PATH);
    assert!(chegou(
        &mut h,
        ph2d_tool_vector::ids::VECTOR_EXPAND_OFFSET_PATH
    ));
    ph2d_panel_vector::set_offset_live_on(false);
}

/// ⭐ O perfil de largura (os quatro W e o `Apply Power Stroke`) mora no «Stroke settings», e o
/// Offset Path já não o pinta. *Mutação: tirar o `power_stroke_rows` do `stroke_settings_body` ⇒
/// vermelho.*
#[test]
fn o_perfil_de_largura_mora_no_stroke_settings() {
    let mut h = host();
    let power = ph2d_tool_vector::ids::VECTOR_EXPAND_POWER_STROKE;
    assert!(pintado(&mut h, power).is_none(), "ainda no Offset Path");
    assert!(pintado(&mut h, ids::VECTOR_EXPAND_W_START).is_none());
    crate::clica(&mut h, ids::VECTOR_STROKE_SETTINGS);
    assert!(pintado(&mut h, ids::VECTOR_EXPAND_W_START).is_some());
    assert!(pintado(&mut h, power).is_some());
    crate::clica(&mut h, power);
    assert!(chegou(&mut h, power), "pintado e morto");
    ph2d_panel_vector::state::close_open_layer();
}

/// ⭐ O `Outline Stroke` saiu do Offset Path para a secção `Convert`, e clica.
#[test]
fn o_outline_stroke_mora_no_convert() {
    let mut h = host();
    let id = ph2d_tool_vector::ids::VECTOR_EXPAND_OUTLINE_STROKE;
    assert!(pintado(&mut h, id).is_some());
    crate::clica(&mut h, id);
    assert!(chegou(&mut h, id));
}
