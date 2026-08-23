//! **Os ids da §11 Animation** (spec
//! [`08_animation_inline.md`](../../../../docs/Sprite_projeto/08_animation_inline.md) §8.7).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — mesmo padrão do
//! [`super::inspector_anchor`] e do [`super::inspector_slice`].
//!
//! # ⚠️ A linha da lista É a animação ATUAL — e isso difere da §12 de propósito
//!
//! Na §12 Sockets, clicar numa linha muda **só** a ficha aberta e **não** vai ao barramento: qual
//! âncora se edita é um facto da UI. Aqui é o contrário, e a diferença é do domínio: numa
//! biblioteca de animações, *a que se está a ver* e *a que toca* serem a mesma coisa é o que o
//! artista espera (é o que o `AnimationPlayer` do Godot faz), e separá-las obrigaria a um segundo
//! controlo — um seletor com as mesmas entradas da lista logo abaixo dele.
//!
//! ⇒ Clicar numa linha **é** uma edição da cena (escreve `SpriteAnimator::current`), e por isso
//! entra no undo. Isto poupa 65 ids e a máquina de popover de um seletor que duplicaria a lista.
//!
//! **A POSIÇÃO NO ARRAY É A TAG** nos segmentados de direção e de loop — o despacho deriva-a de
//! `position(|&o| o == id)`. ⛔ Nunca reordene esses arrays.

use super::*;

/// §11 Animation — o cabeçalho colapsável. Entra em [`super::LIVE_SECTIONS`] com o ponto de cor.
pub const INSP_LIVE_ANIM_SECTION: NodeId = hash_node_id("insp_live_anim_section");
/// §11 Animation — ponto de cor do cabeçalho.
pub const INSP_LIVE_ANIM_COLOR: NodeId = hash_node_id("insp_live_anim_color");

/// **As linhas da lista** — uma por animação, até ao cap de `ph2d_ecs::ANIM_TAGS_MAX`.
///
/// ⚠️ O comprimento deste array **é** o cap do modelo, e há gate na shell a prendê-los. Foi para
/// o alcançar que o cap desceu de 256 (o número da spec) para 64 — *um modelo que aceita o que o
/// painel não mostra produz estado inalcançável por gesto nenhum*.
pub const INSP_ANIM_ROW: [NodeId; 64] = build_anim_rows();

const fn build_anim_rows() -> [NodeId; 64] {
    // `hash_node_id` é `const`, mas não se pode formatar uma string em `const fn` — daí a tabela
    // explícita, como na §12. A ORDEM é o índice da animação.
    [
        hash_node_id("insp_anim_row_00"),
        hash_node_id("insp_anim_row_01"),
        hash_node_id("insp_anim_row_02"),
        hash_node_id("insp_anim_row_03"),
        hash_node_id("insp_anim_row_04"),
        hash_node_id("insp_anim_row_05"),
        hash_node_id("insp_anim_row_06"),
        hash_node_id("insp_anim_row_07"),
        hash_node_id("insp_anim_row_08"),
        hash_node_id("insp_anim_row_09"),
        hash_node_id("insp_anim_row_10"),
        hash_node_id("insp_anim_row_11"),
        hash_node_id("insp_anim_row_12"),
        hash_node_id("insp_anim_row_13"),
        hash_node_id("insp_anim_row_14"),
        hash_node_id("insp_anim_row_15"),
        hash_node_id("insp_anim_row_16"),
        hash_node_id("insp_anim_row_17"),
        hash_node_id("insp_anim_row_18"),
        hash_node_id("insp_anim_row_19"),
        hash_node_id("insp_anim_row_20"),
        hash_node_id("insp_anim_row_21"),
        hash_node_id("insp_anim_row_22"),
        hash_node_id("insp_anim_row_23"),
        hash_node_id("insp_anim_row_24"),
        hash_node_id("insp_anim_row_25"),
        hash_node_id("insp_anim_row_26"),
        hash_node_id("insp_anim_row_27"),
        hash_node_id("insp_anim_row_28"),
        hash_node_id("insp_anim_row_29"),
        hash_node_id("insp_anim_row_30"),
        hash_node_id("insp_anim_row_31"),
        hash_node_id("insp_anim_row_32"),
        hash_node_id("insp_anim_row_33"),
        hash_node_id("insp_anim_row_34"),
        hash_node_id("insp_anim_row_35"),
        hash_node_id("insp_anim_row_36"),
        hash_node_id("insp_anim_row_37"),
        hash_node_id("insp_anim_row_38"),
        hash_node_id("insp_anim_row_39"),
        hash_node_id("insp_anim_row_40"),
        hash_node_id("insp_anim_row_41"),
        hash_node_id("insp_anim_row_42"),
        hash_node_id("insp_anim_row_43"),
        hash_node_id("insp_anim_row_44"),
        hash_node_id("insp_anim_row_45"),
        hash_node_id("insp_anim_row_46"),
        hash_node_id("insp_anim_row_47"),
        hash_node_id("insp_anim_row_48"),
        hash_node_id("insp_anim_row_49"),
        hash_node_id("insp_anim_row_50"),
        hash_node_id("insp_anim_row_51"),
        hash_node_id("insp_anim_row_52"),
        hash_node_id("insp_anim_row_53"),
        hash_node_id("insp_anim_row_54"),
        hash_node_id("insp_anim_row_55"),
        hash_node_id("insp_anim_row_56"),
        hash_node_id("insp_anim_row_57"),
        hash_node_id("insp_anim_row_58"),
        hash_node_id("insp_anim_row_59"),
        hash_node_id("insp_anim_row_60"),
        hash_node_id("insp_anim_row_61"),
        hash_node_id("insp_anim_row_62"),
        hash_node_id("insp_anim_row_63"),
    ]
}

