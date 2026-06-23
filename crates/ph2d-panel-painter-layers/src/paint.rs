//! Painter layers panel paint — W3.T3.4 UI-plumbing (interactive rows).
//!
//! Render canon (mirror do `ph2d-panel-painter-sidebar` paint):
//! - Visibility gate via `PanelHostInternal::panel_visible`
//! - Right-dock rect de `ctx.layout.painter_layers`
//! - Chrome publish (`set_panel_rect`) pra dispatch hit-test
//! - Canon chrome: dark-glass surface + corner dot + title "Layers" + a
//!   dock-toggle button ("Brush", mode C) + close (X) button
//! - One interactive row per layer (top-to-bottom, recursing groups) — the
//!   row painters + W3.T3.8 drag overlay live in the [`crate::paint_rows`]
//!   sibling (panel-LOC cap); this file is the orchestrator.
//! - "+ Layer" button at the foot of the body
//! - the single open blend dropdown popover is painted last (on top)
//!
//! Per-row widget ids are derived via
//! [`ph2d_editor_core::ids::painter_layer_widget_id`] and **registered in the
//! `WidgetStore` here in `paint`** (the panel owns `store_mut`), so the normal
//! dispatch path emits `WidgetEvent`s — no hierarchy-style companion-bit
//! dispatcher allowlist needed. `event.rs` classifies those into a tool-
//! agnostic [`PanelEvent`] forwarded over `EditorAction::ToolPanelEvent`.

