//! The **Lighting** card — the canvas's light rig.
//!
//! ## Four lamps, one on screen
//!
//! Krita's Phong Bumpmap paints all four of its lights at once and ends up with **24 controls**;
//! `docs/Painter/17_impasto_deposito_pesquisa2.md` §2.4 files that under *"o conto-moral do excesso"*.
//! So the card edits the **selected** lamp — a four-chip selector, then that lamp's knobs — and its row
//! count does not grow with the rig. Four lamps, six rows.
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
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_i18n::tr;
use ph2d_tool_painter::BrushSettings;

/// The four lamp chips, in order. Their ids are flat constants (not a `light_id(i)` helper) because the
/// a11y tree and the arch-gates both want to see every widget spelled out.
const LAMP_IDS: [ph2d_a11y::NodeId; 4] = [
    ph2d_tool_painter::ids::PAINTER_IMPASTO_LIGHT_1,
    ph2d_tool_painter::ids::PAINTER_IMPASTO_LIGHT_2,
    ph2d_tool_painter::ids::PAINTER_IMPASTO_LIGHT_3,
    ph2d_tool_painter::ids::PAINTER_IMPASTO_LIGHT_4,
];

/// Rows 2..6 of the Lighting card: the lamp selector, then the SELECTED lamp's Angle / Elevation /
/// Intensity / Color. Returns the next `y`.
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
    let mut ry = crate::paint_seg_row::seg_row_owned(
        ctx,
        theme,
        x,
        w,
        y,
        ph2d_tool_painter::ids::PAINTER_IMPASTO_LIGHT_1,
        tr("panel.painter_layers.impasto.light"),
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
            ph2d_tool_painter::ids::PAINTER_IMPASTO_LIGHT_ON,
            "panel.painter_layers.impasto.enable",
            lamp.on,
        );
    }

    ry = card_row(
        ctx,
        theme,
        x,
        w,
        ry,
        "panel.painter_layers.impasto.angle",
        ph2d_tool_painter::ids::PAINTER_IMPASTO_LIGHT_ANGLE,
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
        "panel.painter_layers.impasto.elevation",
        ph2d_tool_painter::ids::PAINTER_IMPASTO_LIGHT_ELEV,
        f32::from(lamp.elev_deg),
        crate::paint_impasto::ELEV_MIN_DEG,
        crate::paint_impasto::ELEV_MAX_DEG,
        crate::paint_impasto::DEG_STEP,
        0,
    );

    // ── Row: Intensity, then the lamp's COLOUR on a row of its own.
    //
    //    ⛔ Until 2026-09-23 the colour was a 22 px square at the end of the Intensity row ("one thought:
    //    how much of what light — a row of its own would be a row spent on a square"). The owner's
    //    2026-09-21 report struck that very shape: every colour in the app is a BAR in the value column,
    //    named on the left. With a bar the row is no longer spent on a square — see `paint_brush_rows::color_row`.
    ry = card_row(
        ctx,
        theme,
        x,
        w,
        ry,
        "panel.painter_layers.impasto.intensity",
        ph2d_tool_painter::ids::PAINTER_IMPASTO_LIGHT_POWER,
        lamp.intensity,
        0.0,
        crate::paint_impasto::LIGHT_POWER_MAX,
        number_field::FINE_STEP,
        2,
    );
    let sw_id = ph2d_tool_painter::ids::PAINTER_IMPASTO_LIGHT_COLOR;
    let open = ctx.host.store().picker_target() == Some(sw_id);
    let enc = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit normalize
    register_button(ctx.host.store_mut(), sw_id);
    let after = crate::paint_brush_rows::color_row(
        ctx,
        theme,
        x,
        w,
        ry,
        "panel.painter_layers.impasto.light_color",
        sw_id,
        [enc(lamp.color[0]), enc(lamp.color[1]), enc(lamp.color[2])],
    );
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
