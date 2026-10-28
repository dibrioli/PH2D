//! As medidas do calendário, em px LÓGICOS (as do quadro: a grelha `4`, o texto `14/12`, as
//! esquinas `4`/`8` do cartão). As cores vêm dos tokens `kanban-*`; aqui só a geometria.

// ── O cabeçalho ──
pub(crate) const PAD: f32 = 12.0;
pub(crate) const HEADER_H: f32 = 48.0;
pub(crate) const TITLE_PX: f32 = 18.0;
pub(crate) const BUTTON_H: f32 = 32.0;
pub(crate) const BUTTON_PAD_X: f32 = 12.0;
pub(crate) const BUTTON_GAP: f32 = 6.0;
pub(crate) const BUTTON_RADIUS: f32 = 6.0;
pub(crate) const BUTTON_PX: f32 = 14.0;
/// O botão de seta (‹ ›): quadrado.
pub(crate) const ARROW_W: f32 = 32.0;
/// A seta desenhada: meia-altura do chevron e a espessura do traço.
pub(crate) const CHEVRON: f32 = 5.0;
pub(crate) const CHEVRON_STROKE: f32 = 2.0;

// ── A grelha ──
pub(crate) const WEEKDAY_H: f32 = 28.0;
pub(crate) const WEEKDAY_PX: f32 = 12.0;
pub(crate) const GRID_RADIUS: f32 = 8.0;
pub(crate) const GRID_LINE: f32 = 1.0;
pub(crate) const DAY_NUM_H: f32 = 28.0;
pub(crate) const DAY_NUM_PX: f32 = 13.0;
/// O círculo do dia de hoje.
pub(crate) const TODAY_R: f32 = 11.0;
pub(crate) const CELL_PAD: f32 = 4.0;
/// O «+» de um dia (ao pairar): quadrado, no canto de cima à direita.
pub(crate) const PLUS_W: f32 = 22.0;
pub(crate) const PLUS_ARM: f32 = 5.0;

// ── Os itens ──
pub(crate) const LANE_H: f32 = 22.0;
/// Na semana as faixas são mais altas (a hora vai à frente do título).
pub(crate) const LANE_H_WEEK: f32 = 26.0;
pub(crate) const LANE_GAP: f32 = 3.0;
pub(crate) const ITEM_PX: f32 = 12.0;
pub(crate) const ITEM_RADIUS: f32 = 4.0;
pub(crate) const ITEM_PAD_X: f32 = 6.0;
/// A faixa de cor à esquerda de um item de um dia.
pub(crate) const STRIP_W: f32 = 3.0;
/// A marca de concluído: o círculo e o traço do visto.
pub(crate) const DONE_R: f32 = 6.0;
pub(crate) const DONE_STROKE: f32 = 1.5;
/// O visto dentro do círculo de raio `1`: três pontos.
pub(crate) const DONE_TICK: [(f32, f32); 3] = [(-0.45, 0.0), (-0.1, 0.35), (0.5, -0.35)];
/// A linha «+N more».
pub(crate) const MORE_H: f32 = 18.0;

// ── O arrasto ──
/// Quanto o rato anda antes de um clique virar arrasto.
pub(crate) const DRAG_SLOP: f32 = 4.0;
/// A roda: quanto `dy` acumulado muda um mês (ou uma semana).
pub(crate) const WHEEL_STEP: f32 = 60.0;

// ── Os popovers (o compositor do «+» e a lista de um dia) ──
pub(crate) const POP_W: f32 = 272.0;
pub(crate) const POP_PAD: f32 = 12.0;
pub(crate) const POP_RADIUS: f32 = 8.0;
pub(crate) const POP_TITLE_H: f32 = 28.0;
pub(crate) const POP_ROW_H: f32 = 28.0;
pub(crate) const POP_GAP: f32 = 8.0;
pub(crate) const POP_SHADOW: f32 = 3.0;
pub(crate) const FIELD_H: f32 = 32.0;
pub(crate) const FIELD_PX: f32 = 14.0;
pub(crate) const CARET_W: f32 = 1.5;
pub(crate) const FOCUS_STROKE: f32 = 2.0;
/// A hora de um cartão criado num dia: o meio-dia (a do seletor de datas por omissão).
pub(crate) const NEW_DUE_HOUR: u8 = 12;