use crate::PainterLayersPanel;
use crate::paint_rows::{paint_drag_ghost, paint_drop_indicator, paint_layer_subtree};
use crate::state::{self, PainterLayersPanelState, set_last_content_h, set_last_visible_h};
use ph2d_editor_core::IconId;
use ph2d_editor_core::ids::{self as core_ids, PainterLayerWidget, painter_layer_widget_id};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{paint_text, rect_to_vello, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, panel_close_button_rect, panel_drag_handle_rect,
    panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, PAINTER_LAYERS_SCROLLBAR_ID, paint_button, paint_scrollbar,
    scrollbar_is_needed, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

// Chrome layout metrics (the per-row metrics live in `paint_rows.rs`).
const TOGGLE_BTN_W: f32 = 52.0; // LITERAL-PX-OK: header dock-toggle button width
const HEADER_ICON_W: f32 = 28.0; // LITERAL-PX-OK: action icon-button square
const TOOLBAR_H: f32 = 36.0; // LITERAL-PX-OK: action toolbar strip height (icon + pad)
const MOD_BTN_W: f32 = 52.0; // LITERAL-PX-OK: modifier toggle button width (text)
const APPLY_BTN_W: f32 = 80.0; // LITERAL-PX-OK: "Apply" CTA button width

pub(crate) fn paint(_state: &mut PainterLayersPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(PainterLayersPanel::ID) {
        // Painter closed while the brush colour picker was open → close it too,
        // else the floating picker lingers on the canvas. (Only ours — never the
        // Inspector's swatch target.)
        if ctx.host.store().picker_target() == Some(core_ids::PAINTER_COLOR_THUMB) {
            ctx.host.store_mut().set_picker_target(None);
        }
        ctx.host
            .store_mut()
            .clear_panel_rect(core_ids::PAINTER_LAYERS_PANEL);
        set_last_content_h(0.0);
        set_last_visible_h(0.0);
        return;
    }

    let rect: Rect = ctx.layout.painter_layers;
    let theme = ctx.host.theme();
    state::set_pending_blend_dd(None);
    state::set_pending_adj_menu(None);
    state::set_pending_brush_blend_dd(None);

    ctx.host
        .store_mut()
        .set_panel_rect(core_ids::PAINTER_LAYERS_PANEL, rect);

    // Chrome: dark-glass surface + corner accent.
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);

    // Dock-slot drag + resize handles (shared canon — Inspector right-dock).
    {
        let drag_rect =
            panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
        let resize_rect = panel_resize_handle_rect(rect);
        let resize_bl_rect = panel_resize_handle_rect_bl(rect);
        let hit_index = ctx.host.hit_index_mut();
        hit_index.register(core_ids::INSP_DRAG_HANDLE, drag_rect);
        hit_index.register(core_ids::INSP_RESIZE_HANDLE, resize_rect);
        hit_index.register(core_ids::INSP_RESIZE_HANDLE_BL, resize_bl_rect);
    }

    // Dock view-mode (published by the bridge): Layers/Effects vs Brush props.
    let shows_layers = state::current_dock_shows_layers();
    let title_size = paint_panel_title(
        rect,
        if shows_layers { "Layers" } else { "Brush" },
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );

    // Dock-toggle (label = the OTHER view) + close (X).
    paint_dock_toggle(ctx, rect, theme, shows_layers);
    paint_panel_close_button(
        rect,
        core_ids::PAINTER_LAYERS_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    let header_bottom = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();

    // ── Brush-properties view: scrollable brush body (extracted to keep `paint`
    // under the panel fn-LOC cap). Same scroll machinery as the Layers view below. ─
    if !shows_layers {
        paint_brush_view(ctx, theme, rect, header_bottom);
        return;
    }
    // Layers view: if the shared colour picker is still open from the Brush view,
    // close it (it edits the brush colour, irrelevant here).
    {
        let store = ctx.host.store_mut();
        if store.picker_target() == Some(core_ids::PAINTER_COLOR_THUMB) {
            store.set_picker_target(None);
        }
    }

    // Two fixed toolbar strips below the header: actions (New / Group /
    // Duplicate / Delete) then modifiers (Mask / Clip / Lock / Ref), between the
    // header and the scrollable list.
    let toolbar_rect = Rect::new(rect.x, header_bottom, rect.w, TOOLBAR_H);
    let modifier_toolbar_rect = Rect::new(rect.x, header_bottom + TOOLBAR_H, rect.w, TOOLBAR_H);

    // Body region (clipped), starting below both toolbar strips.
    let body_top = header_bottom + TOOLBAR_H * 2.0;
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);

    ctx.scene.push_clip(&rect_to_vello(body_rect));

    let scroll_y = ctx
        .host
        .store()
        .panel_scroll(core_ids::PAINTER_LAYERS_PANEL)
        .max(0.0);
    // Body content scrolls: the paint origin is offset up by the scroll
    // position; content_h is measured from it (scroll-independent). Extra top
    // padding so the active-row accent outline is not clipped at the body top
    // and the first row sits clear of the Layers title.
    let body_paint_top = body_top + Spacing::Md.px() - scroll_y;
    let mut y = body_paint_top;
    let content_w = rect.w - PANEL_HEAD_PAD * 2.0;

    // Row-id set published to the dispatch so it knows which NodeIds are
    // draggable layer rows (Coord drag foundation `1c3411d`). Filled in the
    // layer branch, pushed in the scroll-bounds block below.
    let mut painter_row_ids: std::collections::BTreeSet<ph2d_a11y::NodeId> =
        std::collections::BTreeSet::new();
    // W3.T3.8 drag overlay: the live reparent gesture (id being dragged + the
    // latest cursor). Copied out (state is `Copy`) so no store borrow lingers
    // into the `store_mut()` calls below. `None` unless a drag passed the
    // distance threshold (`active`).
    let dragging = ctx.host.store().painter_layer_drag().filter(|d| d.active);
    // The floating "lifted layer" pill (name + cursor) painted last, unclipped.
    let mut ghost: Option<(String, f32, f32)> = None;
    match state::current_layers() {
        Some(stack) if !stack.is_empty() => {
            painter_row_ids = stack
                .all_ids()
                .filter(|&l| !stack.is_mask(l)) // masks are selectable but not draggable
                .map(|l| painter_layer_widget_id(l.0, PainterLayerWidget::Row))
                .collect();
            let active = stack.active();
            // W3 multi-select: the rows to highlight (active = strong outline,
            // the rest = soft wash). Published by the bridge each frame.
            let selected = state::current_selection();
            // Full-row drop bands collected during the walk (id, full-row rect,
            // is_group) so the indicator below mirrors `find_painter_layer_drop`
            // exactly (same rows, same 30/40/30) — WYSIWYG drop.
            let mut drag_rows: Vec<(ph2d_a11y::NodeId, Rect, bool)> = Vec::new();
            y = paint_layer_subtree(
                ctx,
                theme,
                &stack,
                stack.root(),
                active,
                &selected,
                0,
                rect.x + PANEL_HEAD_PAD,
                content_w,
                y,
                dragging,
                &mut drag_rows,
            );
            if let Some(d) = dragging {
                // Live drop indicator, on top of the rows (still inside the body
                // clip so it can't bleed over the header/footer).
                paint_drop_indicator(
                    ctx,
                    theme,
                    &drag_rows,
                    d.cursor_y,
                    d.dragged,
                    rect.x + PANEL_HEAD_PAD,
                    content_w,
                );
                // Decode the dragged NodeId → its layer name for the ghost pill.
                ghost = stack
                    .all_ids()
                    .find(|lid| {
                        painter_layer_widget_id(lid.0, PainterLayerWidget::Row) == d.dragged
                    })
                    .and_then(|lid| stack.get(lid))
                    .map(|l| (l.name.clone(), d.cursor_x, d.cursor_y));
            }
        }
        _ => {
            let font = TypeToken::Base.px();
            paint_text(
                ctx.text_system,
                ctx.scene,
                "No layers",
                rect.x + PANEL_HEAD_PAD,
                y,
                font,
                content_w,
                resolve(ColorToken::Text2, theme),
            );
            y += font + Spacing::Md.px();
        }
    }

    y += Spacing::Xs.px();
    // Footer: the "Apply" CTA (right) — commits the composite into the sprite
    // (the only discoverable commit; Cmd/Ctrl+Enter is the hidden shortcut).
    // "+ Layer" moved to the header icon cluster (New layer / Group / Duplicate /
    // Delete) — `paint_header_actions`, registered post-body (last-wins).
    let footer_x = rect.x + PANEL_HEAD_PAD;
    let apply_rect = Rect::new(footer_x + content_w - APPLY_BTN_W, y, APPLY_BTN_W, ROW_H_PX);
    paint_apply_button(ctx, apply_rect, theme);
    y += ROW_H_PX;

    let content_h = (y - body_paint_top + PANEL_HEAD_PAD).max(0.0);
    set_last_content_h(content_h);
    set_last_visible_h(body_h);

    ctx.scene.pop_layer();

    // Visual scrollbar (self-gates when the content fits). Wheel/trackpad
    // scrolling works via the generic `dispatch_wheel` once the bounds below are
    // published; thumb-DRAG works via the foundational
    // `scrollbar_panel_for_id → PAINTER_LAYERS_PANEL` mapping (Coord `d5146b7`)
    // plus the hit-rect registered in the post-body chrome block. `is_active`
    // tints the thumb Accent while dragging (mirror of the Inspector).
    let scrollbar_active = matches!(
        ctx.host.store().scrollbar_drag(),
        Some(d) if d.panel == core_ids::PAINTER_LAYERS_PANEL
    );
    paint_scrollbar(
        body_rect,
        scroll_y,
        content_h,
        body_h,
        scrollbar_active,
        ctx.scene,
        theme,
    );

    paint_panel_corner_dot_bl(rect, ctx.scene, theme);
    // Re-register ALL header chrome AFTER the body rows. `HitIndex::hit` is
    // last-registered-wins, and scrolled-up rows have hit rects with `y <
    // body_top` that overlap the header — without this they shadow the
    // dock-toggle / drag / resize handles (audit HIGH). The body paint is
    // clipped so those rows are invisible, but their hits are not clipped.
    {
        let close = panel_close_button_rect(rect);
        let toggle = Rect::new(
            close.x - Spacing::Sm.px() - TOGGLE_BTN_W,
            close.y,
            TOGGLE_BTN_W,
            close.h,
        );
        let hit = ctx.host.hit_index_mut();
        hit.register(
            core_ids::INSP_DRAG_HANDLE,
            panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE),
        );
        hit.register(core_ids::INSP_RESIZE_HANDLE, panel_resize_handle_rect(rect));
        hit.register(
            core_ids::INSP_RESIZE_HANDLE_BL,
            panel_resize_handle_rect_bl(rect),
        );
        hit.register(core_ids::PAINTER_LAYERS_TOGGLE_DOCK, toggle);
        hit.register(core_ids::PAINTER_LAYERS_CLOSE, close);
        // Scrollbar drag: register the whole TRACK (not just the thin thumb) AFTER the rows (same
        // last-wins reason as the chrome above), so the user can grab the bar ANYWHERE — clicking the
        // visible rail off the thumb used to hit nothing. It also hands the dispatch the real track
        // height (`begin_scrollbar_drag` snapshots `track_h = rect.h`) for proportional scrolling.
        if scrollbar_is_needed(content_h, body_h) {
            hit.register(PAINTER_LAYERS_SCROLLBAR_ID, scrollbar_track_rect(body_rect));
        }
    }

    // Action toolbar icons (New layer / Group / Duplicate / Delete) — one row
    // below the header. Painted + registered AFTER the rows (same last-wins
    // reason as the chrome above; scrolled rows must not shadow the toolbar).
    paint_action_toolbar(ctx, toolbar_rect, theme);
    paint_modifier_toolbar(ctx, modifier_toolbar_rect, theme);

    // Publish scroll bounds so `dispatch_wheel` scrolls this panel + clamp the
    // offset to the new content (so deleting/collapsing rows snaps back).
    {
        let store = ctx.host.store_mut();
        store.set_panel_content_h(core_ids::PAINTER_LAYERS_PANEL, content_h);
        store.set_panel_visible_h(core_ids::PAINTER_LAYERS_PANEL, body_h);
        let max_scroll = (content_h - body_h).max(0.0);
        if store.panel_scroll(core_ids::PAINTER_LAYERS_PANEL) > max_scroll {
            store.set_panel_scroll(core_ids::PAINTER_LAYERS_PANEL, max_scroll);
        }
        // Tell the dispatch which NodeIds are draggable layer rows (Down on one
        // of these begins a `PainterLayerReparent` drag — Coord drag foundation).
        store.set_painter_layer_row_ids(painter_row_ids);
    }

    // Deferred: the single open blend dropdown popover, on top of everything.
    if let Some((layer_u64, chip_rect, cur_mode)) = state::take_pending_blend_dd() {
        crate::blend::paint_blend_popover(ctx, theme, layer_u64, chip_rect, cur_mode);
    }

    // Deferred: the open "+ Adjustment" kind-picker menu, on top of everything
    // (after the blend popover — only one is realistically open at a time).
    if let Some(menu_chip) = state::take_pending_adj_menu() {
        crate::adjust_menu::paint_adjustment_menu_popover(ctx, theme, menu_chip);
    }

    // Deferred: the active Texture layer's editor dropdown popovers (Kind + Color Ramp Mode /
    // Interp / Alpha), drained on top of the rows so they float unclipped.
    crate::paint_texture::paint_texture_popovers(ctx, theme);

    // W3.T3.8: the floating drag ghost is the very top layer (a drag and an open
    // blend popover are mutually exclusive, so order vs the popover is moot).
    // Painted after the body clip pop so it tracks the cursor past the list.
    if let Some((name, cx, cy)) = ghost {
        paint_drag_ghost(ctx, theme, &name, cx, cy);
    }
}