/// **«+ Add Animation»** — cria uma tag nova com o próximo nome livre (`anim_N`).
pub const INSP_ANIM_ADD: NodeId = hash_node_id("insp_anim_add");
/// **«× Remove»** — retira a animação selecionada.
pub const INSP_ANIM_REMOVE: NodeId = hash_node_id("insp_anim_remove");

/// O nome da animação selecionada.
pub const INSP_ANIM_NAME: NodeId = hash_node_id("insp_anim_name");
/// Primeira célula do intervalo, **inclusive**.
pub const INSP_ANIM_FROM: NodeId = hash_node_id("insp_anim_from");
/// Última célula, **inclusive**.
pub const INSP_ANIM_TO: NodeId = hash_node_id("insp_anim_to");
/// Duração de cada frame desta animação, em ms. ⚠️ Absoluta, nunca um multiplicador de FPS.
pub const INSP_ANIM_FRAME_MS: NodeId = hash_node_id("insp_anim_frame_ms");
/// Pausa no último frame do ciclo, em ms.
pub const INSP_ANIM_HOLD_MS: NodeId = hash_node_id("insp_anim_hold_ms");
/// Pausa entre ciclos, em ms.
pub const INSP_ANIM_DELAY_MS: NodeId = hash_node_id("insp_anim_delay_ms");
/// Quantos ciclos tocar. `0` = para sempre — ver o rótulo do campo.
pub const INSP_ANIM_REPEAT: NodeId = hash_node_id("insp_anim_repeat");

/// A direção da animação selecionada — a ordem é a de `ph2d_ecs::AnimDirection::ALL`.
pub const INSP_ANIM_DIR: [NodeId; 4] = [
    hash_node_id("insp_anim_dir_fwd"),
    hash_node_id("insp_anim_dir_rev"),
    hash_node_id("insp_anim_dir_pp"),
    hash_node_id("insp_anim_dir_ppr"),
];

// ── O TOCADOR (o estado, e não a autoria) ────────────────────────────────────────────────────

/// **«+ Add Animator»** — anexa o estado de reprodução a esta sprite.
pub const INSP_ANIM_ADD_PLAYER: NodeId = hash_node_id("insp_anim_add_player");
/// A caixa «Playing».
pub const INSP_ANIM_PLAYING: NodeId = hash_node_id("insp_anim_playing");
/// A caixa «Autoplay» — começa a tocar quando o projeto abre.
pub const INSP_ANIM_AUTOPLAY: NodeId = hash_node_id("insp_anim_autoplay");
/// A escala de velocidade, em múltiplos (`1,0` = normal; negativo toca ao contrário).
pub const INSP_ANIM_SPEED: NodeId = hash_node_id("insp_anim_speed");
/// **«Rewind»** — repõe o ciclo no princípio da animação atual.
pub const INSP_ANIM_REWIND: NodeId = hash_node_id("insp_anim_rewind");

/// **A barra de frames — e ela é ARRASTÁVEL** (pedido do Enio, 2026-08-23).
///
/// ⚠️ **Um `Slider`, e não um desenho.** Ela nasceu como duas barras pintadas à mão, sem id e sem
/// entrada no store: *bonita, informativa e morta sob o rato*. Ser um `Slider` registado é o que
/// faz o despachante dar-lhe o salto-ao-clique (`pointer_down`) e o arrasto (`pointer_move`) sem
/// uma linha de máquina nova — a mesma porta da Opacidade e do Emissive.
///
/// ⚠️ **O valor é `0..1` NORMALIZADO** (é o que um `Slider` guarda); quem o converte em célula é o
/// despacho da §11, que tem o intervalo da animação aberta. ⛔ Não guarde a célula aqui: o
/// intervalo muda com a grelha, e um `0..1` sobrevive a isso.
pub const INSP_ANIM_FRAME_SCRUB: NodeId = hash_node_id("insp_anim_frame_scrub");

/// A direção que SUBSTITUI a da animação. `0` = herdar; `1..=4` = as quatro de
/// `ph2d_ecs::AnimDirection::ALL`.
pub const INSP_ANIM_DIR_OVERRIDE: [NodeId; 5] = [
    hash_node_id("insp_anim_diro_inherit"),
    hash_node_id("insp_anim_diro_fwd"),
    hash_node_id("insp_anim_diro_rev"),
    hash_node_id("insp_anim_diro_pp"),
    hash_node_id("insp_anim_diro_ppr"),
];
/// O loop que substitui o da animação: `0` herdar · `1` ligado · `2` desligado.
pub const INSP_ANIM_LOOP_OVERRIDE: [NodeId; 3] = [
    hash_node_id("insp_anim_loop_inherit"),
    hash_node_id("insp_anim_loop_on"),
    hash_node_id("insp_anim_loop_off"),
];
