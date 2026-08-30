//! **O registo dos widgets da §12 Sockets / Named Anchors** ([ADR-0072]).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** (600) — o mesmo corte que o
//! [`super::populate_physics`] fez antes. Os dois controlos que o Enio pediu em 2026-08-23
//! (o botão de repor na âncora, e as duas caixas de visibilidade) levaram o `populate.rs` a
//! **605**, e a regra da casa é cortar para o irmão, nunca subir a tolerância.
//!
//! [ADR-0072]: ../../../docs/architecture/decisions/0072-named-anchor-unification.md

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue, DropdownState, TextInputState};

use super::populate::register_button_ids;

/// **§5 9-Slice** (spec Sprite 03 §3.5) — construída em 2026-08-21.
///
/// ⚠️ **Cada id aqui é o que torna o widget FOCÁVEL**, e é a ponta que este repositório já perdeu
/// seis vezes: um controlo pintado e não registado é indistinguível de um partido — o ponteiro
/// nunca lhe chega e todo gate de compilação continua verde (DIRETIVA §2). O gate
/// `every_painted_id_is_reachable` cobra esta metade.
/// **§12 Sockets / Named Anchors** (ADR-0072) — construída em 2026-08-21.
pub(crate) fn populate_anchors(store: &mut WidgetStore) {
    // As 64 linhas da lista + os dois botões. ⚠️ TODAS as 64 se registam, mesmo que a maioria
    // dos sprites tenha 3 âncoras: um id só registado "quando aparece" nunca aparece, porque o
    // registo acontece uma vez, no arranque, e a lista cresce depois.
    register_button_ids(store, &ids::INSP_ANCHOR_ROW);
    register_button_ids(store, &[ids::INSP_ANCHOR_ADD, ids::INSP_ANCHOR_REMOVE]);
    // §12 «Rides Parent Anchor» (ADR-0072 §2.6) — o chip mais as opções do popover.
    // ⚠️ `selected_index: None` de propósito: **quem sabe o que está montado é o snapshot**, e o
    // store só guarda se o popover está aberto. Semear um índice aqui faria o chip mostrar a
    // montagem do objeto anterior até o primeiro sync — *o seed é dono do VALOR, o dispatch do
    // ESTADO*, e aqui o valor não é do seed.
    register_button_ids(store, &ids::INSP_MOUNT_OPT);
    register_button_ids(store, &[ids::INSP_MOUNT_NONE_OPT, ids::INSP_MOUNT_SNAP]);
    store.register(
        ids::INSP_MOUNT_PICK,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );
    store.register(
        ids::INSP_ANCHOR_NAME,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    for id in [
        ids::INSP_ANCHOR_BOUNDS_ON,
        ids::INSP_ANCHOR_CENTER_ON,
        ids::INSP_ANCHOR_VIS_EDITOR,
    ] {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Unchecked,
            },
        );
    }
    // ⛔⛔ **A caixa «Show anchors at runtime» nasce PARADA, e o bloqueador tem nome:** não existe
    // modo de jogo (`shells/game` / Runtime R1, adiado por decisão do dono do produto). Ela
    // gravava no `.ph2dproj` e não tinha um único leitor — ver
    // [`crate::sections::anchor_mount_row::RUNTIME_BOX_LABEL`], onde está a razão inteira.
    //
    // ⚠️ **Parada no REGISTO e não só na pintura**: o `is_focusable` do despachante lê o estado do
    // store, então uma caixa pintada a cinzento e registada `Normal` continuaria a alternar sob o
    // dedo — o cinzento seria decoração. ⚠️ Ela FICA registada (em vez de sair do store) porque o
    // gate `every_painted_id_is_reachable` mede *«o que se pinta está registado»*, e um id pintado
    // sem registo é a família de defeito que ele existe para apanhar.
    store.register(
        ids::INSP_ANCHOR_VIS_RUNTIME,
        InteractiveState::Checkbox {
            state: CheckboxState::Disabled,
            value: CheckboxValue::Unchecked,
        },
    );
    store.set_tooltip(
        ids::INSP_ANCHOR_VIS_RUNTIME,
        "Parked: this app has no game runtime yet, so there is no runtime for anchors to show in. \
         The setting is kept in the file so saved projects still load.",
    );
    for id in ids::INSP_ANCHOR_POS
        .iter()
        .chain(std::iter::once(&ids::INSP_ANCHOR_ROT))
        .chain(ids::INSP_ANCHOR_BOUNDS.iter())
        .chain(ids::INSP_ANCHOR_CENTER.iter())
        .copied()
    {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: format_number(0.0),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
    }
}