/// Paint the scrollable Brush-properties body (Size / Strength / Blend / Falloff / Stroke / Eraser /
/// Colour) below `header_bottom`. Mirrors the Layers view's scroll machinery: clip the body, offset
/// the paint origin by the scroll position, draw the scrollbar, and publish the content/visible
/// heights so the generic `dispatch_wheel` + the thumb drag route here (shared `PAINTER_LAYERS_PANEL`
/// key + `PAINTER_LAYERS_SCROLLBAR_ID`). The open dropdown popover is drained AFTER the clip is
/// popped, so it floats unclipped over the rows.
fn paint_brush_view(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme, rect: Rect, header_bottom: f32) {
    let body_top = header_bottom;
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    let scroll_y = ctx
        .host
        .store()
        .panel_scroll(core_ids::PAINTER_LAYERS_PANEL)
        .max(0.0);
    let body_paint_top = body_top + Spacing::Md.px() - scroll_y;

    ctx.scene.push_clip(&rect_to_vello(body_rect));
    let content_bottom = crate::paint_brush::paint_brush_body(ctx, theme, rect, body_paint_top);
    ctx.scene.pop_layer();

    let content_h = (content_bottom - body_paint_top + PANEL_HEAD_PAD).max(0.0);
    set_last_content_h(content_h);
    set_last_visible_h(body_h);

    let scrollbar_active = matches!(
        ctx.host.store().scrollbar_drag(),
        Some(d) if d.panel == core_ids::PAINTER_LAYERS_PANEL
    );
    paint_scrollbar(
        body_rect,
        scroll_y,
        content_h,
        body_h,
        scrollbar_active,
        ctx.scene,
        theme,
    );
    if scrollbar_is_needed(content_h, body_h) {
        // Register the whole TRACK as the drag zone (grab the bar anywhere; real `track_h` for the
        // dispatch). The Brush-properties view overflows readily now, so this is the path Enio hits.
        ctx.host
            .hit_index_mut()
            .register(PAINTER_LAYERS_SCROLLBAR_ID, scrollbar_track_rect(body_rect));
    }
    {
        let store = ctx.host.store_mut();
        store.set_panel_content_h(core_ids::PAINTER_LAYERS_PANEL, content_h);
        store.set_panel_visible_h(core_ids::PAINTER_LAYERS_PANEL, body_h);
        let max_scroll = (content_h - body_h).max(0.0);
        if store.panel_scroll(core_ids::PAINTER_LAYERS_PANEL) > max_scroll {
            store.set_panel_scroll(core_ids::PAINTER_LAYERS_PANEL, max_scroll);
        }
    }
    // The open dropdown popover (Blend / Falloff / Method / Jitter Unit) floats over the body,
    // unclipped, above the scrollbar.
    crate::paint_brush::paint_brush_popovers(ctx, theme);
}

