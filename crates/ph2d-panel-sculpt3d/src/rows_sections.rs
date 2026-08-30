//! **QUAIS SEÇÕES EXISTEM, E QUE CABEÇALHOS ELAS TÊM** — irmão (`#[path]`) do
//! [`super`], que responde *quais ROWS existem*.
//!
//! ⚠️ **O corte é de RESPONSABILIDADE, e não de tamanho.** Lá mora a tabela dos
//! knobs — uma entrada por controle contínuo, que cresce a cada wave. Aqui mora
//! a camada acima dela: *que seções o painel tem*, *quais delas têm knobs*, e
//! **a porta única dos cabeçalhos dobráveis**, que é uma pergunta com dois
//! consumidores próprios (`populate` registra, `event` despacha a dobra) e
//! nenhuma relação com o valor de uma row.
//!
//! ⚠️ Ele nasceu em 2026-08-30 quando o `rows.rs` cruzou o teto de 600 LOC do
//! `architecture_panel_loc_cap` ao ganhar a porta única dos cabeçalhos —
//! **curado por corte, nunca subindo o teto nem entrando num allowlist**, que é
//! a regra que esta casa já pagou em quatro painéis.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;

use super::{BRUSH, Row, Section, shading, topology};

/// Toda seção que tem rows de slider, em ordem de pintura.
///
/// ⚠️ **Nem toda seção do painel está aqui** — Tool, Symmetry, Scene e Bake são
/// botões e rádios, não knobs contínuos, e forçá-las nesta tabela pediria uma
/// `Row` que soubesse ser um botão. Elas são pintadas pelo `paint/body.rs` e
/// pelo `paint/tool.rs`, que são quem conhece a ordem completa; o CABEÇALHO
/// delas está na [`BUTTON_SECTIONS`] logo abaixo.
///
/// ⚠️ **A Topology ENTROU quando ganhou o primeiro knob contínuo** (a resolução
/// do remesh), e o que a traz para cá não é a pintura — ela continua sendo
/// desenhada à mão, porque o resto dela são botões — e sim as outras três listas:
/// `populate`, `event` e a varredura de costura percorrem esta tabela, então uma
/// row que mora nela nasce registrada, viva sob o mouse e varrida. Uma row
/// pintada à mão FORA daqui seria o controle morto que esta casa varre a cada
/// wave.
pub static SECTIONS: &[Section] = &[
    Section {
        id: ids::SCULPT3D_SEC_BRUSH,
        title: "panel.sculpt3d.section.brush",
        rows: BRUSH,
    },
    Section {
        id: ids::SCULPT3D_SEC_SHADING,
        title: "panel.sculpt3d.section.shading",
        rows: shading::SHADING,
    },
    Section {
        id: ids::SCULPT3D_SEC_TOPOLOGY,
        title: "panel.sculpt3d.section.topology",
        rows: topology::TOPOLOGY,
    },
];

/// **AS SEÇÕES QUE NÃO TÊM UMA ÚNICA ROW CONTÍNUA** — Tool, Symmetry, Scene e
/// Bake. Elas são pintadas à mão, mas o CABEÇALHO delas é um controle como
/// qualquer outro: ele dobra.
///
/// ⚠️ **Ela existe porque a mesma lista estava escrita DUAS vezes e as duas
/// discordavam** (medido 2026-08-30): o `populate.rs` registrava os QUATRO
/// cabeçalhos e o `is_section_header` do `event.rs` comparava só TRÊS — o
/// `SCULPT3D_SEC_BAKE` pintava o chevron, era hit-indexado, era focável, e o
/// clique caía no `_ => false`. A dobra era **pintada e não acontecia**.
///
/// ⚠️ **É a MESMA forma do `PHYSICS_SEC_LAYERS`**, que a caça de 2026-08-30
/// registrou na catraca do `the_painted_control_reaches_a_consumer`: *duas
/// listas escritas à mão sobre a mesma pergunta, e a que o artista toca é a que
/// envelhece*. Derivando, um cabeçalho novo nasce registrado E dobrável pelo
/// mesmo commit que o faz existir.
pub static BUTTON_SECTIONS: &[NodeId] = &[
    ids::SCULPT3D_SEC_TOOL,
    ids::SCULPT3D_SEC_SYMMETRY,
    ids::SCULPT3D_SEC_SCENE,
    ids::SCULPT3D_SEC_BAKE,
];

/// **TODO cabeçalho dobrável deste painel** — a porta ÚNICA que o `populate`
/// (para registrar) e o `event` (para despachar a dobra) percorrem, e que o
/// gate `every_painted_section_header_folds_and_never_touches_the_scene` varre.
pub fn section_headers() -> impl Iterator<Item = NodeId> {
    SECTIONS
        .iter()
        .map(|s| s.id)
        .chain(BUTTON_SECTIONS.iter().copied())
}

/// Toda row, achatada — o que `populate`, `event` e a varredura de costura
/// percorrem.
pub fn rows() -> impl Iterator<Item = &'static Row> {
    SECTIONS.iter().flat_map(|s| s.rows.iter())
}

/// A row a que um id pertence, se alguma (a pista ou o chip dela).
pub fn row_for(id: NodeId) -> Option<&'static Row> {
    rows().find(|r| r.slider == id || r.chip == id)
}
