//! **Os ids da secção SIGNAL ACTIONS** (TOP-20 #5, W3).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — mesmo padrão do
//! [`super::inspector_timer`], de quem esta secção é o gémeo estrutural: lista + um editor.
//!
//! ⚠️ **A POSIÇÃO NO ARRAY DOS VERBOS É A TAG** — o despacho deriva-a de
//! `position(|&o| o == id)` e o modelo lê-a com `SignalVerb::from_tag`. ⛔ Reordenar
//! [`INSP_ACTION_VERB`] faria um clique escrever outro verbo, **e compila**.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/inspector_action.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// `+ Add Action`.
pub const INSP_ACTION_ADD: NodeId = hash_node_id("insp_action_add");

/// `x Remove Action` — apaga a que está aberta.
pub const INSP_ACTION_REMOVE: NodeId = hash_node_id("insp_action_remove");

/// O nome do sinal que dispara esta linha. **Vazio = nunca.**
pub const INSP_ACTION_ON: NodeId = hash_node_id("insp_action_on");

/// O NOME do objecto que sofre a acção. **Vazio = este objecto.**
pub const INSP_ACTION_TARGET: NodeId = hash_node_id("insp_action_target");

/// O parâmetro do verbo — hoje, o nome do timer. **Vazio = todos.**
pub const INSP_ACTION_ARG: NodeId = hash_node_id("insp_action_arg");

/// ⭐⭐⭐ **O alvo é por NOME** — o segmento da esquerda (TOP-20 #9, W3b).
pub const INSP_ACTION_BY_NAME: NodeId = hash_node_id("insp_action_by_name");
/// ⭐⭐⭐ **O alvo é por TAG** — o segmento da direita.
///
/// ⚠️ **Dois ids e não um toggle**, porque é um `paint_segmented_group_adaptive`: ele pinta N
/// opções com uma marcada, e cada uma precisa do id dela para o clique saber qual foi.
pub const INSP_ACTION_BY_TAG: NodeId = hash_node_id("insp_action_by_tag");
/// ⭐⭐⭐ **O alvo é QUEM BATEU** — o terceiro segmento (suplente #24, 2026-09-19).
pub const INSP_ACTION_BY_OTHER: NodeId = hash_node_id("insp_action_by_other");

/// ⭐⭐⭐ **A CERCA: esta linha reage a QUALQUER UM** — o segmento da esquerda (suplente #24).
pub const INSP_ACTION_FROM_ANYONE: NodeId = hash_node_id("insp_action_from_anyone");
/// ⭐⭐⭐ **A CERCA: esta linha só reage ao PRÓPRIO golpe** — o segmento da direita.
///
/// ⚠️ **Dois ids e não um toggle**, pela mesma razão dos do alvo: é um segmentado, e cada opção
/// precisa do id dela para o clique saber qual foi.
pub const INSP_ACTION_FROM_MYSELF: NodeId = hash_node_id("insp_action_from_myself");

/// **A tag alvo — o CHIP do seletor.** As entradas dele são [`INSP_ACTION_TAG_OPT`].
///
/// ⚠️ **Não é o `INSP_TAGS_PICK` da secção Tags**: dois chips com o mesmo id em duas secções do
/// MESMO painel partilhariam o estado `open`, e abrir um abriria o outro.
pub const INSP_ACTION_TAG_PICK: NodeId = hash_node_id("insp_action_tag_pick");

