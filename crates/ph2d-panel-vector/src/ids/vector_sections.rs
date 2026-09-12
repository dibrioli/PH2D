//! **A lista das seções colapsáveis do painel Vector** — irmã de [`super::vector`] pelo teto de
//! 700 LOC, e o corte é por responsabilidade: os irmãos declaram IDs, este declara uma POLÍTICA
//! sobre eles (*quais cabeçalhos dobram*), e é o único ponto do módulo que referencia os três
//! blocos de seção ao mesmo tempo.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/vector_sections.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 43 crates recompilam deixou de os
//! carregar.

use super::VECTOR_SECTION_APPEARANCE;
use ph2d_a11y::NodeId;
use ph2d_tool_vector::ids::{
    VECTOR_SECTION_ALIGN, VECTOR_SECTION_ARRANGE, VECTOR_SECTION_AXES, VECTOR_SECTION_BLEND,
    VECTOR_SECTION_BOOLEAN, VECTOR_SECTION_CONNECTOR, VECTOR_SECTION_EFFECTS,
    VECTOR_SECTION_ENVELOPE, VECTOR_SECTION_EXPAND, VECTOR_SECTION_FILL, VECTOR_SECTION_FILL_TYPE,
    VECTOR_SECTION_FONT, VECTOR_SECTION_MORPH, VECTOR_SECTION_PARAGRAPH, VECTOR_SECTION_PATH,
    VECTOR_SECTION_PENCIL, VECTOR_SECTION_SHAPE, VECTOR_SECTION_SHAPE_PARAMS, VECTOR_SECTION_SNAP,
    VECTOR_SECTION_STROKE, VECTOR_SECTION_TEXT, VECTOR_SECTION_TOOL, VECTOR_SECTION_TRANSFORM,
    VECTOR_SECTION_VERTEX,
};

/// Todos os cabeçalhos de seção do painel Vector — o `populate` os marca como
/// colapsáveis por esta lista (uma seção nova entra aqui e ganha o collapse de graça;
/// esquecer a marca faz o header virar um título MORTO, que não dobra).
///
/// ⚠️ **É lista APPEND-ONLY partilhada:** ela é fundida contra a `main` de hoje, então uma linha
/// paralela só ACRESCENTA — tirar uma seção daqui é trabalho de integração.
///
/// ⚠️ Esquecer a entrada **não dá erro em lado nenhum**: o `paint` regista o hit-rect e o
/// `dispatch` consulta `is_collapsible_section` antes de disparar o toggle, então o cabeçalho fica
/// pintado, clicável e MORTO. Foi o que aconteceu com o Text on Path e o Pattern on Path, que
/// chegaram à `main` fora desta lista (2026-07-23) e cujo chevron não dobrava. O gate
/// `every_painted_section_header_is_collapsible` (crate do painel) varre as chamadas de
/// `section_header` no fonte e cobra a correspondência — a lista escrita à mão num gate driftaria
/// da tela exatamente como esta driftou.
pub const VECTOR_SECTIONS: &[NodeId] = &[
    VECTOR_SECTION_TOOL,
    VECTOR_SECTION_SHAPE,
    VECTOR_SECTION_SHAPE_PARAMS,
    VECTOR_SECTION_STROKE,
    VECTOR_SECTION_FILL,
    VECTOR_SECTION_FILL_TYPE,
    VECTOR_SECTION_SNAP,
    VECTOR_SECTION_TRANSFORM,
    // ⭐ A APARÊNCIA do objecto (estudo 42 item 2) — logo a seguir ao Transform, que é onde ela é
    // pintada: as duas descrevem a forma selecionada como um todo.
    VECTOR_SECTION_APPEARANCE,
    VECTOR_SECTION_VERTEX,
    VECTOR_SECTION_BOOLEAN,
    VECTOR_SECTION_EXPAND,
    VECTOR_SECTION_BLEND,
    VECTOR_SECTION_MORPH,
    VECTOR_SECTION_ENVELOPE,
    // ⭐ O ESQUELETO (estudo 42 item 5) — ao lado do Envelope de propósito: os dois DEFORMAM formas
    // que já existem, e lidos juntos ensinam a diferença (uma gaiola contra uma cadeia de ossos).
    VECTOR_SECTION_EFFECTS,
    VECTOR_SECTION_ALIGN,
    VECTOR_SECTION_ARRANGE,
    VECTOR_SECTION_PATH,
    VECTOR_SECTION_TEXT,
    VECTOR_SECTION_FONT,
    VECTOR_SECTION_PARAGRAPH,
    VECTOR_SECTION_AXES,
    VECTOR_SECTION_CONNECTOR,
    // Os três que faltavam / o que chegou agora. Os dois primeiros são DÍVIDA da integração de
    // 2026-07-23 (ver o ⚠️ acima); o terceiro é a seção nova do Contour.
    super::VECTOR_SECTION_TEXTPATH,
    super::VECTOR_SECTION_PATTERNPATH,
    // A secção do TEXTURE PATTERN (plano 33) — a TINTA, não o motivo-sobre-guia acima.
    super::VECTOR_SECTION_TEXPAT,
    // ⭐ E a irmã do TRAÇO (plano 35, wave F): *"cada seção deve ter seus ajustes próprios"*.
    super::VECTOR_SECTION_TEXPAT_STROKE,
    // ⭐ E a do PINCEL (plano 36, W4) — knobs PRÓPRIOS: avanço e escala relativa, não reticulado.
    super::VECTOR_SECTION_BRUSH,
    super::VECTOR_SECTION_CONTOUR,
    // FX raster (plano 24) — distinto de EFFECTS (deformadores vetoriais, ADR-0132).
    super::VECTOR_SECTION_FILTERS,
    // O LÁPIS (plano 25 W1): Fidelity + Stabilizer.
    VECTOR_SECTION_PENCIL,
    // A SIMETRIA de desenho (plano 25 §9 W6.3): um MODO, não um efeito.
    ph2d_tool_vector::ids::VECTOR_SECTION_SYMMETRY,
    // A MOLDURA (plano UI/UX W0): o contêiner.
    ph2d_tool_vector::ids::VECTOR_SECTION_FRAME,
    // O RECORTE (2026-08-21): irmã da Frame, mas oferecida a QUALQUER forma fechada.
    ph2d_tool_vector::ids::VECTOR_SECTION_CLIP,
    // O AUTO LAYOUT (plano UI/UX W2, ADR-0153): a moldura que EMPILHA.
    super::VECTOR_SECTION_LAYOUT,
    // AS ÂNCORAS (plano UI/UX W3): a regra do filho que NÃO flui.
    super::VECTOR_SECTION_ANCHORS,
    // OS COMPONENTES (plano UI/UX W5): o mestre e a instância.
    super::VECTOR_SECTION_COMPONENT,
    // A PELE POR-WIDGET (plano UI/UX W6.2): que controle do catálogo a forma veste.
    super::VECTOR_SECTION_WIDGET,
    // OS ESTADOS de UI (plano UI/UX W7): as poses e o tween entre elas.
    super::VECTOR_SECTION_STATES,
    // ⭐ OS ESTADOS do MORPH (plano 32 W7): a máquina que decide em que forma o objecto está.
    // ⛔ Seção PRÓPRIA, e não uma sub-lista da de cima — ver o módulo `vector_morph`.
    super::VECTOR_SECTION_MORPH_STATES,
];
