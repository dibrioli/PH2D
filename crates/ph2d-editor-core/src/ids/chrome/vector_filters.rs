//! **Os ids da seção Filters (FX raster)** — módulo irmão de [`super`] pelo teto de 700 LOC.
//!
//! O corte é por RESPONSABILIDADE, como o do `vector_contour` / `vector_textpath`: estes são os
//! controles do [`ph2d_ecs::VecFilter`] — o FX RASTER por-forma (Blur / Glow / Drop Shadow, plano
//! 24). É deliberadamente distinto da seção **Effects** (`VECTOR_SECTION_EFFECTS`, ADR-0132), que
//! é a pilha de deformadores VETORIAIS (`VecPath -> VecPath`); um filtro produz PIXELS, não
//! geometria, e colapsar os dois nomes esconderia a diferença que decide a arquitetura inteira.
//!
//! ⚠️ **Bloco APPEND-ONLY**, como os do Conector / Blend / Contour: um id é o hash de uma STRING,
//! então reordenar não quebra nada — mas renomear uma string quebra tudo o que a referencia por
//! nome, e é assim que um widget fica órfão em silêncio.

use ph2d_a11y::NodeId;

use super::super::hash_node_id;

// ── Filters: o FX raster por-forma ──────────────────────────────────────────────
// O efeito é o componente `ph2d_ecs::VecFilter` na entidade da forma; presença = tem filtro,
// ausência = forma nua. Estes ids são a única porta do PRODUTO para ele.
/// Seção **FILTERS** — a forma ganha um FX raster (Blur / Glow / Drop Shadow).
pub const VECTOR_SECTION_FILTERS: NodeId = hash_node_id("vector.section.filters");

/// **Filter: None** — remove o `VecFilter` da seleção (a forma volta nua). É o default, e o
/// único chip que APAGA em vez de armar.
pub const VECTOR_FILTER_KIND_NONE: NodeId = hash_node_id("vector.filter.kind.none");
/// **Filter: Blur** — a própria forma borrada (substitui o desenho).
pub const VECTOR_FILTER_KIND_BLUR: NodeId = hash_node_id("vector.filter.kind.blur");
/// **Filter: Glow** — a silhueta borrada e tingida ATRÁS da forma (que segue desenhada por cima).
pub const VECTOR_FILTER_KIND_GLOW: NodeId = hash_node_id("vector.filter.kind.glow");
/// **Filter: Drop Shadow** — como o Glow, mas deslocado por um offset.
pub const VECTOR_FILTER_KIND_SHADOW: NodeId = hash_node_id("vector.filter.kind.shadow");

/// **Radius** — o `stdDev` do borrão, em unidades de MUNDO (dar zoom aumenta o borrão na tela).
pub const VECTOR_FILTER_RADIUS: NodeId = hash_node_id("vector.filter.radius");
/// O campo numérico gêmeo do [`VECTOR_FILTER_RADIUS`].
pub const VECTOR_FILTER_RADIUS_NUM: NodeId = hash_node_id("vector.filter.radius.num");

/// **Offset X** — o deslocamento da Drop Shadow em X (mundo). Só no Drop Shadow.
pub const VECTOR_FILTER_OFFX: NodeId = hash_node_id("vector.filter.offx");
/// O campo numérico gêmeo do [`VECTOR_FILTER_OFFX`].
pub const VECTOR_FILTER_OFFX_NUM: NodeId = hash_node_id("vector.filter.offx.num");
/// **Offset Y** — o deslocamento da Drop Shadow em Y (mundo). Só no Drop Shadow.
pub const VECTOR_FILTER_OFFY: NodeId = hash_node_id("vector.filter.offy");
/// O campo numérico gêmeo do [`VECTOR_FILTER_OFFY`].
pub const VECTOR_FILTER_OFFY_NUM: NodeId = hash_node_id("vector.filter.offy.num");

/// **Color** — a cor do Glow / Drop Shadow (swatch, abre o picker OKLCH). O Blur a ignora.
pub const VECTOR_FILTER_COLOR: NodeId = hash_node_id("vector.filter.color");

/// **Opacity** — a opacidade do efeito (0..1).
pub const VECTOR_FILTER_OPACITY: NodeId = hash_node_id("vector.filter.opacity");
/// O campo numérico gêmeo do [`VECTOR_FILTER_OPACITY`].
pub const VECTOR_FILTER_OPACITY_NUM: NodeId = hash_node_id("vector.filter.opacity.num");
