//! The **Lighting** card — the canvas's light rig.
//!
//! ## Four lamps, one on screen
//!
//! Krita's Phong Bumpmap paints all four of its lights at once and ends up with **24 controls**;
//! `docs/Painter/17_impasto_deposito_pesquisa2.md` §2.4 files that under *"o conto-moral do excesso"*.
//! So the card edits the **selected** lamp — a four-chip selector, then that lamp's knobs — and its row
//! count does not grow with the rig. Four lamps, five rows.
//!
//! Lamps 2-4 are **off** by default, which is what keeps a canvas nobody has opened the rig on
//! byte-identical to the single-light build.
//!
//! **Shine is global** and lives in the card's last row: it is a property of the PAINT (how wet the oil
//! is), not of a lamp. Giving every light its own specular strength is four knobs for one material.

use crate::card::card_row;
use crate::number_field;
use crate::paint::register_button;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::{ColorSwatch, SwatchSize, SwatchState, paint_color_swatch};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};
use ph2d_tool_painter::BrushSettings;

/// The four lamp chips, in order. Their ids are flat constants (not a `light_id(i)` helper) because the
/// a11y tree and the arch-gates both want to see every widget spelled out.
const LAMP_IDS: [ph2d_a11y::NodeId; 4] = [
    core_ids::PAINTER_IMPASTO_LIGHT_1,
    core_ids::PAINTER_IMPASTO_LIGHT_2,
    core_ids::PAINTER_IMPASTO_LIGHT_3,
    core_ids::PAINTER_IMPASTO_LIGHT_4,
];

/// Width of the colour swatch at the end of the Intensity row.
const SWATCH_W: f32 = 28.0; // LITERAL-PX-OK: swatch box, sized to the row height

/// Rows 2..5 of the Lighting card: the lamp selector, then the SELECTED lamp's Angle / Elevation /
/// Intensity + colour. Returns the next `y`.
pub(crate) fn paint_light_rows(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let rig = &brush.impasto_rig;
    let sel = (rig.selected as usize).min(LAMP_IDS.len() - 1);
    let lamp = rig.lights[sel];

    // ── Row: the lamp selector. A lamp that is OFF says so in its own chip — an artist scanning the rig
    //    must not have to click through four lamps to find out which are lit.
    let mut ry = crate::paint_impasto::seg_row_owned(
        ctx,
        theme,
        x,
        w,
        y,
        core_ids::PAINTER_IMPASTO_LIGHT_1,
        "Light",
        &LAMP_IDS
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let on = rig.lights[i].on && rig.lights[i].intensity > 0.0;
                (
                    *id,
                    if on {
                        format!("{}", i + 1)
                    } else {
                        format!("{}·", i + 1)
                    },
                )
            })
            .collect::<Vec<_>>(),
        sel,
    );

    // ── Row: Enable — but ONLY for lamps 2-4. The key cannot be switched off (Show Impasto already IS
    //    that switch), and a dimmed checkbox is not the answer: a dimmed control still hit-registers, and
    //    the rule of this house is that a control which does not apply is NOT PAINTED.
    if sel > 0 {
        ry = crate::paint_brush_top::paint_checkbox_row(
            ctx,
            theme,
            x,
            w,
            ry,
            core_ids::PAINTER_IMPASTO_LIGHT_ON,
            "Enable",
            lamp.on,
        );
    }

    ry = card_row(
        ctx,
        theme,
        x,
        w,
        ry,
        "Angle",
        core_ids::PAINTER_IMPASTO_LIGHT_ANGLE,
        f32::from(lamp.angle_deg),
        0.0,
        crate::paint_impasto::ANGLE_MAX_DEG,
        crate::paint_impasto::DEG_STEP,
        0,
    );
    ry = card_row(
        ctx,
        theme,
        x,
        w,
        ry,
        "Elevation",
        core_ids::PAINTER_IMPASTO_LIGHT_ELEV,
        f32::from(lamp.elev_deg),
        crate::paint_impasto::ELEV_MIN_DEG,
        crate::paint_impasto::ELEV_MAX_DEG,
        crate::paint_impasto::DEG_STEP,
        0,
    );

    // ── Row: Intensity + the lamp's COLOUR swatch, side by side. The two are one thought ("how much of
    //    what light"), and the swatch is small — a row of its own would be a row spent on a square.
    let box_w = w - SWATCH_W - Spacing::Xs.px();
    let after = card_row(
        ctx,
        theme,
        x,
        box_w,
        ry,
        "Intensity",
        core_ids::PAINTER_IMPASTO_LIGHT_POWER,
        lamp.intensity,
        0.0,
        crate::paint_impasto::LIGHT_POWER_MAX,
        number_field::FINE_STEP,
        2,
    );
    let sw_id = core_ids::PAINTER_IMPASTO_LIGHT_COLOR;
    let sr = Rect::new(x + box_w + Spacing::Xs.px(), ry, SWATCH_W, ROW_H_PX);
    let open = ctx.host.store().picker_target() == Some(sw_id);
    let enc = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit normalize
    paint_color_swatch(
        &ColorSwatch {
            id: sw_id,
            label: String::new(),
            rgba: [
                enc(lamp.color[0]),
                enc(lamp.color[1]),
                enc(lamp.color[2]),
                255,
            ],
            state: if open {
                SwatchState::Focused
            } else {
                SwatchState::Normal
            },
            size: SwatchSize::Sm,
        },
        sr,
        ctx.scene,
        theme,
    );
    register_button(ctx.host.store_mut(), sw_id);
    ctx.host.hit_index_mut().register(sw_id, sr);
    // Read-back: the shared picker writes the pick onto the swatch's widget colour; forward it to the
    // tool ONLY when it actually differs, or every frame with the picker open would be an undo step.
    if open
        && let Some(picked) = ctx.host.store().widget_color(sw_id)
        && [enc(lamp.color[0]), enc(lamp.color[1]), enc(lamp.color[2])]
            != [picked[0], picked[1], picked[2]]
    {
        ctx.host
            .bus_mut()
            .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                sw_id,
                format!("{},{},{}", picked[0], picked[1], picked[2]),
            )));
    }
    after
}
