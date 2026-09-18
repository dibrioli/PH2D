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

/// **A TOPOLOGIA** — quatro rows: o alvo do passe dinâmico (colado ao
/// interruptor que o arma) e os três argumentos dos botões que reconstroem a
/// malha.
///
/// ⚠️ **A FAIXA É MEDIDA, e o recurso é a memória do campo TRANSIENTE**
/// (`ph2d-sdf/tests/it/measure_remesh.rs`, esfera `uv(96,144)`):
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
    // ⭐⭐ **O ALVO DE DENSIDADE DA TOPOLOGIA DINÂMICA**, e ele mora colado ao
    // interruptor que o arma ([`Place::AfterDyntopo`]) — não no bloco do fim da
    // secção, que é dos argumentos do botão de retopologia.
    //
    // ⚠️⚠️ **Ele substitui TRÊS CHIPS com nome** (*grosso · médio · fino*), por
    // report do dono em 2026-09-14: *«porque não temos um slider neste pincel
    // para definir a densidade da malha»*. ⛔ **Os dois não coexistem** — eles
    // escrevem o mesmo número, e duas superfícies sobre um valor só divergem no
    // dia em que uma ganhar clamp e a outra não. A tecla `U` fica, a ciclar os
    // três valores com nome: é o atalho, como o `[`/`]` do raio é o da pista
    // dele.
    //
    // ⭐⭐⭐ **ELE PEDE UMA CONTAGEM, E NÃO DEPENDE DO ZOOM** — ordem do dono
    // (14/09): *«a densidade da malha deve ser independente do zoom»*. A pista
    // percorre `MIN_TRIS`..`MAX_TRIS` **geometricamente**
    // ([`ph2d_mesh::tris_for_detail`]), e o alvo de aresta sai daí contra a
    // **ÁREA DA SUPERFÍCIE** da peça — que é propriedade da forma, não da
    // tesselação nem da vista.
    //
    // ⚠️ **A faixa não é escolhida, e cada ponta tem o recurso NOMEADO:** o
    // extremo grosso é o joelho de volume medido (`~200` triângulos, o
    // `MIN_QUADS` do botão de retopologia em triângulos) e o fino é o **relógio
    // do dab** (cada passe faz um `rebuild` inteiro; a `100 000` triângulos ele
    // come `~35 %` do orçamento de `8 ms`). As duas tabelas estão nos docs das
    // constantes.
    //
    // ⛔ **A âncora era o RAIO DO PINCEL até 14/09**, e o raio é derivado do
    // raio em PIXELS através da câmera: medido, o mesmo pincel e o mesmo ponto
    // da pista davam `4,9×` de alvo diferente só por aproximar ou afastar.
    // *O pincel diz ONDE; esta pista diz QUÃO FINO.*
    Row {
        label: "panel.sculpt3d.dyn_detail",
        slider: crate::ids::SCULPT3D_DYN_DETAIL,
        chip: crate::ids::SCULPT3D_DYN_DETAIL_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: fracao do curso, nao metrica de layout
        decimals: 2,
        get: |u| u.dyn_detail,
        set: |u, v| u.dyn_detail = v,
        show: |_| true,
        level: UiLevel::Basic,
        place: Place::AfterDyntopo,
    },
    // ⭐⭐⭐ **O PENTE DE TOPOLOGIA** — quanto a malha debaixo do traço se
    // reorganiza numa GRADE alinhada com ele. Clean-room sob
    // `docs/3D/cleanroom/SPEC_pente_de_topologia.md`; a lei vive na `ph2d-rake`.
    //
    // ⚠️ **Ele é um campo do PINCEL e mora AQUI** — a única pré-condição de
    // estado dele é a topologia dinâmica ARMADA (espec §2.1: desarmada, os dois
    // lados do controlo dão a MESMA malha byte a byte), e essa caixa está uma
    // linha acima. É a mesma lei que pôs o alvo de densidade neste sítio:
    // *uma pista mora ao lado do controlo que a governa*.
    //
    // ⚠️ **Ele NÃO depende do passe de refino correr** (espec §2.2): sem refino
    // nenhum o pente continua a agir, e com efeito MAIOR — a cerca é o
    // interruptor, nunca «o passe vai partir alguma aresta».
    //
    // # ⚠️ A faixa é `0..1` e o TECTO é MEDIDO — ver [`ph2d_sculpt3d::Brush::pente`]
    //
    // ⛔ **Coincidir com a do alvo é resultado, não cópia.** O botão dele
    // SATURA acima de `0,75` (a `0°` até desce) e o nosso não satura em ponto
    // nenhum: o que acaba é a MALHA. A `1,0` o pior triângulo da faixa mede
    // `4,56°` e a `3,0` mede `0,62°` — um triângulo de seis décimos de grau não
    // tem normal utilizável, logo não tem sombra. *Todo o nosso curso faz
    // alguma coisa.*
    //
    // ⭐ E a metade de baixo **melhora** a malha (`8,21°` a `0,25` contra
    // `7,86°` desligado): o pente desfaz as lascas que o próprio refino deixa.
    Row {
        label: "panel.sculpt3d.pente",
        slider: crate::ids::SCULPT3D_PENTE,
        chip: crate::ids::SCULPT3D_PENTE_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: fracao do curso, nao metrica de layout
        decimals: 2,
        get: |u| u.brush.pente,
        set: |u, v| u.brush.pente = v,
        show: crate::rows::penteia,
        level: UiLevel::Basic,
        place: Place::AfterDyntopo,
    },
    Row {
        label: "panel.sculpt3d.remesh_res",
        slider: crate::ids::SCULPT3D_REMESH_RES,
        chip: crate::ids::SCULPT3D_REMESH_RES_NUM,
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
        slider: crate::ids::SCULPT3D_QUAD_DETAIL,
        chip: crate::ids::SCULPT3D_QUAD_DETAIL_NUM,
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
        slider: crate::ids::SCULPT3D_QUAD_ADAPT,
        chip: crate::ids::SCULPT3D_QUAD_ADAPT_NUM,
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
