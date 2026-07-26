//! **A lista das seções colapsáveis do painel Vector** — irmã de [`super::vector`] pelo teto de
//! 700 LOC, e o corte é por responsabilidade: os irmãos declaram IDs, este declara uma POLÍTICA
//! sobre eles (*quais cabeçalhos dobram*), e é o único ponto do módulo que referencia os três
//! blocos de seção ao mesmo tempo.

use ph2d_a11y::NodeId;

use super::vector::*;

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
    VECTOR_SECTION_VERTEX,
    VECTOR_SECTION_BOOLEAN,
    VECTOR_SECTION_EXPAND,
    VECTOR_SECTION_BLEND,
    VECTOR_SECTION_MORPH,
    VECTOR_SECTION_ENVELOPE,
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
    super::vector_textpath::VECTOR_SECTION_TEXTPATH,
    super::vector_patternpath::VECTOR_SECTION_PATTERNPATH,
    super::vector_contour::VECTOR_SECTION_CONTOUR,
    // FX raster (plano 24) — distinto de EFFECTS (deformadores vetoriais, ADR-0132).
    super::vector_filters::VECTOR_SECTION_FILTERS,
];
