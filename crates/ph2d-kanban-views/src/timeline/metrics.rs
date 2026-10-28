//! As medidas da Linha do tempo, em px LÓGICOS (o idioma da Timeline do Trello sobre a grelha do
//! quadro: faixas por grupo, barras de cantos `4`, cabeçalho de escala em duas filas).

// ── A barra de ferramentas ──
pub(crate) const TOOLBAR_H: f32 = 48.0;
pub(crate) const TOOLBAR_PAD: f32 = 12.0;
pub(crate) const BUTTON_H: f32 = 28.0;
pub(crate) const BUTTON_PAD_X: f32 = 10.0;
pub(crate) const BUTTON_GAP: f32 = 4.0;
pub(crate) const GROUP_GAP: f32 = 16.0;
pub(crate) const RADIUS: f32 = 6.0;
pub(crate) const TEXT_PX: f32 = 14.0;
pub(crate) const SMALL_PX: f32 = 12.0;

// ── A escala ──
pub(crate) const SCALE_TOP_H: f32 = 24.0;
pub(crate) const SCALE_BOTTOM_H: f32 = 24.0;
pub(crate) const SCALE_PAD_X: f32 = 6.0;
/// Px por dia em cada zoom (Dia, Semana, Mês, Trimestre): uma coluna de dia lê o título de um
/// cartão (Dia), sete dias cabem no que antes era um (Semana), um mês e um trimestre por ecrã.
pub(crate) const PX_PER_DAY: [f32; 4] = [120.0, 44.0, 16.0, 6.0];

// ── As faixas ──
pub(crate) const LANE_LABEL_W: f32 = 200.0;
pub(crate) const LANE_PAD_Y: f32 = 8.0;
pub(crate) const LANE_LABEL_PAD_X: f32 = 14.0;
pub(crate) const ROW_H: f32 = 36.0;
pub(crate) const LINE: f32 = 1.0;

// ── As barras ──
pub(crate) const BAR_H: f32 = 28.0;
pub(crate) const BAR_RADIUS: f32 = 4.0;
pub(crate) const BAR_PAD_X: f32 = 8.0;
/// A pega de cada ponta (o cursor de redimensionar do Trello).
pub(crate) const HANDLE_W: f32 = 8.0;
pub(crate) const HANDLE_GRIP_W: f32 = 2.0;
pub(crate) const HANDLE_GRIP_H: f32 = 12.0;
/// Uma barra estreita demais para o título escreve-o à direita, fora dela.
pub(crate) const BAR_MIN_TEXT_W: f32 = 40.0;
pub(crate) const DONE_ALPHA: f32 = 0.55;
/// Px de movimento antes de um carregar virar arrasto (abaixo disso é um clique).
pub(crate) const DRAG_SLOP: f32 = 4.0;
pub(crate) const FLIGHT_SHADOW: f32 = 3.0;

// ── Hoje ──
pub(crate) const TODAY_W: f32 = 2.0;
pub(crate) const TODAY_DOT: f32 = 4.0;

// ── A gaveta «No dates» ──
pub(crate) const DRAWER_W: f32 = 260.0;
pub(crate) const DRAWER_PAD: f32 = 12.0;
pub(crate) const DRAWER_HEAD_H: f32 = 40.0;
pub(crate) const DRAWER_CARD_H: f32 = 36.0;
pub(crate) const DRAWER_GAP: f32 = 8.0;
pub(crate) const CLOSE_W: f32 = 28.0;

// ── A roda ──
pub(crate) const WHEEL_PX: f32 = 1.0;
