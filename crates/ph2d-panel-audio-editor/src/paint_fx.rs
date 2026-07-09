//! The effects rack section of the Audio Editor panel (W3 block 3a).
//!
//! A selector (`◀ Name ⟲ ▶`) with a per-effect **Reset** icon beside the name,
//! one labelled slider per parameter of the selected effect (starting at its
//! neutral, no-op values), and **Apply / Cancel**.
//!
//! The rack **auditions live**: touching the selector or any slider marks it
//! dirty, and the shell renders the effect into the sounding preview so it is
//! heard (and drawn) while you tune. Apply commits exactly that buffer as one undo
//! step; Cancel throws it away.
//!
//! The panel is UI-only: sliders carry **normalized 0..1** values and the shell
//! publishes each slot's label + already-formatted value (`audio/fx_params.rs`),
//! so no DSP range or unit ever lands here.

use crate::paint::button;
use crate::{
    AEDIT_FX_APPLY, AEDIT_FX_CANCEL, AEDIT_FX_NEXT, AEDIT_FX_PARAMS, AEDIT_FX_PREV, AEDIT_FX_RESET,
    snapshot,
};
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::{paint_text_centered, resolve};
use ph2d_editor_core::widget::{
    ButtonState, IconButtonStyle, IconGlyph, Slider, SliderOrientation, paint_icon_button,
    paint_slider,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Width of the `◀` / `▶` selector arrows.
const ARROW_W: f32 = 26.0; // LITERAL-PX-OK: selector arrow button width (chrome)

/// Paint the rack starting at `y`; returns the `y` below it. `loaded` dims the
/// controls when there is no clip to act on.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_fx_section(
    mut y: f32,
    x: f32,
    w: f32,
    loaded: bool,
    row_h: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) -> f32 {
    let gap = Spacing::Sm.px();

    // Selector row: ◀ | effect name | reset | ▶
    button(
        Rect::new(x, y, ARROW_W, row_h),
        "\u{25c0}",
        loaded,
        AEDIT_FX_PREV,
        scene,
        text_system,
        theme,
        hit_index,
    );
    // Per-effect Reset: a frameless icon beside the name that puts THIS effect's
    // parameters back on their neutral defaults. Dimmed while they already are.
    let icon_w = row_h;
    let name_x = x + ARROW_W + gap;
    let reset_x = x + w - ARROW_W - gap - icon_w;
    let name_w = (reset_x - gap - name_x).max(1.0);
    paint_text_centered(
        text_system,
        scene,
        &snapshot::fx_kind_name(),
        Rect::new(name_x, y, name_w, row_h),
        TypeToken::Sm.px(),
        resolve(
            if loaded {
                ColorToken::Text1
            } else {
                ColorToken::Text2
            },
            theme,
        ),
    );
    let reset_rect = Rect::new(reset_x, y, icon_w, icon_w);
    let reset_state = if loaded && !snapshot::fx_at_defaults() {
        ButtonState::Normal
    } else {
        ButtonState::Disabled
    };
    paint_icon_button(
        reset_rect,
        IconGlyph::Builtin(IconId::Reset),
        IconButtonStyle::Plain,
        reset_state,
        scene,
        theme,
    );
    hit_index.register(AEDIT_FX_RESET, reset_rect);
    button(
        Rect::new(x + w - ARROW_W, y, ARROW_W, row_h),
        "\u{25b6}",
        loaded,
        AEDIT_FX_NEXT,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += row_h + Spacing::Md.px();

    // One row per parameter of the selected effect: `label ......... value`, with
    // the slider under it. Slots the effect doesn't use are simply not painted —
    // and not hit-registered, so a stale slider can't be grabbed.
    let views = snapshot::fx_param_views();
    let norms = snapshot::fx_norms();
    let label_h = TypeToken::Xs.px();
    let half = (w * 0.5).max(1.0);
    for (i, (label, value)) in views.iter().enumerate().take(AEDIT_FX_PARAMS.len()) {
        paint_text_centered(
            text_system,
            scene,
            label,
            Rect::new(x, y, half, label_h),
            TypeToken::Xs.px(),
            resolve(ColorToken::Text2, theme),
        );
        paint_text_centered(
            text_system,
            scene,
            value,
            Rect::new(x + half, y, half, label_h),
            TypeToken::Xs.px(),
            resolve(ColorToken::Text1, theme),
        );
        y += label_h + Spacing::Xs.px();

        let id = AEDIT_FX_PARAMS[i];
        let track = Rect::new(x, y, w, Spacing::Md.px());
        let mut slider = Slider::new(id, label.as_str()).orientation(SliderOrientation::Horizontal);
        slider.set_value(norms[i]);
        paint_slider(&slider, track, scene, theme);
        hit_index.register(id, track);
        y += Spacing::Md.px() + gap;
    }

    y += Spacing::Xs.px();

    // Live chain readout: effects you TUNED and then switched away from are stacked
    // under the active one, so the audition is `clip → stage₀ → … → active`.
    let chain = snapshot::fx_chain_len();
    if chain > 0 {
        let label = if chain == 1 {
            "Chain: 1 effect below".to_string()
        } else {
            format!("Chain: {chain} effects below")
        };
        paint_text_centered(
            text_system,
            scene,
            &label,
            Rect::new(x, y, w, TypeToken::Xs.px()),
            TypeToken::Xs.px(),
            resolve(ColorToken::Accent, theme),
        );
        y += TypeToken::Xs.px() + Spacing::Xs.px();
    }

    // The rack auditions live: touching the selector or a slider renders the
    // effect into the sounding preview without committing. Apply turns exactly the
    // buffer you heard (whole chain included) into one undo step; Cancel throws it
    // all away. Cancel is only meaningful while something is auditioning.
    let auditioning = snapshot::fx_auditioning();
    let btn_w = ((w - gap) * 0.5).max(1.0);
    button(
        Rect::new(x, y, btn_w, row_h),
        "Apply",
        loaded,
        AEDIT_FX_APPLY,
        scene,
        text_system,
        theme,
        hit_index,
    );
    button(
        Rect::new(x + btn_w + gap, y, btn_w, row_h),
        "Cancel",
        auditioning,
        AEDIT_FX_CANCEL,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y + row_h + gap
}
