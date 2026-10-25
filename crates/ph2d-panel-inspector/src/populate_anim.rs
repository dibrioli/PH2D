//! **O registo dos widgets da §11 Animation** (spec Sprite 08 §8.7).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte que a §12 fez.
//!
//! ⚠️ **Nenhum valor é semeado aqui.** Quem sabe o que está a tocar, a que velocidade e com que
//! intervalo é o **snapshot**; o store só guarda o visual e o texto que se está a escrever. Semear
//! números aqui faria a §11 de uma sprite mostrar os da sprite anterior até ao primeiro sync —
//! *o seed é dono do VALOR, o dispatch é dono do ESTADO*, e aqui o valor não é do seed.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{
    CheckboxState, CheckboxValue, DropdownState, SliderOrientation, SliderState, TextInputState,
};
use ph2d_i18n::tr;

use super::populate::register_button_ids;

pub(crate) fn populate_anim(store: &mut WidgetStore) {
    // A lista, os dois botões da biblioteca, o botão que anexa o tocador, o rebobinar, e os
    // três segmentados — todos BOTÕES, porque é o `is_focusable` que decide se o clique chega.
    register_button_ids(store, &crate::ids::INSP_ANIM_ROW);
    register_button_ids(
        store,
        &[
            crate::ids::INSP_ANIM_ADD,
            crate::ids::INSP_ANIM_REMOVE,
            crate::ids::INSP_ANIM_ADD_PLAYER,
            crate::ids::INSP_ANIM_REWIND,
            crate::ids::INSP_ANIM_PLAYING,
        ],
    );
    register_button_ids(store, &crate::ids::INSP_ANIM_DIR);
    register_button_ids(store, &crate::ids::INSP_ANIM_CYCLE);
    register_button_ids(store, &crate::ids::INSP_ANIM_DIR_OVERRIDE);
    register_button_ids(store, &crate::ids::INSP_ANIM_LOOP_OVERRIDE);

    // ⚠️ **A barra de frames é um `Slider` REGISTADO, e é isso que a torna arrastável.** Sem
    // entrada no store ela é pintada, hit-registada e **morta sob o rato** — o despachante decide
    // pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio.
    store.register(
        crate::ids::INSP_ANIM_FRAME_SCRUB,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.0,
            orientation: SliderOrientation::Horizontal,
        },
    );

    store.register(
        crate::ids::INSP_ANIM_FLIP,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Unchecked,
        },
    );
    // ⚠️ Os três campos de TEXTO da §11: o nome da animação e os dois nomes de sinal (§8.10).
    for id in [
        crate::ids::INSP_ANIM_NAME,
        crate::ids::INSP_ANIM_SIGNAL_FINISH,
        crate::ids::INSP_ANIM_SIGNAL_LOOP,
    ] {
        store.register(
            id,
            InteractiveState::TextInput {
                state: TextInputState::Normal,
                text: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        );
    }
    for (id, value) in [
        (crate::ids::INSP_ANIM_FROM, 0.0_f64),
        (crate::ids::INSP_ANIM_TO, 0.0),
        (crate::ids::INSP_ANIM_FPS, 10.0), // LITERAL-PX-OK: quadros por segundo (100 ms), valor de domínio
        (crate::ids::INSP_ANIM_FRAME_MS_THIS, 1.0), // 1× = herda o ritmo (lei do Godot)
        (crate::ids::INSP_ANIM_HOLD_MS, 0.0),
        (crate::ids::INSP_ANIM_DELAY_MS, 0.0),
        (crate::ids::INSP_ANIM_REPEAT, 0.0),
        (crate::ids::INSP_ANIM_SPEED, 1.0),
        (crate::ids::INSP_ANIM_RUN_FROM, 0.5), // LITERAL-PX-OK: m/s, o `CharacterAnimations::default`
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format_number(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
    }
    // ⚠️ **A velocidade tem faixa**, e ela é a do motor (`ph2d_ecs::SPEED_MAX_Q16`, ±100×). Sem
    // isto o scrub de arrasto teria passo livre sobre um valor que o commit depois trunca — o
    // knob prometeria uma excursão que a cena não tem.
    store.set_number_range(crate::ids::INSP_ANIM_SPEED, -100.0, 100.0, 0.1); // LITERAL-PX-OK: faixa do motor
    // m/s relativos ao chão; o tecto 10 é a faixa da UI (o `speed` de partida é 6), não recurso.
    store.set_number_range(crate::ids::INSP_ANIM_RUN_FROM, 0.0, 10.0, 0.1); // LITERAL-PX-OK: m/s
    populate_anim_card(store);
}

/// **O cartão «Animation»** (plano GameEngineFuncional 07): o ▶ de cada linha, «Edit Animations…»,
/// «Who Picks», os quatro menus de estado e o grupo «Advanced» (nasce FECHADO).
fn populate_anim_card(store: &mut WidgetStore) {
    register_button_ids(store, &crate::ids::INSP_ANIM_PLAY_ROW);
    register_button_ids(store, &[crate::ids::INSP_ANIM_OPEN_EDITOR]);
    register_button_ids(store, &crate::ids::INSP_ANIM_CHOOSER);
    register_button_ids(store, &crate::ids::INSP_ANIM_STATE_OPT);
    for id in crate::ids::INSP_ANIM_STATE_PICK {
        store.register(
            id,
            InteractiveState::Dropdown {
                state: DropdownState::Normal,
                open: false,
                selected_index: None,
            },
        );
        store.set_tooltip(id, tr("panel.inspector.anim_card.state_tip"));
    }
    for id in crate::ids::INSP_ANIM_PLAY_ROW
        .into_iter()
        .chain([crate::ids::INSP_ANIM_PLAYING])
    {
        store.set_tooltip(id, tr("panel.inspector.anim_card.play_tip"));
    }
    store.set_tooltip(
        crate::ids::INSP_ANIM_REWIND,
        tr("panel.inspector.anim_card.rewind_tip"),
    );
    store.set_tooltip(
        crate::ids::INSP_ANIM_OPEN_EDITOR,
        tr("panel.inspector.anim_card.open_editor_tip"),
    );
    // «Show Sheet» é curto (cabe na coluna única do painel); o «onde» mora no balão.
    store.set_tooltip(
        crate::ids::INSP_SHEET_PREVIEW,
        tr("panel.inspector.sprite_sheet.show_sheet_tip"),
    );
    store.set_tooltip(
        crate::ids::INSP_ANIM_RUN_FROM,
        tr("panel.inspector.anim_card.run_from_tip"),
    );
    store.set_tooltip(
        crate::ids::INSP_ANIM_FLIP,
        tr("panel.inspector.anim_card.flip_tip"),
    );
    // ⛔ `_if_unchosen` (a lei da máscara da câmera): re-popular não pode fechar o que o artista abriu.
    store.set_collapsed_if_unchosen(crate::ids::INSP_ANIM_ADVANCED, true);
    store.mark_collapsible_section(crate::ids::INSP_ANIM_ADVANCED);
}
