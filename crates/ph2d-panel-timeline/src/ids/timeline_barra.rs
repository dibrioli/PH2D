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
