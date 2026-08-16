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
pub static TOPOLOGY: &[Row] = &[Row {
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
}];
