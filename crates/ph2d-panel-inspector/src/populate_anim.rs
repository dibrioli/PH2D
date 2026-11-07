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
    populate_anim_markers(store);
}

/// As linhas dos marcadores em quadro. A faixa do «Frame» depende da animação: quem a põe é o `sync`.
fn populate_anim_markers(store: &mut WidgetStore) {
    use crate::ids;
    register_button_ids(store, &[ids::INSP_ANIM_MARKER_ADD]);
    register_button_ids(store, &ids::INSP_ANIM_MARKER_DEL);
    store.set_tooltip(ids::INSP_ANIM_MARKER_ADD, tr("card.anim.marker_tip"));
    for i in 0..ids::MAX_ANIM_MARKER_ROWS {
        store.set_tooltip(ids::INSP_ANIM_MARKER_DEL[i], tr("card.anim.marker_del_tip"));
        store.register(
            ids::INSP_ANIM_MARKER_FRAME[i],
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 1.0,
                buffer: format_number(1.0),
                caret: 0,
                last_committed: 1.0,
                selection_anchor: None,
            },
        );
        store.register(
            ids::INSP_ANIM_MARKER_SIGNAL[i],
            InteractiveState::TextInput {
                state: TextInputState::Normal,
                text: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        );
    }
}

/// **Os balões do cartão** (plano 22): o que um nome curto não diz mora sob o rato (ordem do dono 21/09).
fn populate_anim_tips(store: &mut WidgetStore) {
    use crate::ids;
    for (id, key) in [
        (ids::INSP_ANIM_ADD, "panel.inspector.anim_card.add_tip"),
        (
            ids::INSP_ANIM_REMOVE,
            "panel.inspector.anim_card.remove_tip",
        ),
        (ids::INSP_ANIM_SPEED, "panel.inspector.anim_card.speed_tip"),
        (ids::INSP_ANIM_FPS, "panel.inspector.anim_card.fps_tip"),
        (
            ids::INSP_ANIM_CYCLE[0],
            "panel.inspector.anim_card.cycle_repeat_tip",
        ),
        (
            ids::INSP_ANIM_CYCLE[1],
            "panel.inspector.anim_card.cycle_once_tip",
        ),
        (
            ids::INSP_ANIM_CYCLE[2],
            "panel.inspector.anim_card.cycle_ping_pong_tip",
        ),
        (
            ids::INSP_ANIM_CHOOSER[1],
            "panel.inspector.anim_card.script_hint",
        ),
        (
            ids::INSP_ANIM_CHOOSER[2],
            "panel.inspector.anim_card.rules_hint",
        ),
        (ids::INSP_ANIM_HOLD_MS, "panel.inspector.anim_card.hold_tip"),
        (
            ids::INSP_ANIM_DELAY_MS,
            "panel.inspector.anim_card.delay_tip",
        ),
        (ids::INSP_ANIM_FROM, "panel.inspector.anim_card.from_tip"),
        (ids::INSP_ANIM_TO, "panel.inspector.anim_card.to_tip"),
        (
            ids::INSP_ANIM_SIGNAL_FINISH,
            "panel.inspector.anim_card.signal_finish_tip",
        ),
        (
            ids::INSP_ANIM_SIGNAL_LOOP,
            "panel.inspector.anim_card.signal_loop_tip",
        ),
    ] {
        store.set_tooltip(id, tr(key));
    }
    // «Me» é a propriedade `animator.autoplay`: o nome dela à frente da dica.
    store.set_tooltip(
        ids::INSP_ANIM_CHOOSER[0],
        format!(
            "{} · {}",
            tr("component.field.sprite_animator.2"),
            tr(if crate::cards::active() {
                "panel.inspector.anim_card.me_hint"
            } else {
                "panel.inspector.anim_card.me_hint_sem_menu"
            })
        ),
    );
    for (grupo, key) in [
        (
            &ids::INSP_ANIM_DIR[..],
            "panel.inspector.anim_card.direction_tip",
        ),
        (
            &ids::INSP_ANIM_DIR_OVERRIDE[..],
            "panel.inspector.anim_card.direction_override_tip",
        ),
        (
            &ids::INSP_ANIM_LOOP_OVERRIDE[..],
            "panel.inspector.anim_card.loop_override_tip",
        ),
    ] {
        for &id in grupo {
            store.set_tooltip(id, tr(key));
        }
    }
}

/// **O cartão «Animation»** (plano GameEngineFuncional 07): o ▶ de cada linha, «Edit Animations…»,
/// «Who Picks», a lista de regras e o grupo «Advanced» (nasce FECHADO).
fn populate_anim_card(store: &mut WidgetStore) {
    register_button_ids(store, &crate::ids::INSP_ANIM_PLAY_ROW);
    register_button_ids(store, &[crate::ids::INSP_ANIM_OPEN_EDITOR]);
    register_button_ids(store, &crate::ids::INSP_ANIM_CHOOSER);
    register_button_ids(store, &[crate::ids::INSP_ANIM_START_DD]);
    register_button_ids(store, &crate::ids::INSP_ANIM_START_OPT);
    store.set_tooltip(
        crate::ids::INSP_ANIM_START_DD,
        tr("panel.inspector.anim_card.starts_with_tip"),
    );
    // A lista de regras (plano 08): pega, ×, «+ Rule» e as linhas dos dois popovers são BOTÕES; os
    // menus «When» e «Play» de cada regra são `Dropdown`.
    register_button_ids(store, &crate::ids::INSP_ANIM_RULE_GRIP);
    register_button_ids(store, &crate::ids::INSP_ANIM_RULE_DEL);
    register_button_ids(store, &[crate::ids::INSP_ANIM_RULE_ADD]);
    register_button_ids(store, &crate::ids::INSP_ANIM_RULE_WHEN_OPT);
    register_button_ids(store, &crate::ids::INSP_ANIM_RULE_PLAY_OPT);
    for (ids, tip) in [
        (
            crate::ids::INSP_ANIM_RULE_WHEN,
            "panel.inspector.anim_card.rule_when_tip",
        ),
        (
            crate::ids::INSP_ANIM_RULE_PLAY,
            "panel.inspector.anim_card.rule_play_tip",
        ),
    ] {
        for id in ids {
            store.register(
                id,
                InteractiveState::Dropdown {
                    state: DropdownState::Normal,
                    open: false,
                    selected_index: None,
                },
            );
            store.set_tooltip(id, tr(tip));
        }
    }
    for (ids, tip) in [
        (
            crate::ids::INSP_ANIM_RULE_GRIP,
            "panel.inspector.anim_card.rule_grip_tip",
        ),
        (
            crate::ids::INSP_ANIM_RULE_DEL,
            "panel.inspector.anim_card.rule_del_tip",
        ),
    ] {
        for id in ids {
            store.set_tooltip(id, tr(tip));
        }
    }
    for id in crate::ids::INSP_ANIM_PLAY_ROW
        .into_iter()
        .chain([crate::ids::INSP_ANIM_PLAYING])
    {
        // O nome da propriedade à frente: é o que o script escreve (`animator.playing`).
        let tip = format!(
            "{} · {}",
            tr("component.field.sprite_animator.1"),
            tr("panel.inspector.anim_card.play_tip")
        );
        store.set_tooltip(id, tip);
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
    populate_anim_tips(store);
    // ⛔ `_if_unchosen` (a lei da máscara da câmera): re-popular não pode fechar o que o artista abriu.
    store.set_collapsed_if_unchosen(crate::ids::INSP_ANIM_ADVANCED, true);
    store.mark_collapsible_section(crate::ids::INSP_ANIM_ADVANCED);
}
