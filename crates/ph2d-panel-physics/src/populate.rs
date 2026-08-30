//! Widget registration — walked from [`crate::rows::rows`], so a painted row
//! cannot be an unregistered one.
//!
//! A widget that `paint` hit-indexes but nobody registers is `is_focusable() ==
//! false`, and its click is dropped **in silence** — no compile error, no
//! warning, just a control that does nothing (the vector-pills class of bug
//! `architecture_panel_wiring_parity` exists to catch). Deriving this list from
//! the same table `paint` walks is how that stops being a thing anyone can
//! forget.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};

use crate::{interact, rows};

fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

pub fn populate(store: &mut WidgetStore) {
    for row in rows::rows() {
        store.register(
            row.slider,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.5,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            row.chip,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: "0".to_string(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_slider_number_mapped(row.slider, row.chip, row.scale(), row.offset());
        // ⚠️ The range is registered HERE, and it is not optional: without it
        // the chip derives its drag step from the buffer text and scrubs ~50
        // units per pixel, so one pixel of drag hits the ceiling and the chip
        // becomes a min↔max switch. Typing still works, which is why this class
        // of bug survives review.
        store.set_number_range(row.chip, f64::from(row.min), f64::from(row.max), row.step);
    }

    // The Interaction rows (W-Hand) — the SECOND table, registered by the same
    // loop shape for the same reason. A row that `paint` hit-indexes and nobody
    // registers is `is_focusable() == false` and its click is dropped in silence.
    for row in interact::IROWS {
        store.register(
            row.slider,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.5,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            row.chip,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: "0".to_string(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_slider_number_mapped(row.slider, row.chip, row.scale(), row.offset());
        store.set_number_range(row.chip, f64::from(row.min), f64::from(row.max), row.step);
    }

    // Os rádios da seção Interaction. As OPÇÕES são o que o ponteiro acerta, e
    // o `paint_segmented_adaptive` registra retângulo de hit mas NÃO entrada de
    // store — então uma opção não registrada fica pintada, hit-indexada e morta
    // sob o mouse (a falha exata que as 36 células da matriz ensinaram a este
    // painel).
    //
    // ⚠️ **Uma lista de listas, e não três laços.** Eram dois rádios, viraram
    // três com a Pose (W-IK), e um laço por array é a forma que apodrece: o
    // quarto nasce fora da regra e o modo de falha é *pintado e morto*, que
    // nenhum gate de compilação vê.
    for group in [
        &ids::PHYSICS_INTERACT_TOOL_OPT[..],
        &ids::PHYSICS_HOLD_MODE_OPT[..],
        &ids::PHYSICS_IK_ANGLE_OPT[..],
        &ids::PHYSICS_JOINT_TOOL_OPT[..],
    ] {
        for &id in group {
            button(store, id);
        }
    }

    // Section headers are interactive (the chevron folds them), so they are
    // registered like any other control — a painted chevron that nobody
    // registered is an affordance that does nothing.
    // ⚠️ A UMA lista (`rows::section_header_ids`), e não os quatro nomes soltos que aqui estavam:
    // eles eram a 1.ª de três cópias da mesma enumeração, e a 3.ª (o `event.rs`) tinha três.
    for id in rows::section_header_ids() {
        button(store, id);
    }

    // The 36 matrix cells. Registered in a loop, which is exactly why the seam
    // test clicks every one of them: `architecture_panel_wiring_parity` cannot
    // see loop registrations, so nothing else would notice these going dead.
    for &cell in ids::PHYSICS_LAYER_CELL.iter() {
        button(store, cell);
    }

    // Commands. Registered as Buttons — including "Show Colliders", which LOOKS
    // like a checkbox and is not one: a `Checkbox` emits `Toggled`, which this
    // panel's `event.rs` does not forward, so it would be registered and dead
    // (the painter-layers sculpt segments carry the same warning).
    button(store, ids::PHYSICS_SHOW_COLLIDERS);
    // ⭐ **O interruptor de adormecer** (2026-08-30) — registado como Button, e não pelo laço das
    // `rows`, porque ele deixou de ser um slider: a `rapier2d` 0.35 lê do
    // `sleep_angular_threshold` apenas o SINAL. Mesma família do «Show Colliders» acima, e ⛔ pela
    // mesma razão não é um `Checkbox`: este painel não encaminha `Toggled`.
    button(store, ids::PHYSICS_SLEEP_SPIN);
    // Os dois verbos de FITA (W25). Registrados sempre, pintados só quando há o
    // que descartar ou o que devolver: registrar é barato e a alternativa —
    // registrar condicionalmente — faria o botão nascer morto sob o mouse no
    // primeiro frame em que ele aparece, que é o defeito que este `populate`
    // existe para não ter.
    button(store, ids::PHYSICS_CLEAR_RUN);
    button(store, ids::PHYSICS_RESTORE_RUN);
    button(store, ids::PHYSICS_RESET_DEFAULTS);
    button(store, ids::PHYSICS_CLOSE);
}
