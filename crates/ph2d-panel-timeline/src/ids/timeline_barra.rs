//! A barra no molde do Rive (`docs/Timeline/15_plano_redesenho_molde_rive.md`, onda 1): as duas portas
//! das janelinhas e as três células de «When the clip ends». Nenhuma é função nova: as células pedem os
//! eventos de `TIMELINE_LOOP` / `TIMELINE_PINGPONG`.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// A engrenagem «Timeline options» (abre a janelinha das opções da vista e do relógio).
pub const TIMELINE_OPTIONS: NodeId = hash_node_id("timeline.options");
/// O «⋯» do chip do clipe ativo (abre o menu do clipe).
pub const TIMELINE_CLIP_MENU: NodeId = hash_node_id("timeline.clip_menu");
/// «When the clip ends: Stop».
pub const TIMELINE_END_STOP: NodeId = hash_node_id("timeline.end_stop");
/// «… Loop».
pub const TIMELINE_END_LOOP: NodeId = hash_node_id("timeline.end_loop");
/// «… Ping-Pong».
pub const TIMELINE_END_PINGPONG: NodeId = hash_node_id("timeline.end_pingpong");

/// Doc 17, item 5: as duas células da vista «Keys | Curves» (só na aba Keys, nos cartões).
pub const TIMELINE_VIEW_KEYS: NodeId = hash_node_id("timeline.view_keys");
/// «Curves»: a área das faixas vira UM grafo com as curvas de todas as linhas com chaves.
pub const TIMELINE_VIEW_CURVES: NodeId = hash_node_id("timeline.view_curves");

/// O olho de uma linha na vista Curves: esconde ou mostra a curva dela.
#[must_use]
pub fn timeline_curve_eye_id(target: u64) -> NodeId {
    super::timeline::dynamic_id("timeline.curve_eye", &[target])
}
