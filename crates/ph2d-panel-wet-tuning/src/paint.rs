//! The panel body. Walks [`crate::rows`] — the same table `populate` and
//! `event` walk — so what is drawn, registered and dispatched cannot
//! disagree. The rect derives from the inspector slot: same height, docked
//! immediately to its LEFT ("na lateral do painel do painter").

use ph2d_editor_core::ids;
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::widget::{
    ButtonState, Checkbox, CheckboxValue, IconButtonStyle, IconGlyph, SectionHeader,
    WET_TUNING_SCROLLBAR_ID, paint_checkbox, paint_icon_button, paint_scrollbar,
    paint_section_header, paint_slider_with_chip_layout_adaptive, scrollbar_is_needed,
    scrollbar_thumb_rect, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};
use ph2d_tool_painter::BrushSettings;

use crate::state::{self, WetTuningPanelState};
use crate::{WetTuningPanel, rows};

/// Label gutter — knob names are words ("Gate saturation"), same width the
/// physics panel settled on.
pub(crate) const LABEL_COL_W: f32 = 78.0; // LITERAL-PX-OK: panel grid metric (label gutter width)
/// Width reserved at a row's right edge for its per-knob reset button.
const RESET_W: f32 = 22.0; // LITERAL-PX-OK: per-knob reset button slot width