/// **As opções da tag alvo** — o mesmo tecto MEDIDO do `INSP_TAGS_OPT` (64 linhas, o ecrã maior
/// mais dois; ver o doc de lá para a tabela).
/// ⛔⛔ **Ids PRÓPRIOS, e não os do `INSP_TAGS_OPT`.** Os dois selectores nunca estão abertos ao
/// mesmo tempo, mas o DESPACHO não sabe isso: ele resolve a opção por `position()` sobre o array,
/// e com o mesmo array os dois braços casariam — o da secção *Tags* corre primeiro no router, logo
/// escolher uma tag AQUI marcaria o objecto em vez de apontar a acção. *Dois gestos diferentes com
/// o mesmo id é um deles a comer o outro, em silêncio.*
pub const INSP_ACTION_TAG_OPT: [NodeId; 64] = [
    hash_node_id("insp_action_tag_opt_00"),
    hash_node_id("insp_action_tag_opt_01"),
    hash_node_id("insp_action_tag_opt_02"),
    hash_node_id("insp_action_tag_opt_03"),
    hash_node_id("insp_action_tag_opt_04"),
    hash_node_id("insp_action_tag_opt_05"),
    hash_node_id("insp_action_tag_opt_06"),
    hash_node_id("insp_action_tag_opt_07"),
    hash_node_id("insp_action_tag_opt_08"),
    hash_node_id("insp_action_tag_opt_09"),
    hash_node_id("insp_action_tag_opt_10"),
    hash_node_id("insp_action_tag_opt_11"),
    hash_node_id("insp_action_tag_opt_12"),
    hash_node_id("insp_action_tag_opt_13"),
    hash_node_id("insp_action_tag_opt_14"),
    hash_node_id("insp_action_tag_opt_15"),
    hash_node_id("insp_action_tag_opt_16"),
    hash_node_id("insp_action_tag_opt_17"),
    hash_node_id("insp_action_tag_opt_18"),
    hash_node_id("insp_action_tag_opt_19"),
    hash_node_id("insp_action_tag_opt_20"),
    hash_node_id("insp_action_tag_opt_21"),
    hash_node_id("insp_action_tag_opt_22"),
    hash_node_id("insp_action_tag_opt_23"),
    hash_node_id("insp_action_tag_opt_24"),
    hash_node_id("insp_action_tag_opt_25"),
    hash_node_id("insp_action_tag_opt_26"),
    hash_node_id("insp_action_tag_opt_27"),
    hash_node_id("insp_action_tag_opt_28"),
    hash_node_id("insp_action_tag_opt_29"),
    hash_node_id("insp_action_tag_opt_30"),
    hash_node_id("insp_action_tag_opt_31"),
    hash_node_id("insp_action_tag_opt_32"),
    hash_node_id("insp_action_tag_opt_33"),
    hash_node_id("insp_action_tag_opt_34"),
    hash_node_id("insp_action_tag_opt_35"),
    hash_node_id("insp_action_tag_opt_36"),
    hash_node_id("insp_action_tag_opt_37"),
    hash_node_id("insp_action_tag_opt_38"),
    hash_node_id("insp_action_tag_opt_39"),
    hash_node_id("insp_action_tag_opt_40"),
    hash_node_id("insp_action_tag_opt_41"),
    hash_node_id("insp_action_tag_opt_42"),
    hash_node_id("insp_action_tag_opt_43"),
    hash_node_id("insp_action_tag_opt_44"),
    hash_node_id("insp_action_tag_opt_45"),
    hash_node_id("insp_action_tag_opt_46"),
    hash_node_id("insp_action_tag_opt_47"),
    hash_node_id("insp_action_tag_opt_48"),
    hash_node_id("insp_action_tag_opt_49"),
    hash_node_id("insp_action_tag_opt_50"),
    hash_node_id("insp_action_tag_opt_51"),
    hash_node_id("insp_action_tag_opt_52"),
    hash_node_id("insp_action_tag_opt_53"),
    hash_node_id("insp_action_tag_opt_54"),
    hash_node_id("insp_action_tag_opt_55"),
    hash_node_id("insp_action_tag_opt_56"),
    hash_node_id("insp_action_tag_opt_57"),
    hash_node_id("insp_action_tag_opt_58"),
    hash_node_id("insp_action_tag_opt_59"),
    hash_node_id("insp_action_tag_opt_60"),
    hash_node_id("insp_action_tag_opt_61"),
    hash_node_id("insp_action_tag_opt_62"),
    hash_node_id("insp_action_tag_opt_63"),
];

/// **O verbo — o CHIP do seletor.** As entradas dele são [`INSP_ACTION_VERB`].
///
/// ⚠️ **Ele é o único id desta família registado como `Dropdown`**: o `open` do popover é o
/// estado dele, e a ESCOLHA nunca vive aqui — ela é do snapshot, relida a cada quadro. *O seed é
/// dono do valor, o dispatch é dono do estado* (a lei que a §12 já paga).
pub const INSP_ACTION_VERB_PICK: NodeId = hash_node_id("insp_action_verb_pick");

