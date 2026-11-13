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
/// O widget do fim na 2.ª fileira (dono 13/11): o nome do escolhido abre a lista.
pub const TIMELINE_END_DD: NodeId = hash_node_id("timeline.end_dd");

/// «Edit curves on canvas» (dono 10/11): a caixa da barra, só na aba Keys e nos cartões; ligada, o
/// canvas mostra as alças que entortam o caminho do objeto.
pub const TIMELINE_CURVES_ON_CANVAS: NodeId = hash_node_id("timeline.curves_on_canvas");

/// «Show clips as a list» (dono 12/11), em «Timeline options»: os chips dos clipes viram UMA lista suspensa.
pub const TIMELINE_CLIPS_AS_LIST: NodeId = hash_node_id("timeline.clips_as_list");

/// O «Done» da pílula «Animating …» (doc 17 item 9, B7): devolve a cena e pára a prévia do clipe.
pub const TIMELINE_ANIMATING_DONE: NodeId = hash_node_id("timeline.animating_done");

/// O Stop dos controlos flutuantes (dono 13/11): pára e volta ao início.
pub const TIMELINE_STOP: NodeId = hash_node_id("timeline.stop");