pub(crate) fn paint(_state: &mut WetTuningPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(WetTuningPanel::ID) {
        ctx.host.store_mut().clear_panel_rect(ids::WET_TUNING_PANEL);
        return;
    }
    let insp: Rect = ctx.layout.inspector;
    let gap = Spacing::Xs.px();
    let rect = Rect::new((insp.x - insp.w - gap).max(0.0), insp.y, insp.w, insp.h);
    let theme = ctx.host.theme();
    let brush = state::current();

    ctx.host
        .store_mut()
        .set_panel_rect(ids::WET_TUNING_PANEL, rect);
    paint_panel_surface(rect, ctx.scene, theme);
    let title_size = paint_panel_title(
        rect,
        tr("panel.wet_tuning.title"),
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    // The close button forwards the SAME Tuning toggle the basic section
    // owns (visibility is the tool's authored fact; the bridge mirrors it).
    paint_panel_close_button(
        rect,
        ids::WET_TUNING_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    let scroll = ctx.host.store().panel_scroll(ids::WET_TUNING_PANEL);

    ctx.scene.push_clip(&rect_to_vello(body_rect));
    let y_after = paint_body(
        ctx,
        &brush,
        rect.x + PANEL_HEAD_PAD,
        (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0),
        body_top - scroll,
        theme,
    );
    let content_h = (y_after + scroll) - body_top + PANEL_HEAD_PAD;
    ctx.scene.pop_layer();

    paint_scrollbar_and_publish(ctx, body_rect, content_h, body_h, scroll, theme);
}

fn paint_body(
    ctx: &mut PaintCtx,
    brush: &BrushSettings,
    x: f32,
    w: f32,
    mut y: f32,
    theme: Theme,
) -> f32 {
    // While the artist's Paper SLOT drives the tooth, the engine's own tile
    // is not the source — its three physical knobs hide (lei 3: a knob that
    // does nothing is a dead control wearing a live one's clothes).
    let artist_paper_armed = brush.paper_kind != 0;
    for section in rows::SECTIONS {
        let open = !ctx.host.store().is_collapsed(section.header);
        y = header_row(
            ctx,
            theme,
            x,
            w,
            y,
            section.header,
            section.reset,
            {
                // The PAPER header carries the eye — the paperVisibility
                // master's on/off face (the same authored fact as the basic
                // section's Paper checkbox).
                matches!(section.group, ph2d_wet_paint::tuning::KnobGroup::Paper)
                    .then_some((ids::WET_TUNING_PAPER_EYE, brush.wet_paper_visual))
            },
            tr(section.label),
            open,
        );
        if !open {
            continue;
        }
        for row in rows::rows().iter().filter(|r| r.group == section.group) {
            if artist_paper_armed && rows::is_engine_paper_physical(row.key) {
                continue;
            }
            let value = brush.wet_knobs.knobs[row.knob];
            y = paint_row(ctx, row, value, x, w, y, theme);
        }
    }
    // EXPERIMENTAL — the two K–M checkboxes + the note.
    y = header_row(
        ctx,
        theme,
        x,
        w,
        y,
        ids::WET_TUNING_GROUP_HEADERS[5],
        ids::WET_TUNING_GROUP_HEADERS[5], // no reset: the reset slot repeats the header (skipped)
        None,
        tr("panel.wet_tuning.group.experimental"),
        !ctx.host
            .store()
            .is_collapsed(ids::WET_TUNING_GROUP_HEADERS[5]),
    );
    if !ctx
        .host
        .store()
        .is_collapsed(ids::WET_TUNING_GROUP_HEADERS[5])
    {
        y = checkbox_row(
            ctx,
            theme,
            x,
            w,
            y,
            ids::WET_TUNING_KM_MIXING,
            tr("panel.wet_tuning.km_mixing"),
            brush.wet_km_mixing,
        );
        y = checkbox_row(
            ctx,
            theme,
            x,
            w,
            y,
            ids::WET_TUNING_KM_GLAZE,
            tr("panel.wet_tuning.km_glaze"),
            brush.wet_km_glaze,
        );
        y = note_text(ctx, theme, x, w, y, tr("panel.wet_tuning.note"));
    }
    y
}

/// One slider+chip row from the table, with its per-knob reset at the right.
fn paint_row(
    ctx: &mut PaintCtx,
    row: &rows::TuneRow,
    value: f64,
    x: f32,
    w: f32,
    y: f32,
    theme: Theme,
) -> f32 {
    let row_w = (w - RESET_W - Spacing::Xs.px()).max(0.0);
    let track = row.track_of(value);
    let text = format!("{value:.*}", row.decimals);
    let label = tr(row.label.as_str());
    // Returns the row HEIGHT used (the physics panel's convention).
    let used = {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_slider_with_chip_layout_adaptive(
            Rect::new(x, y, row_w, ROW_H_PX),
            label,
            track,
            value,
            Some(&text),
            row.slider,
            row.chip,
            LABEL_COL_W,
            ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX,
            store,
            hit_index,
            scene,
            text_system,
            theme,
        )
    };
    let reset_rect = Rect::new(
        x + w - RESET_W,
        y,
        RESET_W,
        ROW_H_PX.min(used.max(ROW_H_PX)),
    );
    paint_icon_button(
        reset_rect,
        IconGlyph::Builtin(ph2d_editor_core::IconId::Reset),
        IconButtonStyle::Plain,
        ButtonState::Normal,
        ctx.scene,
        theme,
    );
    ctx.host.hit_index_mut().register(row.reset, reset_rect);
    y + used + Spacing::Xs.px()
}

/// Section header: chevron+label (collapse toggles on click), the group
/// reset at the right, and — for PAPER — the visibility eye beside it.
#[allow(clippy::too_many_arguments)]
fn header_row(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    header: ph2d_a11y::NodeId,
    reset: ph2d_a11y::NodeId,
    eye: Option<(ph2d_a11y::NodeId, bool)>,
    label: &str,
    open: bool,
) -> f32 {
    let rect = Rect::new(x, y, w, ROW_H_PX);
    let hdr = SectionHeader::new(header, label).collapsible(open);
    paint_section_header(&hdr, rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(header, rect);
    let mut right = x + w - RESET_W;
    if reset != header {
        let r = Rect::new(right, y, RESET_W, ROW_H_PX);
        paint_icon_button(
            r,
            IconGlyph::Builtin(ph2d_editor_core::IconId::Reset),
            IconButtonStyle::Plain,
            ButtonState::Normal,
            ctx.scene,
            theme,
        );
        ctx.host.hit_index_mut().register(reset, r);
        right -= RESET_W + Spacing::Xs.px();
    }
    if let Some((eye_id, on)) = eye {
        let r = Rect::new(right, y, RESET_W, ROW_H_PX);
        paint_icon_button(
            r,
            IconGlyph::Builtin(if on {
                ph2d_editor_core::IconId::Eye
            } else {
                ph2d_editor_core::IconId::EyeClosed
            }),
            IconButtonStyle::Plain,
            ButtonState::Normal,
            ctx.scene,
            theme,
        );
        ctx.host.hit_index_mut().register(eye_id, r);
    }
    y + ROW_H_PX + Spacing::Xs.px()
}

fn checkbox_row(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    label: &str,
    on: bool,
) -> f32 {
    let rect = Rect::new(x, y, w, ROW_H_PX);
    let cb = Checkbox::new(id, label).value(if on {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    });
    paint_checkbox(&cb, rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(id, rect);
    y + ROW_H_PX + Spacing::Xs.px()
}

/// The Experimental card's explanatory note — short lines, wrapped by hand
/// (the note is two fixed sentences; `paint_text` clips at `max_width`, so
/// the wrap estimate only decides where the second line starts).
fn note_text(ctx: &mut PaintCtx, theme: Theme, x: f32, w: f32, y: f32, note: &str) -> f32 {
    let color = ph2d_editor_core::paint::resolve(ph2d_tokens::ColorToken::Text2, theme);
    let size = ph2d_tokens::TypeToken::Xs.px();
    let line_h = ROW_H_PX * 0.7; // LITERAL-PX-OK: caption line height fraction of a row
    let approx_chars = ((w / (size * 0.52)).max(8.0)) as usize; // LITERAL-PX-OK: mean glyph advance fraction for wrap estimate
    let mut cur = String::new();
    let mut yy = y;
    let emit = |line: &str, yy: &mut f32, ctx: &mut PaintCtx| {
        ph2d_editor_core::paint::paint_text(
            ctx.text_system,
            ctx.scene,
            line,
            x,
            *yy + line_h,
            size,
            w,
            color,
        );
        *yy += line_h;
    };
    for word in note.split_whitespace() {
        if !cur.is_empty() && cur.len() + 1 + word.len() > approx_chars {
            emit(&cur, &mut yy, ctx);
            cur.clear();
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(word);
    }
    if !cur.is_empty() {
        emit(&cur, &mut yy, ctx);
    }
    yy + Spacing::Xs.px()
}

fn paint_scrollbar_and_publish(
    ctx: &mut PaintCtx,
    body_rect: Rect,
    content_h: f32,
    body_h: f32,
    scroll: f32,
    theme: Theme,
) {
    if scrollbar_is_needed(content_h, body_h) {
        let track = scrollbar_track_rect(body_rect);
        let thumb = scrollbar_thumb_rect(track, scroll, content_h, body_h);
        let is_active = matches!(
            ctx.host.store().scrollbar_drag(),
            Some(d) if d.panel == ids::WET_TUNING_PANEL
        );
        paint_scrollbar(
            body_rect, scroll, content_h, body_h, is_active, ctx.scene, theme,
        );
        ctx.host
            .hit_index_mut()
            .register(WET_TUNING_SCROLLBAR_ID, thumb);
    }
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::WET_TUNING_PANEL, content_h);
    store.set_panel_visible_h(ids::WET_TUNING_PANEL, body_h);
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(ids::WET_TUNING_PANEL) > max_scroll {
        store.set_panel_scroll(ids::WET_TUNING_PANEL, max_scroll);
    }
}
