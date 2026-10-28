//! As medidas da tabela, em px LÓGICOS (as da vista Tabela do Trello Premium sobre a escala da
//! maquete: corpo `14`, pequeno `12`, linha de `40`). As cores vêm dos tokens `kanban-*`.

/// A margem entre a área da vista e o cartão branco da tabela.
pub(crate) const PAD: f32 = 16.0;
pub(crate) const RADIUS: f32 = 12.0;
pub(crate) const HEADER_H: f32 = 36.0;
pub(crate) const ROW_H: f32 = 40.0;
/// O recuo do texto dentro de uma célula.
pub(crate) const CELL_PAD: f32 = 12.0;
pub(crate) const BODY_PX: f32 = 14.0;
pub(crate) const SMALL_PX: f32 = 12.0;
pub(crate) const RULE_W: f32 = 1.0;
/// O recuo vertical do filete entre colunas no cabeçalho.
pub(crate) const RULE_INSET: f32 = 8.0;
/// O quadrado da cor da lista na coluna List, e quanto ele se aproxima do nome.
pub(crate) const DOT: f32 = 10.0;
pub(crate) const DOT_GAP: f32 = 4.0;

// ── As larguras das colunas ──
pub(crate) const CARD_W: f32 = 320.0;
pub(crate) const LIST_W: f32 = 168.0;
pub(crate) const LABELS_W: f32 = 232.0;
pub(crate) const MEMBERS_W: f32 = 140.0;
pub(crate) const DUE_W: f32 = 184.0;
pub(crate) const FIELD_W: f32 = 150.0;

// ── As peças de uma célula ──
pub(crate) const PILL_H: f32 = 24.0;
pub(crate) const PILL_RADIUS: f32 = 4.0;
pub(crate) const PILL_PAD_X: f32 = 8.0;
pub(crate) const PILL_GAP: f32 = 4.0;
pub(crate) const AVATAR: f32 = 26.0;
pub(crate) const AVATAR_PX: f32 = 11.0;
pub(crate) const AVATAR_OVERLAP: f32 = 4.0;
pub(crate) const AVATAR_RING: f32 = 2.0;
/// A seta da ordenação no cabeçalho (um triângulo).
pub(crate) const SORT_ARROW: f32 = 4.0;
pub(crate) const CHECK: f32 = 16.0;
pub(crate) const CHECK_RADIUS: f32 = 3.0;

// ── Os popovers ──
pub(crate) const POP_W: f32 = 304.0;
pub(crate) const POP_PAD: f32 = 8.0;
pub(crate) const POP_ROW_H: f32 = 36.0;
pub(crate) const POP_RADIUS: f32 = 8.0;
pub(crate) const POP_GAP: f32 = 4.0;
/// A roda: uma linha de roda (`16` px do hero) rola `48` px — a do quadro.
pub(crate) const WHEEL_STEP: f32 = 48.0;
pub(crate) const WHEEL_LINE_PX: f32 = 16.0;
