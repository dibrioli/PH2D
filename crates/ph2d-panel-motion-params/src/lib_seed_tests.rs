//! **O SEED e o DISPATCH dividem o widget** — irmão `#[path]` de `lib_tests.rs`,
//! cortado no teto de 600 LOC pela linha de ASSUNTO: tudo aqui responde *quem é
//! dono de que campo do widget quando o painel se re-semeia a cada quadro*, e
//! nada mais responde isso.
//!
//! A lei, numa frase: **o seed é dono do VALOR, o dispatch é dono do ESTADO.**
//! O `mirror_number`/`mirror_text` já a cumpriam por construção (registram uma
//! vez e depois remendam campos), e o ramo do toggle a diz em prosa; o ramo
//! escalar fazia o oposto quatro dezenas de linhas abaixo.

use super::*;

/// A pooled scalar row, the shape `seed_rows` mirrors every frame.
fn scalar_row_snapshot(value: f64) -> ParamsSnapshot {
    ParamsSnapshot {
        node: 7,
        title: "Grid".into(),
        modified: Default::default(),
        sections: Vec::new(),
        rows: vec![ParamRow::Scalar(ScalarRow {
            name: "rows",
            label: "Rows".into(),
            value,
            min: 0.0,
            max: 10.0,
            hard_min: 0.0,
            hard_max: 10.0,
            step: 1.0,
            integer: false,
            driven_by: None,
            display: RowDisplay::default(),
        })],
    }
}

/// **A paint does not erase the state the pointer wrote.**
///
/// `seed_rows` mirrors the doc into the pooled widgets on every paint, and it used
/// to do that by re-registering them whole — with `state` hardcoded to `Normal`.
/// The pointer pass writes `Hovered` and the paint immediately after threw it away,
/// so a row in the Motion params panel lit under the mouse and went out again
/// before it was ever drawn. Measured, one paint: `Hovered -> Normal`, on BOTH
/// halves — and the chip half had read hover since the day it was written, so it
/// was dark long before the slider learnt to react.
///
/// The three neighbours in the same file already had this right (the toggle branch
/// copies the state across; `mirror_number`/`mirror_text` patch in place and never
/// touch it). The scalar branch was the one outlier.
///
/// **Mutation that must bleed:** `mirror_slider`/`mirror_chip` going back to a
/// wholesale `store.register(..)` with `SliderState::Normal` / `TextInputState::Normal`.
#[test]
fn a_paint_does_not_erase_the_hover_the_pointer_wrote() {
    set_current_params(Some(scalar_row_snapshot(3.0)));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let slider = param_slider_id(0);
    let chip = param_chip_id(0);
    host.store_mut().register(
        slider,
        InteractiveState::Slider {
            state: SliderState::Hovered,
            value: 0.3,
            orientation: SliderOrientation::Horizontal,
        },
    );
    host.store_mut().register(
        chip,
        InteractiveState::NumberInput {
            state: TextInputState::Hovered,
            value: 3.0,
            buffer: "3".into(),
            caret: 0,
            last_committed: 3.0,
            selection_anchor: None,
        },
    );
    host.paint::<MotionParamsPanel>(
        &mut state,
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 300.0, 800.0),
    );
    assert_eq!(
        host.store().slider(slider).map(|(s, _)| s),
        Some(SliderState::Hovered),
        "the paint erased the slider's hover: the row goes dark under the mouse"
    );
    assert!(
        matches!(
            host.store().get(chip),
            Some(InteractiveState::NumberInput {
                state: TextInputState::Hovered,
                ..
            })
        ),
        "the paint erased the chip's hover"
    );
    set_current_params(None);
}

/// **The CONTROL: the seed still seeds.**
///
/// Without this half, "never mirror anything" would satisfy the gate above — the
/// state survives beautifully when nobody writes to the widget at all. The seed
/// owns the VALUE, and a doc value that moved has to reach both halves of the row
/// on the very next paint.
#[test]
fn the_seed_still_mirrors_the_doc_value_into_both_halves() {
    set_current_params(Some(scalar_row_snapshot(2.0)));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionParamsPanel>();
    let mut state = MotionParamsPanelState;
    let slider = param_slider_id(0);
    let chip = param_chip_id(0);
    let viewport = ph2d_editor_core::zones::Rect::new(0.0, 0.0, 300.0, 800.0);
    host.paint::<MotionParamsPanel>(&mut state, viewport);
    assert!(
        (host.store().slider(slider).map_or(-1.0, |(_, v)| v) - 0.2).abs() < 1e-5,
        "the track did not take the doc value"
    );
    // The document moves under a panel that is not being touched.
    set_current_params(Some(scalar_row_snapshot(9.0)));
    host.paint::<MotionParamsPanel>(&mut state, viewport);
    assert!(
        (host.store().slider(slider).map_or(-1.0, |(_, v)| v) - 0.9).abs() < 1e-5,
        "the track did not follow the doc: the seed stopped seeding"
    );
    let Some(InteractiveState::NumberInput {
        value,
        buffer,
        last_committed,
        ..
    }) = host.store().get(chip)
    else {
        panic!("chip missing");
    };
    assert!(
        (*value - 9.0).abs() < 1e-9,
        "the chip did not follow the doc"
    );
    assert!(
        (*last_committed - 9.0).abs() < 1e-9,
        "the chip's committed value did not follow the doc"
    );
    assert_eq!(
        buffer,
        &format_number(9.0),
        "the chip's text did not follow"
    );
    set_current_params(None);
}
