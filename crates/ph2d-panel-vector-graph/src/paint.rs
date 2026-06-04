//! Geometry-Graph panel paint — chrome + 8 labeled slider rows.
//!
//! Each row pairs a left label column (the param name), a normalized
//! `0..1` horizontal [`Slider`] in the middle, and a right-pinned value
//! readout column showing the param's current REAL value (track mapped
//! through [`crate::state::ParamSpec::track_to_value`]): `width` as
//! integer px, `inner_ratio` as 2 decimals, `kind` as the variant name
//! (Rect / Ellipse / Polygon / Star / Spiral), the rest as sensible
//! integers / decimals.
//!
//! HR-15: zero hex, zero UI `f32` literal, zero hardcoded UI string except
//! the English labels (app-UI-english-only convention — matches the other
//! panels). All colors / spacing / sizes resolve through `ph2d_tokens`.

use crate::VectorGraphPanel;
use crate::state::{KIND_NAMES, PARAMS, set_last_content_h, set_last_visible_h};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_surface,
    paint_panel_title,
};
use ph2d_editor_core::widget::{Slider, SliderState, paint_slider};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

/// Left label-column width (the param name gutter).
const LABEL_COL_W: f32 = 96.0; // LITERAL-PX-OK: panel grid metric (param-name label gutter)
/// Right value-readout column width.
const VALUE_COL_W: f32 = 56.0; // LITERAL-PX-OK: panel grid metric (numeric value readout column)

pub(crate) fn paint(_state: &mut crate::VectorGraphPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(VectorGraphPanel::ID) {
        ctx.host
            .store_mut()
            .clear_panel_rect(core_ids::VGRAPH_PANEL);
        set_last_content_h(0.0);
        set_last_visible_h(0.0);
        return;
    }

    let rect: Rect = ctx.layout.inspector;
    let theme = ctx.host.theme();
    ctx.host
        .store_mut()
        .set_panel_rect(core_ids::VGRAPH_PANEL, rect);

    // Chrome — dark-glass surface + canonical title.
    paint_panel_surface(rect, ctx.scene, theme);
    let title_size = paint_panel_title(
        rect,
        "Geometry Graph",
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );

    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let inner_x = rect.x + PANEL_HEAD_PAD;
    let inner_w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    let font = TypeToken::Base.px();
    let row_h = ROW_H_PX;
    let row_gap = Spacing::Sm.px();
    let col_gap = Spacing::Sm.px();
    let text_color = resolve(ColorToken::Text1, theme);
    let value_color = resolve(ColorToken::Text2, theme);

    let mut y = body_top;
    for spec in PARAMS {
        // Live track ?? default-seeded track (so the readout is exact even
        // before the first repaint after a drag).
        let track = ctx
            .host
            .store()
            .slider(spec.id)
            .map(|(_, v)| v)
            .unwrap_or_else(|| spec.value_to_track(spec.default));
        let real = spec.track_to_value(track);
        let slider_state = ctx
            .host
            .store()
            .slider(spec.id)
            .map(|(s, _)| s)
            .unwrap_or(SliderState::Normal);

        let text_y = y + (row_h - font) * 0.5;

        // Label column.
        paint_text(
            ctx.text_system,
            ctx.scene,
            spec.label,
            inner_x,
            text_y,
            font,
            LABEL_COL_W,
            text_color,
        );

        // Slider column (between label + value).
        let slider_x = inner_x + LABEL_COL_W + col_gap;
        let slider_w = (inner_w - LABEL_COL_W - VALUE_COL_W - col_gap * 2.0).max(0.0);
        let slider_rect = Rect::new(slider_x, y, slider_w, row_h);
        let slider = Slider::new(spec.id, spec.label)
            .state(slider_state)
            .accent(true);
        let slider = set_track(slider, track);
        paint_slider(&slider, slider_rect, ctx.scene, theme);
        ctx.host.hit_index_mut().register(spec.id, slider_rect);

        // Value-readout column (right-pinned).
        let value_x = inner_x + inner_w - VALUE_COL_W;
        let value_text = format_value(spec.id, real);
        paint_text(
            ctx.text_system,
            ctx.scene,
            &value_text,
            value_x,
            text_y,
            font,
            VALUE_COL_W,
            value_color,
        );

        y += row_h + row_gap;
    }

    let content_h = (y - body_top + PANEL_HEAD_PAD).max(0.0);
    set_last_content_h(content_h);
    set_last_visible_h(body_h);
}

/// Apply a normalized `0..1` `track` to a freshly-built [`Slider`] via its
/// clamping setter (keeps the value in range).
fn set_track(mut slider: Slider, track: f32) -> Slider {
    slider.set_value(track);
    slider
}

/// Format a param's real `value` for the right-column readout.
fn format_value(id: NodeId, value: f32) -> String {
    use crate::ids;
    if id == ids::VGRAPH_KIND {
        let idx = (value.round() as i64).clamp(0, KIND_NAMES.len() as i64 - 1) as usize; // CLAMP-OK: integer clamp, no NaN; 0..=len-1 valid (KIND_NAMES non-empty)
        KIND_NAMES[idx].to_string()
    } else if id == ids::VGRAPH_INNER_RATIO {
        format!("{value:.2}")
    } else if id == ids::VGRAPH_ROTATION {
        // Radians — show 2 decimals (range 0..2π).
        format!("{value:.2}")
    } else {
        // Width / Height / Sides / Turns / Samples — integer readout.
        format!("{}", value.round() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids;

    #[test]
    fn kind_value_shows_variant_name() {
        assert_eq!(format_value(ids::VGRAPH_KIND, 0.0), "Rect");
        assert_eq!(format_value(ids::VGRAPH_KIND, 1.4), "Ellipse");
        assert_eq!(format_value(ids::VGRAPH_KIND, 4.0), "Spiral");
        // Out-of-range rounds clamp into the table.
        assert_eq!(format_value(ids::VGRAPH_KIND, 9.0), "Spiral");
    }

    #[test]
    fn ratio_shows_two_decimals_others_integer() {
        assert_eq!(format_value(ids::VGRAPH_INNER_RATIO, 0.4), "0.40");
        assert_eq!(format_value(ids::VGRAPH_WIDTH, 100.0), "100");
        assert_eq!(format_value(ids::VGRAPH_SIDES, 6.0), "6");
        assert_eq!(
            format_value(ids::VGRAPH_ROTATION, std::f32::consts::TAU),
            "6.28"
        );
    }
}
