//! **Os ids da secção TIMERS** (TOP-20 #2, W3).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — mesmo padrão do
//! [`super::inspector_anim`] e do [`super::inspector_anchor`].
//!
//! # ⚠️ A lista + UM editor, e não seis controlos por linha
//!
//! Um `Timers` guarda até [`ph2d_ecs::TIMERS_MAX`] timers e cada um tem **cinco** campos. Desenhar
//! os cinco em cada linha custaria `5 × 16 = 80` ids e uma coluna que não cabe na largura do
//! Inspector. ⇒ o molde é o da §11 Animation: a **lista** escolhe qual timer está aberto, e um
//! editor só, abaixo dela, mostra os campos desse.
//!
//! ⚠️ **A escolha da linha NÃO vai ao barramento**, e aqui a §12 é que é o precedente certo: qual
//! timer se edita é um facto da UI e vive no `InspectorState`. Na §11 a linha aberta **é** a
//! animação que toca, que é estado da cena — um `Timers` não tem «o timer actual».

use super::*;

/// A secção TIMERS — o cabeçalho colapsável. Entra em [`super::LIVE_SECTIONS`] com o ponto de cor.
pub const INSP_LIVE_TIMER_SECTION: NodeId = hash_node_id("insp_live_timer_section");
/// TIMERS — ponto de cor do cabeçalho.
pub const INSP_LIVE_TIMER_COLOR: NodeId = hash_node_id("insp_live_timer_color");

/// **As linhas da lista** — uma por timer, até ao cap de [`ph2d_ecs::TIMERS_MAX`].
///
/// ⚠️ O comprimento deste array **é** o cap do modelo, e há gate na shell a prendê-los: *um modelo
/// que aceita o que o painel não mostra produz estado inalcançável por gesto nenhum* (a lei que o
/// `ANIM_TAGS_MAX` já pagou, com o número a descer de 256 para 64 por causa dela).
pub const INSP_TIMER_ROW: [NodeId; 16] = [
    hash_node_id("insp_timer_row_00"),
    hash_node_id("insp_timer_row_01"),
    hash_node_id("insp_timer_row_02"),
    hash_node_id("insp_timer_row_03"),
    hash_node_id("insp_timer_row_04"),
    hash_node_id("insp_timer_row_05"),
    hash_node_id("insp_timer_row_06"),
    hash_node_id("insp_timer_row_07"),
    hash_node_id("insp_timer_row_08"),
    hash_node_id("insp_timer_row_09"),
    hash_node_id("insp_timer_row_10"),
    hash_node_id("insp_timer_row_11"),
    hash_node_id("insp_timer_row_12"),
    hash_node_id("insp_timer_row_13"),
    hash_node_id("insp_timer_row_14"),
    hash_node_id("insp_timer_row_15"),
];
