//! As medidas do painel, em px LÓGICOS: as da tabela e do quadro (corpo `14`, pequeno `12`) e as
//! marcas da skill de dataviz (barra ≤ `24` com ponta de `4`, intervalo de `2` na cor da superfície,
//! filete de `1`, alvo de toque ≥ `24`). As cores vêm dos tokens `kanban-*`.

/// A margem da vista e o intervalo entre os cartões.
pub(crate) const PAD: f32 = 16.0;
pub(crate) const GAP: f32 = 16.0;
pub(crate) const RADIUS: f32 = 12.0;
pub(crate) const CARD_PAD: f32 = 20.0;

// ── A fila de números (os stat tiles) ──
pub(crate) const TILE_H: f32 = 104.0;
pub(crate) const TILE_VALUE_PX: f32 = 32.0;
pub(crate) const TILE_LABEL_PX: f32 = 13.0;
pub(crate) const TILE_MARK: f32 = 8.0;
pub(crate) const TILE_MARK_GAP: f32 = 8.0;

// ── O cabeçalho de um gráfico ──
pub(crate) const TITLE_PX: f32 = 16.0;
pub(crate) const HEAD_H: f32 = 32.0;
pub(crate) const HEAD_GAP: f32 = 12.0;
pub(crate) const SEG_H: f32 = 28.0;
pub(crate) const SEG_PAD_X: f32 = 12.0;
pub(crate) const SEG_RADIUS: f32 = 6.0;
pub(crate) const SEG_PX: f32 = 13.0;

// ── As barras (horizontais: nomes longos à esquerda, valor na ponta) ──
pub(crate) const BODY_PX: f32 = 14.0;
pub(crate) const SMALL_PX: f32 = 12.0;
pub(crate) const BAR_MAX: f32 = 24.0;
pub(crate) const BAR_MIN: f32 = 12.0;
/// O ar entre duas barras (a barra nunca enche a faixa).
pub(crate) const BAR_AIR: f32 = 10.0;
pub(crate) const BAR_END_RADIUS: f32 = 4.0;
/// A coluna dos nomes: a do mais largo, entre estes dois.
pub(crate) const NAME_MIN_W: f32 = 72.0;
pub(crate) const NAME_MAX_W: f32 = 200.0;
pub(crate) const NAME_GAP: f32 = 12.0;
pub(crate) const VALUE_GAP: f32 = 8.0;
pub(crate) const AXIS_W: f32 = 1.0;
pub(crate) const SWATCH: f32 = 10.0;
pub(crate) const SWATCH_GAP: f32 = 6.0;
pub(crate) const AVATAR: f32 = 20.0;
pub(crate) const AVATAR_PX: f32 = 9.0;

// ── O anel ──
/// A espessura do anel em relação ao raio de fora.
pub(crate) const RING_RATIO: f32 = 0.34;
/// O intervalo entre fatias, em px no arco de fora (a cor da superfície).
pub(crate) const RING_GAP: f32 = 2.0;
/// O que a fatia sob o rato cresce.
pub(crate) const RING_LIFT: f32 = 4.0;
/// Os px de arco por segmento ao aplanar o arco.
pub(crate) const ARC_STEP: f32 = 3.0;
pub(crate) const RING_TOTAL_PX: f32 = 28.0;
pub(crate) const LEGEND_ROW_H: f32 = 28.0;
pub(crate) const LEGEND_GAP: f32 = 24.0;
pub(crate) const LEGEND_MIN_W: f32 = 180.0;
/// A parte da largura que a legenda do anel leva.
pub(crate) const LEGEND_SHARE: f32 = 0.45;
/// O total no meio do anel sobe esta fração do seu corpo acima do centro; a palavra desce esta.
pub(crate) const RING_TOTAL_RISE: f32 = 0.75;
pub(crate) const RING_SUB_DROP: f32 = 0.35;

// ── A dica ──
pub(crate) const TIP_PAD: f32 = 10.0;
pub(crate) const TIP_RADIUS: f32 = 8.0;
pub(crate) const TIP_VALUE_PX: f32 = 16.0;
pub(crate) const TIP_OFFSET: f32 = 14.0;
pub(crate) const TIP_KEY_W: f32 = 12.0;
pub(crate) const TIP_KEY_H: f32 = 3.0;
/// A altura de uma linha de texto em relação ao corpo.
pub(crate) const LINE_RATIO: f32 = 1.4;
/// A chave da dica fica a esta fração do corpo do valor (o meio da linha dele).
pub(crate) const TIP_KEY_RISE: f32 = 0.7;
pub(crate) const TIP_SHADOW_DY: f32 = 2.0;