/// Header dock-toggle — swaps the docked body between the Layers/Effects view
/// and the Brush-properties view. Labelled with the OTHER view (what a click
/// switches TO). Placed left of the close button.
fn paint_dock_toggle(
    ctx: &mut PaintCtx,
    rect: Rect,
    theme: ph2d_tokens::Theme,
    shows_layers: bool,
) {
    let close = panel_close_button_rect(rect);
    let btn_rect = Rect::new(
        close.x - Spacing::Sm.px() - TOGGLE_BTN_W,
        close.y,
        TOGGLE_BTN_W,
        close.h,
    );
    let st = ctx
        .host
        .store()
        .button_state(core_ids::PAINTER_LAYERS_TOGGLE_DOCK)
        .unwrap_or(ButtonState::Normal);
    let label = if shows_layers { "Brush" } else { "Layers" };
    let btn = Button::new(core_ids::PAINTER_LAYERS_TOGGLE_DOCK, label).state(st);
    paint_button(&btn, btn_rect, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(core_ids::PAINTER_LAYERS_TOGGLE_DOCK, btn_rect);
}

/// Action toolbar (one row below the header): New layer · Group · Duplicate ·
/// Delete — the canonical ghost-icon button (same look as the Hierarchy header
/// "+"). Laid out left-to-right inside `toolbar_rect`; painted + registered
/// post-body so scrolled rows don't shadow them.
fn paint_action_toolbar(ctx: &mut PaintCtx, toolbar_rect: Rect, theme: ph2d_tokens::Theme) {
    let mut x = toolbar_rect.x + PANEL_HEAD_PAD;
    // Vertically center the square icon within the toolbar strip.
    let y = toolbar_rect.y + ((toolbar_rect.h - HEADER_ICON_W) * 0.5).max(0.0);
    let specs = [
        (core_ids::PAINTER_LAYERS_ADD, IconId::Add, "New layer"),
        (core_ids::PAINTER_LAYERS_GROUP, IconId::Group, "New group"),
        (
            core_ids::PAINTER_LAYERS_DUPLICATE,
            IconId::Duplicate,
            "Duplicate layer",
        ),
        (
            core_ids::PAINTER_LAYERS_DELETE,
            IconId::Trash,
            "Delete layer",
        ),
    ];
    for (id, icon, label) in specs {
        let btn_rect = Rect::new(x, y, HEADER_ICON_W, HEADER_ICON_W);
        let st = ctx
            .host
            .store()
            .button_state(id)
            .unwrap_or(ButtonState::Normal);
        let btn = Button::new(id, label).icon_only(icon).state(st);
        paint_button(&btn, btn_rect, ctx.scene, ctx.text_system, theme);
        ctx.host.hit_index_mut().register(id, btn_rect);
        x += HEADER_ICON_W + Spacing::Xs.px();
    }

    // "+ Adj" — a Dropdown (W4 T4.15), not a plain action: clicking it toggles a
    // 24-kind picker open via the generic Dropdown dispatch. Painted as an icon
    // button (Accent-filled while open); when open, stash a full-width synthetic
    // chip at this toolbar row so the deferred popover drops straight down the
    // panel (every kind name fits one line — narrow right-align like the blend
    // chip would clip the long names).
    let adj_id = core_ids::PAINTER_LAYERS_ADD_ADJUSTMENT;
    let adj_rect = Rect::new(x, y, HEADER_ICON_W, HEADER_ICON_W);
    let open = matches!(
        ctx.host.store().get(adj_id),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let mut adj_btn = Button::new(adj_id, "Add adjustment").icon_only(IconId::ColorEqualization);
    if open {
        adj_btn.kind = ButtonKind::Accent;
    }
    paint_button(&adj_btn, adj_rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(adj_id, adj_rect);
    if open {
        let menu_chip = Rect::new(
            toolbar_rect.x + PANEL_HEAD_PAD,
            y,
            (toolbar_rect.w - PANEL_HEAD_PAD * 2.0).max(0.0),
            ROW_H_PX,
        );
        state::set_pending_adj_menu(Some(menu_chip));
    }

    // "+ Texture" — create a Texture layer (a procedural brush-texture fill recoloured by a Color
    // Ramp, covering the sprite); a plain action next to "+ Adj".
    x += HEADER_ICON_W + Spacing::Xs.px();
    let tex_id = core_ids::PAINTER_LAYERS_ADD_TEXTURE;
    let tex_rect = Rect::new(x, y, HEADER_ICON_W, HEADER_ICON_W);
    let tex_st = ctx
        .host
        .store()
        .button_state(tex_id)
        .unwrap_or(ButtonState::Normal);
    let tex_btn = Button::new(tex_id, "Add texture layer")
        .icon_only(IconId::Grid)
        .state(tex_st);
    paint_button(&tex_btn, tex_rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(tex_id, tex_rect);
}

/// Modifier toolbar (second row): Mask · Clip · Lock · Ref — text toggle
/// buttons acting on the ACTIVE layer. The three toggles fill `Accent` when on
/// (read from the active layer's flags); Mask is a create action (Accent = the
/// layer already has a mask). Disabled + un-registered when the active layer
/// isn't a raster (a group/mask can't take these), so the click is a no-op.
fn paint_modifier_toolbar(ctx: &mut PaintCtx, toolbar_rect: Rect, theme: ph2d_tokens::Theme) {
    let mods = state::current_layers().and_then(|s| s.active_modifiers());
    let raster = mods.is_some_and(|m| m.is_raster);
    let y = toolbar_rect.y + ((toolbar_rect.h - ROW_H_PX) * 0.5).max(0.0);
    let mut x = toolbar_rect.x + PANEL_HEAD_PAD;
    let specs = [
        (
            core_ids::PAINTER_LAYERS_MASK,
            "Mask",
            mods.is_some_and(|m| m.has_mask),
        ),
        (
            core_ids::PAINTER_LAYERS_CLIP,
            "Clip",
            mods.is_some_and(|m| m.clipping),
        ),
        (
            core_ids::PAINTER_LAYERS_ALPHA_LOCK,
            "Lock",
            mods.is_some_and(|m| m.alpha_locked),
        ),
        (
            core_ids::PAINTER_LAYERS_REFERENCE,
            "Ref",
            mods.is_some_and(|m| m.is_reference),
        ),
    ];
    for (id, label, on) in specs {
        let btn_rect = Rect::new(x, y, MOD_BTN_W, ROW_H_PX);
        let st = if raster {
            ctx.host
                .store()
                .button_state(id)
                .unwrap_or(ButtonState::Normal)
        } else {
            ButtonState::Disabled
        };
        let mut btn = Button::new(id, label).state(st);
        if on && raster {
            btn.kind = ButtonKind::Accent; // toggle ON = filled accent
        }
        paint_button(&btn, btn_rect, ctx.scene, ctx.text_system, theme);
        if raster {
            ctx.host.hit_index_mut().register(id, btn_rect);
        }
        x += MOD_BTN_W + Spacing::Xs.px();
    }
}

/// "Apply" footer CTA — commits the live layer composite into the sprite
/// (routes to `PainterTool::request_commit`). Accent-filled for prominence.
fn paint_apply_button(ctx: &mut PaintCtx, rect: Rect, theme: ph2d_tokens::Theme) {
    let st = ctx
        .host
        .store()
        .button_state(core_ids::PAINTER_APPLY)
        .unwrap_or(ButtonState::Normal);
    let btn = Button::new(core_ids::PAINTER_APPLY, "Apply")
        .accent()
        .state(st);
    paint_button(&btn, rect, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(core_ids::PAINTER_APPLY, rect);
}

/// `register_if_absent` a per-row Button slot (dispatch needs the store entry
/// to emit `Click`; paint draws the visuals). Shared with `paint_rows` (rows)
/// and `blend` (the blend-popover option buttons).
pub(crate) fn register_button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register_if_absent(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}
