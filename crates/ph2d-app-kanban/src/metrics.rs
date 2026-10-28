//! As medidas do quadro, em px LÓGICOS — as da maquete aprovada pelo dono (27/10,
//! `docs/TrelloClone/maquete/project/Main.dc.html` e `Card.dc.html`), que são as do Trello. As cores
//! vêm dos tokens `kanban-*`; aqui só a geometria.

// ── O cabeçalho do Kanban ──
pub(crate) const HEADER_H: f32 = 52.0;
pub(crate) const HEADER_PAD_L: f32 = 16.0;
pub(crate) const HEADER_PAD_R: f32 = 12.0;
pub(crate) const HEADER_GAP: f32 = 8.0;
pub(crate) const HEADER_TITLE_PX: f32 = 18.0;
pub(crate) const BUTTON_H: f32 = 32.0;
pub(crate) const BUTTON_RADIUS: f32 = 6.0;
pub(crate) const BUTTON_PAD_X: f32 = 10.0;
pub(crate) const BUTTON_TEXT_PX: f32 = 14.0;
pub(crate) const ICON_PX: f32 = 16.0;

// ── O quadro e as listas ──
pub(crate) const BOARD_PAD: f32 = 12.0;
pub(crate) const LIST_GAP: f32 = 12.0;
pub(crate) const LIST_W: f32 = 272.0;
pub(crate) const LIST_RADIUS: f32 = 12.0;
pub(crate) const COLLAPSED_W: f32 = 44.0;
/// O título da lista: `10` acima, `14` à esquerda, `8` à direita, `6` abaixo, linha de `20`.
pub(crate) const LIST_HEAD_TOP: f32 = 10.0;
pub(crate) const LIST_HEAD_LEFT: f32 = 14.0;
pub(crate) const LIST_HEAD_RIGHT: f32 = 8.0;
pub(crate) const LIST_HEAD_BOTTOM: f32 = 6.0;
/// O título da lista, em MAIÚSCULAS (dono 28/10: «maior e mais destacado»; a maquete tinha `14`).
pub(crate) const LIST_TITLE_PX: f32 = 15.0;
pub(crate) const LINE_H: f32 = 20.0;
pub(crate) const LIST_ICON_BTN: f32 = 28.0;
pub(crate) const LIST_ICON_GAP: f32 = 6.0;
/// O corpo da lista (os cartões): `2` acima, `8` dos lados, `4` abaixo.
pub(crate) const LIST_BODY_TOP: f32 = 2.0;
pub(crate) const LIST_BODY_X: f32 = 8.0;
pub(crate) const LIST_BODY_BOTTOM: f32 = 4.0;
pub(crate) const CARD_GAP: f32 = 8.0;
/// O pé da lista («Add a card»): `4` acima, `8` dos lados e abaixo.
pub(crate) const LIST_FOOT_TOP: f32 = 4.0;
pub(crate) const LIST_FOOT_BOTTOM: f32 = 8.0;
pub(crate) const ADD_RADIUS: f32 = 8.0;
pub(crate) const ADD_PAD_X: f32 = 8.0;
pub(crate) const NEW_LIST_H: f32 = 44.0;
pub(crate) const NEW_LIST_PAD_X: f32 = 12.0;
pub(crate) const COUNT_RADIUS: f32 = 10.0;
pub(crate) const COUNT_PAD_X: f32 = 6.0;
pub(crate) const COUNT_H: f32 = 18.0;
pub(crate) const SMALL_PX: f32 = 12.0;
/// A pílula do limite de cartões («3 / 3») tem cantos de `4` (a contagem da recolhida, de `10`).
pub(crate) const LIMIT_RADIUS: f32 = 4.0;
/// O glifo de recolher: duas setas na grelha `24` dos ícones — `‹›` na lista aberta (afastam-se),
/// `›‹` na recolhida (juntam-se). Os pontos da maquete (`M9 6l-4 6 4 6M15 6l4 6-4 6`).
pub(crate) const ICON_GRID: f64 = 24.0;
pub(crate) const COLLAPSE_OPEN: [[(f64, f64); 3]; 2] = [
    [(9.0, 6.0), (5.0, 12.0), (9.0, 18.0)],
    [(15.0, 6.0), (19.0, 12.0), (15.0, 18.0)],
];
pub(crate) const COLLAPSE_SHUT: [[(f64, f64); 3]; 2] = [
    [(5.0, 6.0), (9.0, 12.0), (5.0, 18.0)],
    [(19.0, 6.0), (15.0, 12.0), (19.0, 18.0)],
];
/// O traço do glifo, na grelha `24` (o `stroke-width="1.8"` da maquete).
pub(crate) const GLYPH_STROKE: f64 = 1.8;

