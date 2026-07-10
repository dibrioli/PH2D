//! The effects rack section of the Audio Editor panel (W3 blocks 3a/3b).
//!
//! A **chain** of effects, rendered in order: `clip → stage₀ → … → stageₙ`. One
//! stage is selected; the selector (`◀ Name ⟲ ▶`) sets its kind and the sliders
//! tune it. Below, the chain list shows every stage with an eye toggle (per-stage
//! bypass), and the action row adds / removes / reorders the selected one.
//!
//! The rack **auditions live**: any edit marks it dirty and the shell renders the
//! whole chain into the sounding preview, so it is heard (and drawn) while you
//! tune. **Bypass** swaps the dry clip back in without losing the chain — the A/B.
//! **Apply** commits exactly that buffer as one undo step; **Cancel** throws it away.
//!
//! The panel is UI-only: the chain carries **normalized 0..1** slider values and a
//! kind index, and the shell publishes each slot's label + already-formatted value
//! (`audio/fx_params.rs`), so no DSP range or unit ever lands here.

use crate::paint::{ClippedHits, button, toggle};
use crate::{
    AEDIT_FX_ADD, AEDIT_FX_APPLY, AEDIT_FX_BYPASS, AEDIT_FX_CANCEL, AEDIT_FX_DOWN, AEDIT_FX_NEXT,
    AEDIT_FX_PARAMS, AEDIT_FX_PREV, AEDIT_FX_REMOVE, AEDIT_FX_RESET, AEDIT_FX_STAGE_ONS,
    AEDIT_FX_STAGES, AEDIT_FX_UP, AEDIT_PRESET_APPLY, AEDIT_PRESET_LOAD, AEDIT_PRESET_NEXT,
    AEDIT_PRESET_PREV, AEDIT_PRESET_SAVE, MAX_FX_STAGES, presets, snapshot,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, paint_text_centered, resolve};
use ph2d_editor_core::widget::{
    ButtonState, IconButtonStyle, IconGlyph, Slider, SliderOrientation, paint_icon_button,
    paint_slider,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Width of the `◀` / `▶` selector arrows.
const ARROW_W: f32 = 26.0; // LITERAL-PX-OK: selector arrow button width (chrome)
/// Buttons in the chain action row (Add · Remove · Up · Down).
const ACTION_BUTTONS: f32 = 4.0; // LITERAL-PX-OK: fixed count, divides the row width
/// Buttons in the preset action row (Apply · Save · Load).
const PRESET_BUTTONS: f32 = 3.0; // LITERAL-PX-OK: fixed count, divides the row width

/// The painter's shared borrows, bundled so each section fits one argument list.
struct Ctx<'a, 'h> {
    scene: &'a mut VectorScene,
    text_system: &'a mut TextSystem,
    theme: Theme,
    hit_index: &'a mut ClippedHits<'h>,
}

/// Paint the rack starting at `y`; returns the `y` below it. `loaded` dims the
/// controls when there is no clip to act on.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_fx_section(
    y: f32,
    x: f32,
    w: f32,
    loaded: bool,
    row_h: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ClippedHits,
) -> f32 {
    let ctx = &mut Ctx {
        scene,
        text_system,
        theme,
        hit_index,
    };
    let y = paint_presets(y, x, w, loaded, row_h, ctx);
    let y = paint_selector(y, x, w, loaded, row_h, ctx);
    let y = paint_params(y, x, w, loaded, ctx);
    let y = paint_chain(y, x, w, loaded, row_h, ctx);
    paint_commit_row(y, x, w, loaded, row_h, ctx)
}

/// The preset row: `◀ Factory preset ▶` over `Apply · Save · Load`. Apply loads the
/// selected factory preset into the chain (it auditions at once); Save / Load are
/// user preset **files** via a native dialog — the OS browser is their picker, so no
/// in-panel list is needed. Sits above the effect selector: a preset is a starting
/// point you then tune.
fn paint_presets(mut y: f32, x: f32, w: f32, loaded: bool, row_h: f32, ctx: &mut Ctx) -> f32 {
    let gap = Spacing::Sm.px();
    let has_presets = presets::preset_count() > 0;
    button(
        Rect::new(x, y, ARROW_W, row_h),
        "\u{25c0}",
        has_presets,
        AEDIT_PRESET_PREV,
        ctx.scene,
        ctx.text_system,
        ctx.theme,
        ctx.hit_index,
    );
    let name_x = x + ARROW_W + gap;
    let name_w = (w - 2.0 * (ARROW_W + gap)).max(1.0);
    paint_text_centered(
        ctx.text_system,
        ctx.scene,
        &presets::preset_name(),
        Rect::new(name_x, y, name_w, row_h),
        TypeToken::Sm.px(),
        resolve(text_tone(has_presets), ctx.theme),
    );
    button(
        Rect::new(x + w - ARROW_W, y, ARROW_W, row_h),
        "\u{25b6}",
        has_presets,
        AEDIT_PRESET_NEXT,
        ctx.scene,
        ctx.text_system,
        ctx.theme,
        ctx.hit_index,
    );
    y += row_h + gap;

    // Apply (factory) · Save · Load (files). Apply needs a preset to load; Save/Load
    // need a clip loaded, like every other file action.
    let bw = ((w - gap * (PRESET_BUTTONS - 1.0)) / PRESET_BUTTONS).max(1.0);
    for (i, (label, enabled, id)) in [
        ("Apply", has_presets, AEDIT_PRESET_APPLY),
        ("Save", loaded, AEDIT_PRESET_SAVE),
        ("Load", loaded, AEDIT_PRESET_LOAD),
    ]
    .into_iter()
    .enumerate()
    {
        button(
            Rect::new(x + (bw + gap) * i as f32, y, bw, row_h),
            label,
            enabled,
            id,
            ctx.scene,
            ctx.text_system,
            ctx.theme,
            ctx.hit_index,
        );
    }
    y + row_h + Spacing::Md.px()
}

