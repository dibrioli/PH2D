//! **Os ids do card LINE** — o `Style: Solid`, o dropdown `Type` e as rows de cada tipo de linha
//! procedural (plano 38).
//!
//! Módulo irmão de [`super::painter`], cortado por RESPONSABILIDADE quando o pai cruzou o teto de
//! LOC: lá ficam os ids do Painter em geral, aqui os de UMA seção que cresce a cada tipo novo. Eles
//! são re-exportados pelo pai, então **nenhum chamador muda de caminho**.
//!
//! ⚠️ **Todos por HASH DE STRING**, então acrescentar um tipo não move nenhum contador — a mesma
//! razão pela qual o `node_id_collisions` os cobre sem uma lista à mão.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/painter_line.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// **Style: Solid** — o checkbox do card **Line** (acima do Composite Brush): desmarcado o traço é
/// uma LINHA, marcado ele é a forma CERCADA pelo gesto, preenchida. `Click` → alterna.
///
/// ⚠️ A pergunta *"este tipo de linha tem caminho fechado para preencher?"* é do
/// `LineKind::honours_style()`, não deste id — hoje ela devolve `true` para todos, e o dia em que
/// nascer um tipo sem caminho-base é o dia em que ela deixa de ser trivial.
pub const PAINTER_LINE_SOLID: NodeId = hash_node_id("painter_line.solid");

/// **Type** — o dropdown do card **Line**: qual LEI procedural decora o traço
/// (`ph2d_painter_brush::line_kind::LineKind`; `0` = None, o default, `1` = Speed, `2` = Sketchy).
/// `SelectOption` → `set_line_kind`.
pub const PAINTER_LINE_TYPE: NodeId = hash_node_id("painter_line.type");

/// **Reach** — o raio da vizinhança do Sketchy, em DIÂMETROS de pincel (`0..=SKETCHY_REACH_MAX`).
/// `SetValue` → `set_sketchy_reach_norm`.
pub const PAINTER_LINE_SKETCHY_REACH: NodeId = hash_node_id("painter_line.sketchy_reach");

/// **Density** — a fração dos pares dentro do alcance que viram fio. É o ORÇAMENTO do tipo, não um
/// enfeite (`ph2d_painter_brush::line_kind::SKETCHY_DENSITY_MAX`). `SetValue` → `set_sketchy_density_norm`.
pub const PAINTER_LINE_SKETCHY_DENSITY: NodeId = hash_node_id("painter_line.sketchy_density");

/// **Line Width** — a espessura de UM fio, em pixels. `SetValue` → `set_sketchy_width_norm`.
pub const PAINTER_LINE_SKETCHY_WIDTH: NodeId = hash_node_id("painter_line.sketchy_width");

/// **Opacity** — a opacidade de UM fio; a do cruzamento sai do acúmulo. `SetValue` → `set_sketchy_opacity`.
pub const PAINTER_LINE_SKETCHY_OPACITY: NodeId = hash_node_id("painter_line.thread_opacity");

/// **Magnetify** — ligado, o traço costura DOIS trechos que se aproximaram (ainda que a um arco
/// enorme um do outro); desligado, só a porção ATIVA do percurso. `Click` → alterna.
pub const PAINTER_LINE_SKETCHY_MAGNETIFY: NodeId = hash_node_id("painter_line.sketchy_magnetify");

/// **History** — a janela do Wire, em DIÂMETROS de ARCO percorrido. `SetValue` → `set_wire_history_norm`.
pub const PAINTER_LINE_WIRE_HISTORY: NodeId = hash_node_id("painter_line.wire_history");

/// **Connection Line** — o traço em si é pintado, ou sobra só o arame? `Click` → alterna.
pub const PAINTER_LINE_WIRE_CONNECTION: NodeId = hash_node_id("painter_line.wire_connection");

/// **Weight** — quanto a FITA atrasa, em fração do `RIBBON_LAG_MAX_S`. É um TEMPO, então a mesma
/// escolha atrasa mais num gesto rápido — que é o que *pesar* significa. `SetValue` →
/// `set_ribbon_weight_norm`.
pub const PAINTER_LINE_RIBBON_WEIGHT: NodeId = hash_node_id("painter_line.ribbon_weight");

/// **Friction** — o amortecimento `ζ` da fita: baixo chicoteia, alto chega devagar sem ultrapassar.
/// `SetValue` → `set_ribbon_friction_norm`.
pub const PAINTER_LINE_RIBBON_FRICTION: NodeId = hash_node_id("painter_line.ribbon_friction");

