//! **Os ids da §12 Sockets / Named Anchors** ([ADR-0072], spec
//! [`07_named_anchors.md`](../../../../docs/Sprite_projeto/07_named_anchors.md) §7.5).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — mesmo padrão do
//! [`super::inspector_slice`] e do [`super::inspector_sampling`].
//!
//! # LISTA + EDITOR, e não uma ficha por âncora
//!
//! A spec §7.5 desenha uma **ficha por âncora**, empilhadas. Fichas empilhadas exigem ids
//! pré-alocados por LINHA: com o cap de 64 âncoras e ~12 campos cada, seriam **768 ids** — e um
//! painel de 320 px a rolar por 64 fichas abertas não é navegável.
//!
//! Aqui é uma **lista de nomes** (um id por linha, 64) mais **um editor** do que está
//! selecionado (12 ids). 76 no total, e o gesto é o de todo editor com listas longas.
//! ⚠️ A informação que a ficha da spec mostrava de relance — o que cada âncora **é** — não se
//! perde: cada linha traz o seu tipo derivado (`Socket` · `Slice` · `Region`) ao lado do nome.
//!
//! [ADR-0072]: ../../../../docs/architecture/decisions/0072-named-anchor-unification.md

use super::*;

/// §12 — o cabeçalho colapsável. Entra em [`super::LIVE_SECTIONS`] emparelhado com o ponto.
pub const INSP_LIVE_ANCHOR_SECTION: NodeId = hash_node_id("insp_live_anchor_section");
/// §12 — ponto de cor do cabeçalho.
pub const INSP_LIVE_ANCHOR_COLOR: NodeId = hash_node_id("insp_live_anchor_color");

/// **As linhas da lista** — uma por âncora, até ao cap de `ANCHORS_MAX`.
///
/// ⚠️ **O comprimento deste array É o cap do modelo** (`ph2d_ecs::ANCHORS_MAX`), e há gate na
/// shell a prendê-los. Um array mais curto tornaria as âncoras do fim **inalcançáveis por gesto
/// nenhum** — a classe de defeito que a §9 Sampling pagou com quatro modos mudos.
pub const INSP_ANCHOR_ROW: [NodeId; 64] = build_anchor_rows();