/// `◀ | effect name | ⟲ | ▶` — sets the SELECTED stage's kind. The Reset icon is
/// frameless, sits beside the name, and puts that stage's parameters back on their
/// neutral defaults. It is dimmed while they already are.
fn paint_selector(mut y: f32, x: f32, w: f32, loaded: bool, row_h: f32, ctx: &mut Ctx) -> f32 {
    let gap = Spacing::Sm.px();
    button(
        Rect::new(x, y, ARROW_W, row_h),
        "\u{25c0}",
        loaded,
        AEDIT_FX_PREV,
        ctx.scene,
        ctx.text_system,
        ctx.theme,
        ctx.hit_index,
    );
    let icon_w = row_h;
    let name_x = x + ARROW_W + gap;
    let reset_x = x + w - ARROW_W - gap - icon_w;
    let name_w = (reset_x - gap - name_x).max(1.0);
    let (kind, _) = snapshot::fx_sel_stage();
    paint_text_centered(
        ctx.text_system,
        ctx.scene,
        &snapshot::fx_kind_name(kind),
        Rect::new(name_x, y, name_w, row_h),
        TypeToken::Sm.px(),
        resolve(text_tone(loaded), ctx.theme),
    );
    icon_button(
        Rect::new(reset_x, y, icon_w, icon_w),
        IconId::Reset,
        loaded && !snapshot::fx_at_defaults(),
        AEDIT_FX_RESET,
        ctx,
    );
    button(
        Rect::new(x + w - ARROW_W, y, ARROW_W, row_h),
        "\u{25b6}",
        loaded,
        AEDIT_FX_NEXT,
        ctx.scene,
        ctx.text_system,
        ctx.theme,
        ctx.hit_index,
    );
    y += row_h + Spacing::Md.px();
    y
}

/// One row per parameter of the selected stage: `label ......... value`, with the
/// slider under it. Slots the effect doesn't use are simply not painted — and not
/// hit-registered, so a stale slider can't be grabbed.
fn paint_params(mut y: f32, x: f32, w: f32, loaded: bool, ctx: &mut Ctx) -> f32 {
    let gap = Spacing::Sm.px();
    let views = snapshot::fx_param_views();
    let norms = snapshot::fx_norms();
    let label_h = TypeToken::Xs.px();
    let half = (w * 0.5).max(1.0);
    for (i, (label, value)) in views.iter().enumerate().take(AEDIT_FX_PARAMS.len()) {
        paint_text_centered(
            ctx.text_system,
            ctx.scene,
            label,
            Rect::new(x, y, half, label_h),
            TypeToken::Xs.px(),
            resolve(ColorToken::Text2, ctx.theme),
        );
        paint_text_centered(
            ctx.text_system,
            ctx.scene,
            value,
            Rect::new(x + half, y, half, label_h),
            TypeToken::Xs.px(),
            resolve(text_tone(loaded), ctx.theme),
        );
        y += label_h + Spacing::Xs.px();

        let id = AEDIT_FX_PARAMS[i];
        let track = Rect::new(x, y, w, Spacing::Md.px());
        let mut slider = Slider::new(id, label.as_str()).orientation(SliderOrientation::Horizontal);
        slider.set_value(norms[i]);
        paint_slider(&slider, track, ctx.scene, ctx.theme);
        if loaded {
            ctx.hit_index.register(id, track);
        }
        y += Spacing::Md.px() + gap;
    }
    y + Spacing::Xs.px()
}

