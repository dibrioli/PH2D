//! The Font dropdown popover — a scrollable family list where **each row is drawn
//! in that family's own outline** (a real style preview, like Figma / Photoshop).
//!
//! Split from `paint_modes.rs` (panel LOC cap) and painted in a **deferred pass**
//! (after every body section) so the floating list lands on top. The chip's
//! open/close + light-dismiss + wheel-scroll come free from the generic `Dropdown`
//! dispatch (`InteractiveState::Dropdown`); this module only owns the popover paint
//! + the store scroll/hit wiring, mirroring the Painter panel's `dropdown_popover`.
//!
//! The family list + the `VariableFont`s live on the shell; it pre-renders each
//! name into a `FontPreview.outline` (em-normalised, y-up) and publishes it. Here
//! we place that outline into the row rect with an `Affine` (scale + y-flip). When
//! there are no previews yet the popover asks the shell to build them
//! (`request_font_previews`) and shows a "Loading…" row for that one frame.

use crate::ids;
use crate::state;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text, paint_text_centered, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    ButtonState, DROPDOWN_SCROLLBAR_ID, Dropdown, DropdownOption, paint_scrollbar, resolve_opaque,
    scrollbar_is_needed, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Color, Fill};

/// Glyph cap size as a fraction of the row height (leaves headroom for ascenders /
/// descenders so a preview never touches the neighbouring row).
const GLYPH_PX_RATIO: f64 = 0.62; // LITERAL-PX-OK: preview glyph size ÷ row height (list metric)
/// Baseline drop below the row's vertical centre, as a fraction of the glyph size
/// (optically centres the x-height, which sits above the baseline).
const GLYPH_BASELINE_RATIO: f64 = 0.34; // LITERAL-PX-OK: baseline offset ÷ glyph size (optical centring)