const fn build_anchor_rows() -> [NodeId; 64] {
    // `hash_node_id` é `const`, mas não se pode formatar uma string em `const fn` — daí a
    // tabela explícita. Gerada uma vez; a ORDEM é o índice da âncora.
    [
        hash_node_id("insp_anchor_row_00"),
        hash_node_id("insp_anchor_row_01"),
        hash_node_id("insp_anchor_row_02"),
        hash_node_id("insp_anchor_row_03"),
        hash_node_id("insp_anchor_row_04"),
        hash_node_id("insp_anchor_row_05"),
        hash_node_id("insp_anchor_row_06"),
        hash_node_id("insp_anchor_row_07"),
        hash_node_id("insp_anchor_row_08"),
        hash_node_id("insp_anchor_row_09"),
        hash_node_id("insp_anchor_row_10"),
        hash_node_id("insp_anchor_row_11"),
        hash_node_id("insp_anchor_row_12"),
        hash_node_id("insp_anchor_row_13"),
        hash_node_id("insp_anchor_row_14"),
        hash_node_id("insp_anchor_row_15"),
        hash_node_id("insp_anchor_row_16"),
        hash_node_id("insp_anchor_row_17"),
        hash_node_id("insp_anchor_row_18"),
        hash_node_id("insp_anchor_row_19"),
        hash_node_id("insp_anchor_row_20"),
        hash_node_id("insp_anchor_row_21"),
        hash_node_id("insp_anchor_row_22"),
        hash_node_id("insp_anchor_row_23"),
        hash_node_id("insp_anchor_row_24"),
        hash_node_id("insp_anchor_row_25"),
        hash_node_id("insp_anchor_row_26"),
        hash_node_id("insp_anchor_row_27"),
        hash_node_id("insp_anchor_row_28"),
        hash_node_id("insp_anchor_row_29"),
        hash_node_id("insp_anchor_row_30"),
        hash_node_id("insp_anchor_row_31"),
        hash_node_id("insp_anchor_row_32"),
        hash_node_id("insp_anchor_row_33"),
        hash_node_id("insp_anchor_row_34"),
        hash_node_id("insp_anchor_row_35"),
        hash_node_id("insp_anchor_row_36"),
        hash_node_id("insp_anchor_row_37"),
        hash_node_id("insp_anchor_row_38"),
        hash_node_id("insp_anchor_row_39"),
        hash_node_id("insp_anchor_row_40"),
        hash_node_id("insp_anchor_row_41"),
        hash_node_id("insp_anchor_row_42"),
        hash_node_id("insp_anchor_row_43"),
        hash_node_id("insp_anchor_row_44"),
        hash_node_id("insp_anchor_row_45"),
        hash_node_id("insp_anchor_row_46"),
        hash_node_id("insp_anchor_row_47"),
        hash_node_id("insp_anchor_row_48"),
        hash_node_id("insp_anchor_row_49"),
        hash_node_id("insp_anchor_row_50"),
        hash_node_id("insp_anchor_row_51"),
        hash_node_id("insp_anchor_row_52"),
        hash_node_id("insp_anchor_row_53"),
        hash_node_id("insp_anchor_row_54"),
        hash_node_id("insp_anchor_row_55"),
        hash_node_id("insp_anchor_row_56"),
        hash_node_id("insp_anchor_row_57"),
        hash_node_id("insp_anchor_row_58"),
        hash_node_id("insp_anchor_row_59"),
        hash_node_id("insp_anchor_row_60"),
        hash_node_id("insp_anchor_row_61"),
        hash_node_id("insp_anchor_row_62"),
        hash_node_id("insp_anchor_row_63"),
    ]
}

/// **«+ Add Anchor»** — cria uma âncora nova com o próximo nome livre (`anchor_N`).
pub const INSP_ANCHOR_ADD: NodeId = hash_node_id("insp_anchor_add");
/// **«× Remove»** — retira a âncora selecionada.
pub const INSP_ANCHOR_REMOVE: NodeId = hash_node_id("insp_anchor_remove");

/// O nome da âncora selecionada.
pub const INSP_ANCHOR_NAME: NodeId = hash_node_id("insp_anchor_name");
/// Posição `[x, y]` da âncora selecionada, em pixels da fonte.
pub const INSP_ANCHOR_POS: [NodeId; 2] = [
    hash_node_id("insp_anchor_pos_x"),
    hash_node_id("insp_anchor_pos_y"),
];
/// Rotação da âncora selecionada, em graus.
pub const INSP_ANCHOR_ROT: NodeId = hash_node_id("insp_anchor_rot");

/// Liga a ÁREA — `bounds`. Sem ela a âncora é um Socket.
pub const INSP_ANCHOR_BOUNDS_ON: NodeId = hash_node_id("insp_anchor_bounds_on");
/// `[x, y, w, h]` da área.
pub const INSP_ANCHOR_BOUNDS: [NodeId; 4] = [
    hash_node_id("insp_anchor_bounds_x"),
    hash_node_id("insp_anchor_bounds_y"),
    hash_node_id("insp_anchor_bounds_w"),
    hash_node_id("insp_anchor_bounds_h"),
];
/// Liga o MIOLO — `center`. Só tem sentido com a área ligada.
pub const INSP_ANCHOR_CENTER_ON: NodeId = hash_node_id("insp_anchor_center_on");
/// `[x, y, w, h]` do miolo.
pub const INSP_ANCHOR_CENTER: [NodeId; 4] = [
    hash_node_id("insp_anchor_center_x"),
    hash_node_id("insp_anchor_center_y"),
    hash_node_id("insp_anchor_center_w"),
    hash_node_id("insp_anchor_center_h"),
];