// ── Desceu de `ph2d-editor-core/src/ids/inspector_action.rs` em 2026-09-13 (2.ª passagem: a cerca com a
//    `line/render-loop` prendia-os na fundação até às duas linhas se integrarem).

/// **As linhas da lista** — uma por acção, até ao cap de [`ph2d_ecs::SIGNAL_ACTIONS_MAX`].
///
/// ⚠️ O comprimento deste array **é** o cap do modelo, com gate na shell: *um modelo que aceita o
/// que o painel não mostra produz estado inalcançável*.
pub const INSP_ACTION_ROW: [NodeId; 16] = [
    hash_node_id("insp_action_row_00"),
    hash_node_id("insp_action_row_01"),
    hash_node_id("insp_action_row_02"),
    hash_node_id("insp_action_row_03"),
    hash_node_id("insp_action_row_04"),
    hash_node_id("insp_action_row_05"),
    hash_node_id("insp_action_row_06"),
    hash_node_id("insp_action_row_07"),
    hash_node_id("insp_action_row_08"),
    hash_node_id("insp_action_row_09"),
    hash_node_id("insp_action_row_10"),
    hash_node_id("insp_action_row_11"),
    hash_node_id("insp_action_row_12"),
    hash_node_id("insp_action_row_13"),
    hash_node_id("insp_action_row_14"),
    hash_node_id("insp_action_row_15"),
];

/// **O verbo**, uma OPÇÃO do seletor por entrada de `SignalVerb::ALL`.
///
/// ⚠️ **A posição é a tag** — ver o doc do módulo. ⚠️ Estes ids eram uma fileira de cinco botões
/// até 2026-09-09 (*«as actions deveriam ficar num dropdown e não em muitos botões»*, report do
/// dono): passaram a ser as linhas do popover **sem mudar de significado**, que é o que manteve o
/// despacho — `position(|&o| o == id)` — intacto.
pub const INSP_ACTION_VERB: [NodeId; 12] = [
    hash_node_id("insp_action_verb_start"),
    hash_node_id("insp_action_verb_stop"),
    hash_node_id("insp_action_verb_show"),
    hash_node_id("insp_action_verb_hide"),
    hash_node_id("insp_action_verb_toggle"),
    hash_node_id("insp_action_verb_play_sound"),
    hash_node_id("insp_action_verb_stop_sound"),
    // ⚠️ **APENDADO, e nunca no meio** — a posição é a tag, logo inserir um id acima faria todo
    // `SignalAction` já gravado mudar de verbo, em silêncio. (TOP-20 #20: o `Add to counter`.)
    hash_node_id("insp_action_verb_add_to_counter"),
    // ⭐⭐⭐ **O `Destroy`** (suplente #24), APENDADO pela mesma lei. ⚠️ **Ele nasceu em falta e foi
    // o gate `the_verb_labels_come_from_the_engines_own_list` que o apanhou** (`left: 9, right: 8`):
    // sem este id o verbo existe, tem lei, tem gates — e o artista **não lhe chega**. *É a forma
    // exacta do defeito que o `Density` da escultura pagou.*
    hash_node_id("insp_action_verb_destroy"),
    // ⭐⭐⭐ **O `Restart Run`** (o FIM DE JOGO, 2026-09-19), APENDADO pela mesma lei — e escrito no
    // mesmo commit da variante, que é a lição que o `Destroy` deixou uma wave antes.
    hash_node_id("insp_action_verb_restart_run"),
    // ⭐ **O `Damage` e o `Heal`** (plano 28, W2b), APENDADOS pela mesma lei e no mesmo commit.
    hash_node_id("insp_action_verb_damage"),
    hash_node_id("insp_action_verb_heal"),
];

