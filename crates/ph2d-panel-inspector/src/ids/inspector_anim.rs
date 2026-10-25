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
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/inspector_anim.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

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

/// O ritmo da animação em QUADROS POR SEGUNDO (a leitura do editor); o dado é o `frame_ms`, e a
/// conversão é `InspectorAnimRow::fps` / `frame_ms_of_fps`.
pub const INSP_ANIM_FPS: NodeId = hash_node_id("insp_anim_fps");

/// O ciclo como o artista o diz — a ordem é a de `AnimCycleMode::ALL` (Repeat · Once · Ping-pong).
pub const INSP_ANIM_CYCLE: [NodeId; 3] = [
    hash_node_id("insp_anim_cycle_repeat"),
    hash_node_id("insp_anim_cycle_once"),
    hash_node_id("insp_anim_cycle_ping_pong"),
];

/// Pausa no último frame do ciclo, em ms.
pub const INSP_ANIM_HOLD_MS: NodeId = hash_node_id("insp_anim_hold_ms");

/// Pausa entre ciclos, em ms.
pub const INSP_ANIM_DELAY_MS: NodeId = hash_node_id("insp_anim_delay_ms");

/// Quantos ciclos tocar. `0` = para sempre — ver o rótulo do campo.
pub const INSP_ANIM_REPEAT: NodeId = hash_node_id("insp_anim_repeat");

/// **A duração da célula que a BARRA está a mostrar** (spec §8.12). `0` = herda o `Frame ms`.
///
/// ⚠️ Ele vive ao lado da barra de propósito: ela **já** é o selector de célula, e um segundo
/// selector é um sítio onde os dois podem discordar.
pub const INSP_ANIM_FRAME_MS_THIS: NodeId = hash_node_id("insp_anim_frame_ms_this");

/// **O nome do sinal que esta animação grita ao ACABAR** (spec §8.10). Vazio = calada.
pub const INSP_ANIM_SIGNAL_FINISH: NodeId = hash_node_id("insp_anim_signal_finish");

/// **O nome do sinal que ela grita ao fechar um CICLO.** Vazio = calada.
///
/// ⚠️ Dois campos e não um mais uma fase: acabar e dar a volta distinguem-se por serem **nomes
/// diferentes** — é a lei que a física já escreveu para os contatos.
pub const INSP_ANIM_SIGNAL_LOOP: NodeId = hash_node_id("insp_anim_signal_loop");

/// A direção da animação selecionada — a ordem é a de `ph2d_ecs::AnimDirection::ALL`.
pub const INSP_ANIM_DIR: [NodeId; 4] = [
    hash_node_id("insp_anim_dir_fwd"),
    hash_node_id("insp_anim_dir_rev"),
    hash_node_id("insp_anim_dir_pp"),
    hash_node_id("insp_anim_dir_ppr"),
];

/// **«+ Add Animator»** — anexa o estado de reprodução a esta sprite.
pub const INSP_ANIM_ADD_PLAYER: NodeId = hash_node_id("insp_anim_add_player");

/// O ▶/⏸ da fileira «Playback» — o botão é o facto «a tocar» (a caixa «Playing» saiu).
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

// ── O cartão «Animation» (plano GameEngineFuncional 07) ─────────────────────────────────────────

/// O ▶ de cada linha da lista: escolhe a animação E põe-na a tocar. A ORDEM é o índice da animação.
pub const INSP_ANIM_PLAY_ROW: [NodeId; 64] = build_insp_anim_play_row();