// ── O cartão (a frente) ──
pub(crate) const CARD_W: f32 = 256.0;
pub(crate) const CARD_RADIUS: f32 = 8.0;
pub(crate) const CARD_PAD_X: f32 = 12.0;
pub(crate) const CARD_PAD_Y: f32 = 8.0;
pub(crate) const CARD_ROW_GAP: f32 = 6.0;
pub(crate) const CARD_TITLE_PX: f32 = 14.0;
pub(crate) const LABEL_H: f32 = 16.0;
pub(crate) const LABEL_MIN_W: f32 = 48.0;
pub(crate) const LABEL_PAD_X: f32 = 8.0;
pub(crate) const LABEL_RADIUS: f32 = 4.0;
pub(crate) const LABEL_GAP: f32 = 4.0;
pub(crate) const BADGE_ROW_H: f32 = 24.0;
pub(crate) const BADGE_ICON: f32 = 14.0;
pub(crate) const BADGE_GAP: f32 = 6.0;
pub(crate) const BADGE_PAD: f32 = 4.0;
pub(crate) const AVATAR: f32 = 24.0;
pub(crate) const AVATAR_OVERLAP: f32 = 4.0;
pub(crate) const AVATAR_RING: f32 = 2.0;
pub(crate) const AVATAR_PX: f32 = 10.0;
pub(crate) const EDIT_BTN: f32 = 32.0;
pub(crate) const EDIT_INSET: f32 = 6.0;
pub(crate) const COVER_COLOR_H: f32 = 36.0;
pub(crate) const FOCUS_RING: f32 = 2.0;
pub(crate) const FOCUS_OFFSET: f32 = 2.0;
/// O cartão em voo inclina 4° (o Trello) e roda em torno de `(50 %, 60 %)`.
pub(crate) const FLIGHT_TILT_DEG: f64 = 4.0;
pub(crate) const FLIGHT_PIVOT_Y: f64 = 0.6;

// ── Sombras: `(dy, desfoque)` de cada camada; as cores são `kanban-shadow-*` ──
pub(crate) const SHADOW_NEAR: (f64, f64) = (1.0, 1.0);
pub(crate) const SHADOW_RIM: (f64, f64) = (0.0, 1.0);
/// A sombra de pairar no claro (no escuro é um anel de 1 px).
pub(crate) const SHADOW_HOVER_LIGHT: (f64, f64) = (4.0, 8.0);
pub(crate) const SHADOW_FLIGHT_DARK: (f64, f64) = (14.0, 30.0);
pub(crate) const SHADOW_FLIGHT_LIGHT: (f64, f64) = (16.0, 32.0);
pub(crate) const SHADOW_FLIGHT_NEAR: (f64, f64) = (3.0, 8.0);
pub(crate) const HOVER_RING: f32 = 1.0;

// ── O gesto ──
/// A faixa junto à borda que rola o quadro (ou a lista) durante um arrasto, e a velocidade no
/// fundo dela. O Trello rola ao chegar à borda; a velocidade cresce com a fundura.
pub(crate) const EDGE_SCROLL_BAND: f32 = 64.0;
pub(crate) const EDGE_SCROLL_MAX_PX_S: f32 = 900.0;
/// Px da vista por «linha» de roda.
pub(crate) const WHEEL_STEP: f32 = 48.0;
/// Px que a janela dá por linha de roda (o espelho do `on_mouse_wheel` da shell e do Quadro).
pub(crate) const WHEEL_LINE_PX: f32 = 16.0;

// ── O campo de texto no lugar ──
pub(crate) const FIELD_PAD_X: f32 = 8.0;
pub(crate) const FIELD_PAD_Y: f32 = 6.0;
pub(crate) const FIELD_RADIUS: f32 = 6.0;
pub(crate) const CARET_W: f32 = 1.5;
pub(crate) const COMPOSER_MIN_H: f32 = 56.0;

// ── Menus e painel do arquivo ──
pub(crate) const MENU_W: f32 = 304.0;
pub(crate) const MENU_PAD: f32 = 12.0;
pub(crate) const MENU_ROW_H: f32 = 32.0;
pub(crate) const MENU_RADIUS: f32 = 8.0;
pub(crate) const SWATCH_H: f32 = 32.0;
pub(crate) const SWATCH_GAP: f32 = 4.0;
pub(crate) const PANEL_W: f32 = 340.0;

// ── O início (os Kanbans do projecto) ──
pub(crate) const TILE_W: f32 = 194.0;
pub(crate) const TILE_H: f32 = 96.0;
pub(crate) const TILE_GAP: f32 = 16.0;
pub(crate) const HOME_PAD: f32 = 32.0;
