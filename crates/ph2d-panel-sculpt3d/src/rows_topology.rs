//! **A RESOLUÇÃO DA MALHA** — os knobs da seção Topology, irmãos do
//! [`super::rows`], onde ficam os do PINCEL.
//!
//! ⚠️ **O corte é o mesmo que a seção do sombreamento já fez** (`rows_shading`),
//! e pela mesma razão: uma `Section` carrega UMA fatia de rows, então mudar de
//! arquivo não mexe num pixel do que é pintado — o que muda é onde a lista
//! cresce. E as duas crescem por motivos diferentes: a de lá com cada canal do
//! pincel, esta com cada gesto que muda quantos vértices a malha tem.

use crate::rows::{Place, Row};
use crate::state::UiLevel;
use ph2d_editor_core::ids;

/// **A TOPOLOGIA** — hoje uma row só, a resolução do remesh.
///
/// ⚠️ **A FAIXA É MEDIDA, e o recurso é a memória do campo TRANSIENTE**
/// (`ph2d-sdf/tests/measure_remesh.rs`, esfera `uv(96,144)`):
///
/// | resolução | células | campo | malha de saída |
/// |---|---|---|---|
/// | 16 | 9.261 | 0,1 MB | 1.250 v |
/// | 150 | 3,7 M | 24,9 MB | 106.052 v |
/// | **512** | 138 M | **922,5 MB** | 1,23 M v |
/// | 640 | 268 M | 1791,3 MB | 1,93 M v |
/// | 768 | 462 M | **3083,4 MB** | 2,78 M v |
///
/// O teto **não foi escolhido**: o HR-13 declara **3500 MB para o app inteiro**,
/// e a 768 o campo *transiente* sozinho come **3083 MB — 88% de tudo**, para um
/// rascunho que é jogado fora no fim. A 640 já são 51%. **512 é o último degrau
/// que cabe**, a 26%.
///
/// ⚠️ E o piso é 16 porque abaixo dele a saída deixa de ser uma forma (1.250
/// vértices já é blocagem grossa), não porque algum recurso acabe.
pub static TOPOLOGY: &[Row] = &[
    Row {
        label: "panel.sculpt3d.remesh_res",
        slider: ids::SCULPT3D_REMESH_RES,
        chip: ids::SCULPT3D_REMESH_RES_NUM,
        min: 16.0,  // LITERAL-PX-OK: resolucao de voxel, nao metrica de layout
        max: 512.0, // LITERAL-PX-OK: idem -- o teto medido, ver a tabela acima
        step: 1.0,
        decimals: 0,
        get: |u| u.remesh_res,
        set: |u, v| u.remesh_res = v,
        show: |_| true,
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    // ⚠️ **A RETOPOLOGIA tem duas pistas, e elas são de espécie diferente da de
    // cima:** aquela é a resolução de um VOXEL (quantas células o campo tem),
    // estas dizem quão fina é a GRADE de saída. Partilhar um slider seria a
    // mesma pergunta a responder duas coisas.
    //
    // ⚠️ **ESTE KNOB DEIXOU DE SER UM TAMANHO ABSOLUTO, e a troca é a cura de um
    // defeito que o Enio fotografou** (2026-08-19): ele era `Quad Size`, em
    // unidades de objeto, de `0,02` a `1,00` — e **as duas pontas destruíam a
    // peça**. Abaixo do que a malha de entrada resolve a extração devolve malha
    // **vazia**; a `1,5×` a aresta de entrada ela devolve um ciclo de 352 lados
    // com **58 % do volume perdido**. Um mesmo `0,02` é destrutivo numa malha
    // grossa e conservador numa fina: *o número não era da malha*.
    //
    // Agora é uma **fração do curso** (`0` = a grade mais grossa que ainda
    // descreve a forma, `1` = a mais fina que a entrada consegue resolver), e a
    // `ph2d_quadflow::edge_for_detail` converte para o lado do quad **a partir da
    // malha**. Todo ponto do curso é legal por construção, em qualquer modelo —
    // que é a propriedade que o gate `every_point_of_the_detail_slider_is_legal`
    // afirma.
    Row {
        label: "panel.sculpt3d.quad_detail",
        slider: ids::SCULPT3D_QUAD_DETAIL,
        chip: ids::SCULPT3D_QUAD_DETAIL_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: fracao do curso, nao metrica de layout
        decimals: 2,
        get: |u| u.quad_detail,
        set: |u, v| u.quad_detail = v,
        show: |_| true,
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.quad_adapt",
        slider: ids::SCULPT3D_QUAD_ADAPT,
        chip: ids::SCULPT3D_QUAD_ADAPT_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: fracao de adaptacao, nao metrica de layout
        decimals: 2,
        get: |u| u.quad_adapt,
        set: |u, v| u.quad_adapt = v,
        show: |_| true,
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
];