/// **As opções do filtro por tag da §11 Physics** (TOP-20 #9, W3c).
///
/// ⛔ **Ids PRÓPRIOS**, pela razão que o [`INSP_ACTION_TAG_OPT`] já escreve: o despacho resolve a
/// opção por `position()` sobre o array, e com o mesmo array dois braços casariam — o primeiro do
/// router comeria o clique do outro.
pub const INSP_PHYS_TAG_OPT: [NodeId; 64] = [
    hash_node_id("insp_phys_tag_opt_00"),
    hash_node_id("insp_phys_tag_opt_01"),
    hash_node_id("insp_phys_tag_opt_02"),
    hash_node_id("insp_phys_tag_opt_03"),
    hash_node_id("insp_phys_tag_opt_04"),
    hash_node_id("insp_phys_tag_opt_05"),
    hash_node_id("insp_phys_tag_opt_06"),
    hash_node_id("insp_phys_tag_opt_07"),
    hash_node_id("insp_phys_tag_opt_08"),
    hash_node_id("insp_phys_tag_opt_09"),
    hash_node_id("insp_phys_tag_opt_10"),
    hash_node_id("insp_phys_tag_opt_11"),
    hash_node_id("insp_phys_tag_opt_12"),
    hash_node_id("insp_phys_tag_opt_13"),
    hash_node_id("insp_phys_tag_opt_14"),
    hash_node_id("insp_phys_tag_opt_15"),
    hash_node_id("insp_phys_tag_opt_16"),
    hash_node_id("insp_phys_tag_opt_17"),
    hash_node_id("insp_phys_tag_opt_18"),
    hash_node_id("insp_phys_tag_opt_19"),
    hash_node_id("insp_phys_tag_opt_20"),
    hash_node_id("insp_phys_tag_opt_21"),
    hash_node_id("insp_phys_tag_opt_22"),
    hash_node_id("insp_phys_tag_opt_23"),
    hash_node_id("insp_phys_tag_opt_24"),
    hash_node_id("insp_phys_tag_opt_25"),
    hash_node_id("insp_phys_tag_opt_26"),
    hash_node_id("insp_phys_tag_opt_27"),
    hash_node_id("insp_phys_tag_opt_28"),
    hash_node_id("insp_phys_tag_opt_29"),
    hash_node_id("insp_phys_tag_opt_30"),
    hash_node_id("insp_phys_tag_opt_31"),
    hash_node_id("insp_phys_tag_opt_32"),
    hash_node_id("insp_phys_tag_opt_33"),
    hash_node_id("insp_phys_tag_opt_34"),
    hash_node_id("insp_phys_tag_opt_35"),
    hash_node_id("insp_phys_tag_opt_36"),
    hash_node_id("insp_phys_tag_opt_37"),
    hash_node_id("insp_phys_tag_opt_38"),
    hash_node_id("insp_phys_tag_opt_39"),
    hash_node_id("insp_phys_tag_opt_40"),
    hash_node_id("insp_phys_tag_opt_41"),
    hash_node_id("insp_phys_tag_opt_42"),
    hash_node_id("insp_phys_tag_opt_43"),
    hash_node_id("insp_phys_tag_opt_44"),
    hash_node_id("insp_phys_tag_opt_45"),
    hash_node_id("insp_phys_tag_opt_46"),
    hash_node_id("insp_phys_tag_opt_47"),
    hash_node_id("insp_phys_tag_opt_48"),
    hash_node_id("insp_phys_tag_opt_49"),
    hash_node_id("insp_phys_tag_opt_50"),
    hash_node_id("insp_phys_tag_opt_51"),
    hash_node_id("insp_phys_tag_opt_52"),
    hash_node_id("insp_phys_tag_opt_53"),
    hash_node_id("insp_phys_tag_opt_54"),
    hash_node_id("insp_phys_tag_opt_55"),
    hash_node_id("insp_phys_tag_opt_56"),
    hash_node_id("insp_phys_tag_opt_57"),
    hash_node_id("insp_phys_tag_opt_58"),
    hash_node_id("insp_phys_tag_opt_59"),
    hash_node_id("insp_phys_tag_opt_60"),
    hash_node_id("insp_phys_tag_opt_61"),
    hash_node_id("insp_phys_tag_opt_62"),
    hash_node_id("insp_phys_tag_opt_63"),
];