const fn build_insp_anim_play_row() -> [NodeId; 64] {
    [
        hash_node_id("insp_anim_play_row_00"),
        hash_node_id("insp_anim_play_row_01"),
        hash_node_id("insp_anim_play_row_02"),
        hash_node_id("insp_anim_play_row_03"),
        hash_node_id("insp_anim_play_row_04"),
        hash_node_id("insp_anim_play_row_05"),
        hash_node_id("insp_anim_play_row_06"),
        hash_node_id("insp_anim_play_row_07"),
        hash_node_id("insp_anim_play_row_08"),
        hash_node_id("insp_anim_play_row_09"),
        hash_node_id("insp_anim_play_row_10"),
        hash_node_id("insp_anim_play_row_11"),
        hash_node_id("insp_anim_play_row_12"),
        hash_node_id("insp_anim_play_row_13"),
        hash_node_id("insp_anim_play_row_14"),
        hash_node_id("insp_anim_play_row_15"),
        hash_node_id("insp_anim_play_row_16"),
        hash_node_id("insp_anim_play_row_17"),
        hash_node_id("insp_anim_play_row_18"),
        hash_node_id("insp_anim_play_row_19"),
        hash_node_id("insp_anim_play_row_20"),
        hash_node_id("insp_anim_play_row_21"),
        hash_node_id("insp_anim_play_row_22"),
        hash_node_id("insp_anim_play_row_23"),
        hash_node_id("insp_anim_play_row_24"),
        hash_node_id("insp_anim_play_row_25"),
        hash_node_id("insp_anim_play_row_26"),
        hash_node_id("insp_anim_play_row_27"),
        hash_node_id("insp_anim_play_row_28"),
        hash_node_id("insp_anim_play_row_29"),
        hash_node_id("insp_anim_play_row_30"),
        hash_node_id("insp_anim_play_row_31"),
        hash_node_id("insp_anim_play_row_32"),
        hash_node_id("insp_anim_play_row_33"),
        hash_node_id("insp_anim_play_row_34"),
        hash_node_id("insp_anim_play_row_35"),
        hash_node_id("insp_anim_play_row_36"),
        hash_node_id("insp_anim_play_row_37"),
        hash_node_id("insp_anim_play_row_38"),
        hash_node_id("insp_anim_play_row_39"),
        hash_node_id("insp_anim_play_row_40"),
        hash_node_id("insp_anim_play_row_41"),
        hash_node_id("insp_anim_play_row_42"),
        hash_node_id("insp_anim_play_row_43"),
        hash_node_id("insp_anim_play_row_44"),
        hash_node_id("insp_anim_play_row_45"),
        hash_node_id("insp_anim_play_row_46"),
        hash_node_id("insp_anim_play_row_47"),
        hash_node_id("insp_anim_play_row_48"),
        hash_node_id("insp_anim_play_row_49"),
        hash_node_id("insp_anim_play_row_50"),
        hash_node_id("insp_anim_play_row_51"),
        hash_node_id("insp_anim_play_row_52"),
        hash_node_id("insp_anim_play_row_53"),
        hash_node_id("insp_anim_play_row_54"),
        hash_node_id("insp_anim_play_row_55"),
        hash_node_id("insp_anim_play_row_56"),
        hash_node_id("insp_anim_play_row_57"),
        hash_node_id("insp_anim_play_row_58"),
        hash_node_id("insp_anim_play_row_59"),
        hash_node_id("insp_anim_play_row_60"),
        hash_node_id("insp_anim_play_row_61"),
        hash_node_id("insp_anim_play_row_62"),
        hash_node_id("insp_anim_play_row_63"),
    ]
}

/// «Edit animations…» — abre o editor de animações desta sprite numa aba (`AnimFieldEdit::OpenEditor`).
pub const INSP_ANIM_OPEN_EDITOR: NodeId = hash_node_id("insp_anim_open_editor");

/// «Who picks»: Me · Script · The character. ⚠️ A POSIÇÃO é o `AnimChooser::tag`.
pub const INSP_ANIM_CHOOSER: [NodeId; 3] = [
    hash_node_id("insp_anim_chooser_me"),
    hash_node_id("insp_anim_chooser_script"),
    hash_node_id("insp_anim_chooser_character"),
];

/// Os quatro menus dos estados do personagem — Idle · Run · Jump · Fall (a ordem do
/// `AnimFieldEdit::CharacterState`).
pub const INSP_ANIM_STATE_PICK: [NodeId; 4] = [
    hash_node_id("insp_anim_state_idle"),
    hash_node_id("insp_anim_state_run"),
    hash_node_id("insp_anim_state_jump"),
    hash_node_id("insp_anim_state_fall"),
];

/// As linhas do popover de um estado: uma por animação da lista, pela ordem dela. ⚠️ Partilhadas pelos
/// quatro menus — só um popover está aberto de cada vez, e o despacho pergunta QUAL.
pub const INSP_ANIM_STATE_OPT: [NodeId; 64] = build_insp_anim_state_opt();