/// **Gravity** — o peso que faz a fita PENDER. `SetValue` → `set_ribbon_gravity_norm`.
pub const PAINTER_LINE_RIBBON_GRAVITY: NodeId = hash_node_id("painter_line.ribbon_gravity");

/// **Rungs** — a densidade das TRAVESSAS que fazem da fita uma FAIXA. `0` degenera na linha
/// atrasada sozinha (o pincel de arrasto). `SetValue` → `set_ribbon_rungs_norm`.
pub const PAINTER_LINE_RIBBON_RUNGS: NodeId = hash_node_id("painter_line.ribbon_rungs");

/// Card Line / **Rough**: a amplitude do desvio CURTO (o `roughness` do `rough.js`).
pub const PAINTER_LINE_ROUGH_AMOUNT: NodeId = hash_node_id("painter_line.rough_amount");

/// Card Line / **Rough**: a amplitude do ARQUEAMENTO longo (o `bowing`).
pub const PAINTER_LINE_ROUGH_BOWING: NodeId = hash_node_id("painter_line.rough_bowing");

/// Card Line / **Rough**: quantas caminhadas o traço deixa (`2` = o contorno duplo do Excalidraw).
pub const PAINTER_LINE_ROUGH_PASSES: NodeId = hash_node_id("painter_line.rough_passes");

// ── Os NÚMEROS EDITÁVEIS das barras do card Line (2026-09-16) ──────────────────────────────────
//
// ⭐ Cada barra do cartão ganha o chip que todo slider de pincel já tinha desde a ordem do dono de
// 2026-06-26 (*«todos usam o slider-with-chip canónico»*) — e o cartão deixa de ser `rótulo | trilho
// nu | readout`, a forma que a spec §2 recusa para um valor com fracção. O chip espelha a edição
// dele como `ValueChanged` do SLIDER (`WidgetStore::link_slider_number_mapped`), logo o
// encaminhamento para a ferramenta é o mesmo de sempre e nenhum `PanelEvent` novo nasce.

/// Chip de **Reach** (Sketchy).
pub const PAINTER_LINE_SKETCHY_REACH_CHIP: NodeId = hash_node_id("painter_line.sketchy_reach_chip");
/// Chip de **Density** (Sketchy), em PORCENTAGEM.
pub const PAINTER_LINE_SKETCHY_DENSITY_CHIP: NodeId =
    hash_node_id("painter_line.sketchy_density_chip");
/// Chip de **Line Width** (a tinta de fio partilhada).
pub const PAINTER_LINE_SKETCHY_WIDTH_CHIP: NodeId = hash_node_id("painter_line.sketchy_width_chip");
/// Chip de **Opacity** (a tinta de fio partilhada).
pub const PAINTER_LINE_SKETCHY_OPACITY_CHIP: NodeId =
    hash_node_id("painter_line.thread_opacity_chip");
/// Chip de **History** (Wire).
pub const PAINTER_LINE_WIRE_HISTORY_CHIP: NodeId = hash_node_id("painter_line.wire_history_chip");
/// Chip de **Weight** (Ribbon).
pub const PAINTER_LINE_RIBBON_WEIGHT_CHIP: NodeId = hash_node_id("painter_line.ribbon_weight_chip");
/// Chip de **Friction** (Ribbon).
pub const PAINTER_LINE_RIBBON_FRICTION_CHIP: NodeId =
    hash_node_id("painter_line.ribbon_friction_chip");
/// Chip de **Gravity** (Ribbon).
pub const PAINTER_LINE_RIBBON_GRAVITY_CHIP: NodeId =
    hash_node_id("painter_line.ribbon_gravity_chip");
/// Chip de **Rungs** (Ribbon).
pub const PAINTER_LINE_RIBBON_RUNGS_CHIP: NodeId = hash_node_id("painter_line.ribbon_rungs_chip");
/// Chip de **Roughness** (Rough), em décimos de diâmetro.
pub const PAINTER_LINE_ROUGH_AMOUNT_CHIP: NodeId = hash_node_id("painter_line.rough_amount_chip");
/// Chip de **Bowing** (Rough), em décimos de diâmetro.
pub const PAINTER_LINE_ROUGH_BOWING_CHIP: NodeId = hash_node_id("painter_line.rough_bowing_chip");
/// Chip de **Passes** (Rough) — um INTEIRO `1..=ROUGH_PASSES_MAX`.
pub const PAINTER_LINE_ROUGH_PASSES_CHIP: NodeId = hash_node_id("painter_line.rough_passes_chip");