/// Paint the open font dropdown popover for `chip_rect`. No-op wiring is handled by
/// the caller (only invoked when the chip is open).
pub(crate) fn paint(ctx: &mut PaintCtx, chip_rect: Rect, theme: Theme) {
    let id = ids::VECTOR_TEXT_FONT_DD;
    let n = state::with_font_previews(<[state::FontPreview]>::len);
    if n == 0 {
        // Lazy build: the shell scans + parses the system fonts only now, on the
        // first open — never on Text-mode entry.
        state::request_font_previews();
    }
    // At least one row so the popover has a body (the "Loading…" placeholder).
    let count = n.max(1);
    let sel = selected_index();
    let options: Vec<DropdownOption<usize>> = (0..count)
        .map(|i| DropdownOption::new(ids::vector_text_font_option_id(i), i, String::new()))
        .collect();
    let dd = Dropdown::new(id, "", options).selected(sel).open(true);

    let panel = dd.popover_rect_clamped(chip_rect, ctx.viewport);
    let content_h = dd.content_height(chip_rect.h);
    let visible_h = panel.h;
    let max_scroll = (content_h - visible_h).max(0.0);

    // Publish geometry so the wheel + scrollbar drag scroll THIS popover, and clamp
    // a stale scroll if the list shrank.
    {
        let store = ctx.host.store_mut();
        store.set_dropdown_popover(id, panel);
        store.set_panel_content_h(id, content_h);
        store.set_panel_visible_h(id, visible_h);
        if store.panel_scroll(id) > max_scroll {
            store.set_panel_scroll(id, max_scroll);
        }
    }
    let scroll = ctx.host.store().panel_scroll(id).clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
    let scrollbar_active = matches!(ctx.host.store().scrollbar_drag(), Some(d) if d.panel == id);

    // Opaque floating surface (occludes the sections behind it).
    let radius = Radius::Md.px();
    fill_rounded_rect(
        ctx.scene,
        panel,
        radius,
        resolve_opaque(ColorToken::BgElev, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        panel,
        radius,
        1.0,
        resolve_opaque(ColorToken::Border, theme),
    );

    // Clip the rows to the panel so scrolled-out rows don't paint past the edges.
    let clip = ph2d_vector::Rect::new(
        panel.x as f64,
        panel.y as f64,
        (panel.x + panel.w) as f64,
        (panel.y + panel.h) as f64,
    );
    ctx.scene.push_clip(&clip);
    if n == 0 {
        let r = dd.option_rect_in_scrolled(chip_rect, panel, 0, scroll);
        paint_text_centered(
            ctx.text_system,
            ctx.scene,
            "Loading fonts...",
            r,
            TypeToken::Base.px(),
            resolve(ColorToken::Text2, theme),
        );
    } else {
        paint_rows(ctx, &dd, chip_rect, panel, scroll, sel, theme);
    }
    ctx.scene.pop_layer();

    if scrollbar_is_needed(content_h, visible_h) {
        paint_scrollbar(
            panel,
            scroll,
            content_h,
            visible_h,
            scrollbar_active,
            ctx.scene,
            theme,
        );
    }

    // Register the option rows as buttons (so a click emits `Click`) + hit-register
    // only their VISIBLE part; the scrollbar track is the drag zone.
    {
        let store = ctx.host.store_mut();
        for i in 0..count {
            store.register_if_absent(
                ids::vector_text_font_option_id(i),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
    }
    let hit_index = ctx.host.hit_index_mut();
    for i in 0..count {
        let r = dd.option_rect_in_scrolled(chip_rect, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            hit_index.register(
                ids::vector_text_font_option_id(i),
                Rect::new(r.x, top, r.w, bot - top),
            );
        }
    }
    if scrollbar_is_needed(content_h, visible_h) {
        ctx.host
            .hit_index_mut()
            .register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(panel));
    }
}

/// Paint each visible family row: the selected highlight + the name in its own
/// outline (falling back to the UI font when the outline is empty).
fn paint_rows(
    ctx: &mut PaintCtx,
    dd: &Dropdown<usize>,
    chip_rect: Rect,
    panel: Rect,
    scroll: f32,
    sel: usize,
    theme: Theme,
) {
    let text1 = resolve(ColorToken::Text1, theme);
    let accent_fg = resolve(ColorToken::AccentFg, theme);
    let accent = resolve_opaque(ColorToken::Accent, theme);
    let pad = Spacing::Md.px();
    state::with_font_previews(|previews| {
        for (i, pv) in previews.iter().enumerate() {
            let r = dd.option_rect_in_scrolled(chip_rect, panel, i, scroll);
            if r.y + r.h <= panel.y || r.y >= panel.y + panel.h {
                continue; // fully scrolled out (the clip handles partials)
            }
            let is_sel = i == sel;
            if is_sel {
                fill_rounded_rect(ctx.scene, r, Radius::Sm.px(), accent);
            }
            let color = if is_sel { accent_fg } else { text1 };
            if pv.outline.elements().is_empty() {
                // No self-name glyphs (rare) — show the display name in the UI font.
                paint_text(
                    ctx.text_system,
                    ctx.scene,
                    &pv.display,
                    r.x + pad,
                    r.y + (r.h - TypeToken::Base.px()) * 0.5,
                    TypeToken::Base.px(),
                    (r.w - pad * 2.0).max(1.0),
                    color,
                );
            } else {
                fill_name_outline(ctx, pv, r, pad, color);
            }
        }
    });
}

/// Place the em-normalised (y-up) name outline into the row: scale to a pixel cap
/// size, flip Y, and translate to the baseline. Filled `NonZero` (glyph winding).
fn fill_name_outline(ctx: &mut PaintCtx, pv: &state::FontPreview, r: Rect, pad: f32, color: Color) {
    let px = r.h as f64 * GLYPH_PX_RATIO;
    let baseline = r.y as f64 + r.h as f64 * 0.5 + px * GLYPH_BASELINE_RATIO;
    let x0 = r.x as f64 + pad as f64;
    let affine = Affine::translate((x0, baseline)) * Affine::scale_non_uniform(px, -px);
    ctx.scene.inner_mut().fill(
        Fill::NonZero,
        affine,
        &Brush::Solid(color),
        None,
        &pv.outline,
    );
}

/// Index of the currently-chosen family in the published preview list (0 = the
/// bundled default when nothing matches).
fn selected_index() -> usize {
    let cur = state::current_text_font();
    state::with_font_previews(|previews| {
        previews
            .iter()
            .position(|pv| Some(pv.display.as_str()) == cur.as_deref())
            .unwrap_or(0)
    })
}