const fn build_insp_anim_state_opt() -> [NodeId; 64] {
    [
        hash_node_id("insp_anim_state_opt_00"),
        hash_node_id("insp_anim_state_opt_01"),
        hash_node_id("insp_anim_state_opt_02"),
        hash_node_id("insp_anim_state_opt_03"),
        hash_node_id("insp_anim_state_opt_04"),
        hash_node_id("insp_anim_state_opt_05"),
        hash_node_id("insp_anim_state_opt_06"),
        hash_node_id("insp_anim_state_opt_07"),
        hash_node_id("insp_anim_state_opt_08"),
        hash_node_id("insp_anim_state_opt_09"),
        hash_node_id("insp_anim_state_opt_10"),
        hash_node_id("insp_anim_state_opt_11"),
        hash_node_id("insp_anim_state_opt_12"),
        hash_node_id("insp_anim_state_opt_13"),
        hash_node_id("insp_anim_state_opt_14"),
        hash_node_id("insp_anim_state_opt_15"),
        hash_node_id("insp_anim_state_opt_16"),
        hash_node_id("insp_anim_state_opt_17"),
        hash_node_id("insp_anim_state_opt_18"),
        hash_node_id("insp_anim_state_opt_19"),
        hash_node_id("insp_anim_state_opt_20"),
        hash_node_id("insp_anim_state_opt_21"),
        hash_node_id("insp_anim_state_opt_22"),
        hash_node_id("insp_anim_state_opt_23"),
        hash_node_id("insp_anim_state_opt_24"),
        hash_node_id("insp_anim_state_opt_25"),
        hash_node_id("insp_anim_state_opt_26"),
        hash_node_id("insp_anim_state_opt_27"),
        hash_node_id("insp_anim_state_opt_28"),
        hash_node_id("insp_anim_state_opt_29"),
        hash_node_id("insp_anim_state_opt_30"),
        hash_node_id("insp_anim_state_opt_31"),
        hash_node_id("insp_anim_state_opt_32"),
        hash_node_id("insp_anim_state_opt_33"),
        hash_node_id("insp_anim_state_opt_34"),
        hash_node_id("insp_anim_state_opt_35"),
        hash_node_id("insp_anim_state_opt_36"),
        hash_node_id("insp_anim_state_opt_37"),
        hash_node_id("insp_anim_state_opt_38"),
        hash_node_id("insp_anim_state_opt_39"),
        hash_node_id("insp_anim_state_opt_40"),
        hash_node_id("insp_anim_state_opt_41"),
        hash_node_id("insp_anim_state_opt_42"),
        hash_node_id("insp_anim_state_opt_43"),
        hash_node_id("insp_anim_state_opt_44"),
        hash_node_id("insp_anim_state_opt_45"),
        hash_node_id("insp_anim_state_opt_46"),
        hash_node_id("insp_anim_state_opt_47"),
        hash_node_id("insp_anim_state_opt_48"),
        hash_node_id("insp_anim_state_opt_49"),
        hash_node_id("insp_anim_state_opt_50"),
        hash_node_id("insp_anim_state_opt_51"),
        hash_node_id("insp_anim_state_opt_52"),
        hash_node_id("insp_anim_state_opt_53"),
        hash_node_id("insp_anim_state_opt_54"),
        hash_node_id("insp_anim_state_opt_55"),
        hash_node_id("insp_anim_state_opt_56"),
        hash_node_id("insp_anim_state_opt_57"),
        hash_node_id("insp_anim_state_opt_58"),
        hash_node_id("insp_anim_state_opt_59"),
        hash_node_id("insp_anim_state_opt_60"),
        hash_node_id("insp_anim_state_opt_61"),
        hash_node_id("insp_anim_state_opt_62"),
        hash_node_id("insp_anim_state_opt_63"),
    ]
}

/// «Run from» (m/s) do modo «The character».
pub const INSP_ANIM_RUN_FROM: NodeId = hash_node_id("insp_anim_run_from");

/// «Flip when walking back» do modo «The character».
pub const INSP_ANIM_FLIP: NodeId = hash_node_id("insp_anim_flip");

/// O cabeçalho do grupo «Advanced» (os campos finos da animação e as substituições do tocador) —
/// nasce FECHADO.
pub const INSP_ANIM_ADVANCED: NodeId = hash_node_id("insp_anim_advanced");