/// The chain: a header carrying the `+ | trash | ▲ | ▼` actions, then one row per
/// stage in render order. Clicking a row selects it (the selector + sliders follow);
/// the eye toggles it in and out of the render without dropping it — the per-stage
/// half of the A/B. The actions all act on the SELECTED stage. Order matters: a
/// filter before a reverb is not the same as after.
///
/// The panel does not scroll, so the actions ride the header rather than claiming a
/// row of their own.
fn paint_chain(mut y: f32, x: f32, w: f32, loaded: bool, row_h: f32, ctx: &mut Ctx) -> f32 {
    let stage_h = TypeToken::Sm.px() + Spacing::Sm.px();
    let count = snapshot::fx_stage_count();
    let sel = snapshot::fx_sel();

    let label_h = TypeToken::Xs.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        "Chain",
        x,
        y + (row_h - label_h) * 0.5,
        label_h,
        w,
        resolve(ColorToken::Text2, ctx.theme),
    );
    let actions = [
        (IconId::Add, loaded && count < MAX_FX_STAGES, AEDIT_FX_ADD),
        (IconId::Trash, loaded && count > 1, AEDIT_FX_REMOVE),
        (IconId::ChevronUp, loaded && sel > 0, AEDIT_FX_UP),
        (
            IconId::ChevronDown,
            loaded && sel + 1 < count,
            AEDIT_FX_DOWN,
        ),
    ];
    // Right-aligned, so the header reads `Chain ........ + 🗑 ▲ ▼`.
    let actions_x = x + w - row_h * ACTION_BUTTONS;
    for (i, (glyph, enabled, id)) in actions.into_iter().enumerate() {
        let rect = Rect::new(actions_x + row_h * i as f32, y, row_h, row_h);
        icon_button(rect, glyph, enabled, id, ctx);
    }
    y += row_h + Spacing::Xs.px();

    for i in 0..count.min(MAX_FX_STAGES) {
        let Some((name, enabled)) = snapshot::fx_stage_view(i) else {
            continue;
        };
        let row = Rect::new(x, y, w, stage_h);
        if i == sel {
            fill_rounded_rect(
                ctx.scene,
                row,
                Radius::Sm.px(),
                resolve(ColorToken::Bg3, ctx.theme),
            );
        }
        // The eye sits at the row's right edge and swallows its own clicks; the
        // rest of the row selects. Register the row FIRST so the eye's rect, which
        // is registered after, wins the overlap.
        if loaded {
            ctx.hit_index.register(AEDIT_FX_STAGES[i], row);
        }
        let tone = if !enabled {
            ColorToken::Text2
        } else if i == sel {
            ColorToken::Accent
        } else {
            text_tone(loaded)
        };
        let fs = TypeToken::Xs.px();
        paint_text(
            ctx.text_system,
            ctx.scene,
            &format!("{}. {name}", i + 1),
            x + Spacing::Sm.px(),
            y + (stage_h - fs) * 0.5,
            fs,
            w,
            resolve(tone, ctx.theme),
        );
        let eye = Rect::new(x + w - stage_h, y, stage_h, stage_h);
        let glyph = if enabled {
            IconId::Eye
        } else {
            IconId::EyeClosed
        };
        icon_button(eye, glyph, loaded, AEDIT_FX_STAGE_ONS[i], ctx);
        y += stage_h;
    }
    y + Spacing::Sm.px()
}

/// `Bypass` (global A/B) over `Apply | Cancel`. Bypass mutes the whole chain so the
/// dry clip sounds and shows, without losing it — the fastest before/after there is.
/// Apply turns exactly the buffer you heard into one undo step; Cancel drops it.
/// Both are only meaningful while something is auditioning.
fn paint_commit_row(mut y: f32, x: f32, w: f32, loaded: bool, row_h: f32, ctx: &mut Ctx) -> f32 {
    let gap = Spacing::Sm.px();
    let auditioning = snapshot::fx_auditioning();
    let bypassed = snapshot::fx_bypass();
    toggle(
        Rect::new(x, y, w, row_h),
        "Bypass",
        bypassed,
        auditioning,
        AEDIT_FX_BYPASS,
        ctx.scene,
        ctx.text_system,
        ctx.theme,
        ctx.hit_index,
    );
    y += row_h + gap;

    // Apply is dimmed while bypassed: what sounds is the dry clip, so committing
    // would land nothing. Release Bypass to commit what the chain does.
    let btn_w = ((w - gap) * 0.5).max(1.0);
    button(
        Rect::new(x, y, btn_w, row_h),
        "Apply",
        loaded && !bypassed,
        AEDIT_FX_APPLY,
        ctx.scene,
        ctx.text_system,
        ctx.theme,
        ctx.hit_index,
    );
    button(
        Rect::new(x + btn_w + gap, y, btn_w, row_h),
        "Cancel",
        auditioning,
        AEDIT_FX_CANCEL,
        ctx.scene,
        ctx.text_system,
        ctx.theme,
        ctx.hit_index,
    );
    y + row_h + gap
}

/// A frameless icon button. Disabled ones are dimmed and — crucially — do **not**
/// register a hit rect, so they cannot be clicked (2026-07-09 audit).
fn icon_button(rect: Rect, glyph: IconId, enabled: bool, id: NodeId, ctx: &mut Ctx) {
    let state = if enabled {
        ButtonState::Normal
    } else {
        ButtonState::Disabled
    };
    paint_icon_button(
        rect,
        IconGlyph::Builtin(glyph),
        IconButtonStyle::Plain,
        state,
        ctx.scene,
        ctx.theme,
    );
    if enabled {
        ctx.hit_index.register(id, rect);
    }
}

/// Foreground tone for text that dims with the clip's presence.
fn text_tone(loaded: bool) -> ColorToken {
    if loaded {
        ColorToken::Text1
    } else {
        ColorToken::Text2
    }
}
