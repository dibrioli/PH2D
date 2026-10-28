//! As medidas da aba, em px LÓGICOS — as do Kanban (a maquete aprovada, `ph2d-app-kanban/metrics`):
//! a barra de cima tem a altura do cabeçalho do quadro, os botões e o texto os mesmos. As cores vêm
//! dos tokens; aqui só a geometria.

pub(crate) const TOP_H: f32 = 52.0;
pub(crate) const PAD: f32 = 12.0;
pub(crate) const GAP: f32 = 8.0;
pub(crate) const BUTTON_H: f32 = 32.0;
pub(crate) const BUTTON_PAD_X: f32 = 10.0;
pub(crate) const RADIUS: f32 = 6.0;
pub(crate) const CARD_RADIUS: f32 = 8.0;
pub(crate) const TEXT_PX: f32 = 14.0;
pub(crate) const SMALL_PX: f32 = 12.0;
pub(crate) const TITLE_PX: f32 = 18.0;
pub(crate) const BADGE_H: f32 = 18.0;
pub(crate) const BADGE_PAD_X: f32 = 6.0;

/// A coluna do Planner (Inbox + cartões sem data) — a largura de uma lista do Kanban.
pub(crate) const SIDE_W: f32 = 272.0;
pub(crate) const ROW_H: f32 = 36.0;
pub(crate) const ROW_GAP: f32 = 6.0;
pub(crate) const SECTION_H: f32 = 28.0;

/// «My cards»: a coluna de cartões (centrada) e o título de cada grupo.
pub(crate) const LIST_W: f32 = 640.0;
pub(crate) const GROUP_H: f32 = 32.0;

/// O sino: a caixa das notificações sob ele.
pub(crate) const BELL_W: f32 = 360.0;
pub(crate) const NOTICE_H: f32 = 56.0;

/// O popover do evento (o seletor de datas por baixo do título).
pub(crate) const POP_W: f32 = 320.0;
pub(crate) const FIELD_H: f32 = 32.0;

/// O bloco que nasce de um clique em «Schedule» (meia hora, a grelha do Planner do Trello).
pub(crate) const BLOCK_MIN: u32 = 30;
/// O passo da hora de um bloco novo: a próxima meia hora.
pub(crate) const SLOT_MS: i64 = 30 * 60_000;
