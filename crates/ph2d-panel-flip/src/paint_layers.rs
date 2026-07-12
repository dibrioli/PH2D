//! Flip Style panel — the Layers section (ADR-0114 W2 T2.15).
//!
//! Mirrors the Painter layers panel idiom: an Add / Delete toolbar + one block
//! per layer (top→bottom) with an eye (visibility), a padlock (lock), the name
//! (click selects), reorder ↑↓ arrows, a blend-mode dropdown, and a bare opacity
//! slider. The active layer gets an accent outline. Per-row widgets use the
//! runtime-hashed id family (`flip_layer_widget_id`) since the layer count is
//! only known at runtime; a disabled reorder arrow paints dim and is NOT
//! hit-registered, so it no-ops (mirror of the painter).
//!
//! Takes `ctx: &mut PaintCtx` directly (not the `BodyCtx` bundle) because the
//! per-row widgets need `register_if_absent` (a mutable store), interleaved with
//! the immutable-store reads + hit registration — the painter-layers pattern.

use crate::ids;
use crate::state::{FlipLayerRow, FlipLayersSnapshot};
use ph2d_editor_core::IconId;
use ph2d_editor_core::ids::FlipLayerWidget;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, DROPDOWN_SCROLLBAR_ID, Dropdown, DropdownOption,
    DropdownState, Slider, SliderState, paint_button, paint_dropdown_chip,
    paint_dropdown_popover_scrolled, paint_slider, scrollbar_is_needed, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_painter_effects::{BlendMode, MAX_BLEND_MODES};
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};

/// Layout metrics the Layers section needs (subset of the panel body metrics).
pub(crate) struct LayerMetrics {
    pub inner_x: f32,
    pub inner_w: f32,
    pub row_h: f32,
    pub row_gap: f32,
    pub font: f32,
}

/// The single open blend dropdown to paint on top of the rows, deferred to the
/// popover pass so it isn't clipped by later rows.
pub(crate) struct PendingBlend {
    pub layer: u64,
    pub chip_rect: Rect,
    pub cur: u8,
}

/// All 22 blend modes as dropdown options for the layer `layer_u64`.
fn blend_options(layer_u64: u64) -> Vec<DropdownOption<u8>> {
    (0..MAX_BLEND_MODES)
        .map(|m| {
            DropdownOption::new(
                ids::flip_layer_blend_option_id(layer_u64, m),
                m,
                BlendMode::from_u8(m).name(),
            )
        })
        .collect()
}

/// Register a plain action Button if absent.
fn button_absent(ctx: &mut PaintCtx, id: ph2d_a11y::NodeId) {
    ctx.host.store_mut().register_if_absent(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

/// A small square icon Button (eye / lock / reorder arrow). Registers the store
/// slot + hit rect; paints `icon` tinted by `enabled`.
fn icon_button(ctx: &mut PaintCtx, theme: Theme, id: ph2d_a11y::NodeId, icon: IconId, rect: Rect) {
    button_absent(ctx, id);
    let color = resolve(ColorToken::Text2, theme);
    paint_icon(ctx.scene, icon, rect, color, StrokeToken::Default.px());
    ctx.host.hit_index_mut().register(id, rect);
}

/// The Layers section: Add/Delete toolbar + one block per layer (top→bottom).
/// Returns the advanced `y`. A single open blend dropdown is stashed into
/// `pending` for the deferred popover pass.
pub(crate) fn layers_section(
    ctx: &mut PaintCtx,
    theme: Theme,
    m: &LayerMetrics,
    snap: &FlipLayersSnapshot,
    mut y: f32,
    pending: &mut Option<PendingBlend>,
) -> f32 {
    // Section label.
    let label_font = TypeToken::Sm.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        "Layers",
        m.inner_x,
        y,
        label_font,
        m.inner_w,
        resolve(ColorToken::Text2, theme),
    );
    y += label_font + Spacing::Xs.px();

    // Toolbar: Add | Delete (Delete disabled with no active layer).
    let gap = Spacing::Sm.px();
    let half = ((m.inner_w - gap) / 2.0).max(1.0);
    let add_rect = Rect::new(m.inner_x, y, half, m.row_h);
    button_absent(ctx, ids::FLIP_LAYER_ADD);
    {
        let st = ctx
            .host
            .store()
            .button_state(ids::FLIP_LAYER_ADD)
            .unwrap_or(ButtonState::Normal);
        let btn = Button::new(ids::FLIP_LAYER_ADD, "Add Layer")
            .kind(ButtonKind::Default)
            .state(st);
        paint_button(&btn, add_rect, ctx.scene, ctx.text_system, theme);
    }
    ctx.host
        .hit_index_mut()
        .register(ids::FLIP_LAYER_ADD, add_rect);

    let del_rect = Rect::new(m.inner_x + half + gap, y, half, m.row_h);
    let del_enabled = snap.active.is_some();
    button_absent(ctx, ids::FLIP_LAYER_DELETE);
    {
        let st = if del_enabled {
            ctx.host
                .store()
                .button_state(ids::FLIP_LAYER_DELETE)
                .unwrap_or(ButtonState::Normal)
        } else {
            ButtonState::Disabled
        };
        let btn = Button::new(ids::FLIP_LAYER_DELETE, "Delete")
            .kind(ButtonKind::Default)
            .state(st);
        paint_button(&btn, del_rect, ctx.scene, ctx.text_system, theme);
    }
    // A disabled button is not hit-registered → it no-ops (mirror of painter).
    if del_enabled {
        ctx.host
            .hit_index_mut()
            .register(ids::FLIP_LAYER_DELETE, del_rect);
    }
    y += m.row_h + m.row_gap;

    // Per-layer blocks, TOP-first (snap.rows is z-order, index 0 = back).
    let n = snap.rows.len();
    for (idx, row) in snap.rows.iter().enumerate().rev() {
        let can_up = idx + 1 < n; // a layer sits above
        let can_down = idx > 0; // a layer sits below
        y = paint_layer_block(
            ctx,
            theme,
            m,
            row,
            snap.active == Some(row.id),
            idx == 0, // bottom layer = base (no blend chip)
            can_up,
            can_down,
            y,
            pending,
        );
    }
    y
}

/// Paint one layer block: eye/lock/name/↑↓, (blend chip), opacity. The BOTTOM
/// layer (`is_base`) gets NO blend chip — it composites against nothing beneath,
/// so its blend mode is a no-op (mirror of the Painter layers panel hiding the
/// base blend). That block is 2 lines; the others are 3.
#[allow(clippy::too_many_arguments)]
fn paint_layer_block(
    ctx: &mut PaintCtx,
    theme: Theme,
    m: &LayerMetrics,
    row: &FlipLayerRow,
    active: bool,
    is_base: bool,
    can_up: bool,
    can_down: bool,
    mut y: f32,
    pending: &mut Option<PendingBlend>,
) -> f32 {
    let line_gap = Spacing::Xs.px();
    let block_top = y;
    // Rows (eye/name/↑↓, [blend], opacity) + inter-row gaps — line COUNTS, not a
    // design metric. The base layer drops the blend line (2 rows).
    let rows = if is_base { 2.0 } else { 3.0 }; // LITERAL-PX-OK: line COUNT (2 or 3), not a metric
    let block_h = m.row_h * rows + line_gap * (rows - 1.0); // LITERAL-PX-OK: row/gap counts

    // Active-row accent outline (painted first, behind the controls).
    if active {
        fill_rounded_rect(
            ctx.scene,
            Rect::new(m.inner_x, block_top, m.inner_w, block_h),
            Radius::Sm.px(),
            resolve(ColorToken::AccentSoft, theme),
        );
        stroke_rounded_rect(
            ctx.scene,
            Rect::new(m.inner_x, block_top, m.inner_w, block_h),
            Radius::Sm.px(),
            StrokeToken::Default.px(),
            resolve(ColorToken::Accent, theme),
        );
    }

    // ── Line 1: eye | lock | name (row-select) | ↑ ↓ ──
    let icon_w = m.row_h;
    let gap = Spacing::Xs.px();
    let eye_id = ids::flip_layer_widget_id(row.id, FlipLayerWidget::Visibility);
    icon_button(
        ctx,
        theme,
        eye_id,
        if row.visible {
            IconId::Eye
        } else {
            IconId::EyeClosed
        },
        Rect::new(m.inner_x + gap, y, icon_w, m.row_h),
    );
    let lock_id = ids::flip_layer_widget_id(row.id, FlipLayerWidget::Lock);
    icon_button(
        ctx,
        theme,
        lock_id,
        if row.locked {
            IconId::Lock
        } else {
            IconId::Unlock
        },
        Rect::new(m.inner_x + gap + icon_w + gap, y, icon_w, m.row_h),
    );

    // Reorder arrows on the right (disabled = dim + not hit-registered).
    let up_id = ids::flip_layer_widget_id(row.id, FlipLayerWidget::MoveUp);
    let down_id = ids::flip_layer_widget_id(row.id, FlipLayerWidget::MoveDown);
    let down_x = m.inner_x + m.inner_w - icon_w;
    let up_x = down_x - icon_w - gap;
    reorder_arrow(
        ctx,
        theme,
        up_id,
        IconId::ChevronUp,
        Rect::new(up_x, y, icon_w, m.row_h),
        can_up,
    );
    reorder_arrow(
        ctx,
        theme,
        down_id,
        IconId::ChevronDown,
        Rect::new(down_x, y, icon_w, m.row_h),
        can_down,
    );

    // Name — a click on the name strip selects (activates) the layer.
    let name_x = m.inner_x + gap + (icon_w + gap) * 2.0;
    let name_w = (up_x - gap - name_x).max(0.0);
    let row_id = ids::flip_layer_widget_id(row.id, FlipLayerWidget::Row);
    button_absent(ctx, row_id);
    let name_color = if row.visible {
        ColorToken::Text1
    } else {
        ColorToken::Text3
    };
    paint_text(
        ctx.text_system,
        ctx.scene,
        &row.name,
        name_x,
        y + (m.row_h - m.font) * 0.5,
        m.font,
        name_w,
        resolve(name_color, theme),
    );
    ctx.host
        .hit_index_mut()
        .register(row_id, Rect::new(name_x, y, name_w, m.row_h));
    y += m.row_h + line_gap;

    // ── Line 2: blend-mode dropdown chip (não no fundo — blend contra nada). ──
    if !is_base {
        let blend_id = ids::flip_layer_widget_id(row.id, FlipLayerWidget::Blend);
        let chip_rect = Rect::new(m.inner_x + gap, y, m.inner_w - gap * 2.0, m.row_h);
        paint_blend_chip(ctx, theme, blend_id, row, chip_rect, pending);
        y += m.row_h + line_gap;
    }

    // ── Line 3: bare opacity slider + NN% readout ──
    let op_id = ids::flip_layer_widget_id(row.id, FlipLayerWidget::Opacity);
    ctx.host.store_mut().register_if_absent(
        op_id,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: row.opacity,
            orientation: ph2d_editor_core::widget::SliderOrientation::Horizontal,
        },
    );
    let pct_w = 40.0; // LITERAL-PX-OK: fixed "NN%" readout column
    let slider_w = (m.inner_w - gap * 2.0 - pct_w).max(0.0);
    let op_rect = Rect::new(m.inner_x + gap, y, slider_w, m.row_h);
    let st = ctx
        .host
        .store()
        .slider(op_id)
        .map(|(s, _)| s)
        .unwrap_or(SliderState::Normal);
    let val = ctx
        .host
        .store()
        .slider(op_id)
        .map(|(_, v)| v)
        .unwrap_or(row.opacity);
    let mut slider = Slider::new(op_id, "").accent(true).state(st);
    slider.value = val;
    paint_slider(&slider, op_rect, ctx.scene, theme);
    ctx.host.hit_index_mut().register(op_id, op_rect);
    let pct = (val * 100.0).round() as i64; // LITERAL-PX-OK: fraction→percent readout
    paint_text(
        ctx.text_system,
        ctx.scene,
        &format!("{pct}%"),
        m.inner_x + gap + slider_w + gap,
        y + (m.row_h - m.font) * 0.5,
        m.font,
        pct_w,
        resolve(ColorToken::Text2, theme),
    );
    y += m.row_h + m.row_gap;
    y
}

/// A reorder arrow: dim + not hit-registered when disabled (no-op), else an
/// icon button.
fn reorder_arrow(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_a11y::NodeId,
    icon: IconId,
    rect: Rect,
    enabled: bool,
) {
    if enabled {
        icon_button(ctx, theme, id, icon, rect);
    } else {
        paint_icon(
            ctx.scene,
            icon,
            rect,
            resolve(ColorToken::TextDisabled, theme),
            StrokeToken::Default.px(),
        );
    }
}

/// Paint the blend chip (registered as a `Dropdown` for the generic open/close
/// dispatch). If open (single-open enforced), stash it into `pending` for the
/// deferred popover pass.
fn paint_blend_chip(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_a11y::NodeId,
    row: &FlipLayerRow,
    rect: Rect,
    pending: &mut Option<PendingBlend>,
) {
    ctx.host.store_mut().register_if_absent(
        id,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(row.blend as usize),
        },
    );
    let store_open = matches!(
        ctx.host.store().get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    // One popover at a time: only the first open dropdown (top→bottom) wins.
    let open = store_open && pending.is_none();
    if store_open
        && !open
        && let Some(InteractiveState::Dropdown { open: o, .. }) = ctx.host.store_mut().get_mut(id)
    {
        *o = false;
    }

    let dd = Dropdown::new(id, "", blend_options(row.id))
        .selected(row.blend)
        .open(open)
        .state(DropdownState::Normal);
    paint_dropdown_chip(&dd, rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(id, rect);
    if open {
        *pending = Some(PendingBlend {
            layer: row.id,
            chip_rect: rect,
            cur: row.blend,
        });
    }
}

/// Deferred paint of the single open blend dropdown popover (on top of the rows,
/// clamped to the viewport + scrollable — the 22 modes overflow the dock).
/// Registers each option as a Button + its hit rect so option clicks dispatch.
/// Thin replica of the Painter panel's `paint_dropdown_popover`.
pub(crate) fn paint_blend_popover(ctx: &mut PaintCtx, theme: Theme, pending: &PendingBlend) {
    let id = ids::flip_layer_widget_id(pending.layer, FlipLayerWidget::Blend);
    let options = blend_options(pending.layer);
    let dd = Dropdown::new(id, "", options)
        .selected(pending.cur)
        .open(true);
    let viewport = ctx.viewport;
    let panel = dd.popover_rect_clamped(pending.chip_rect, viewport);
    let content_h = dd.content_height(pending.chip_rect.h);
    let visible_h = panel.h;
    {
        let store = ctx.host.store_mut();
        store.set_dropdown_popover(id, panel);
        store.set_panel_content_h(id, content_h);
        store.set_panel_visible_h(id, visible_h);
    }
    let max_scroll = (content_h - visible_h).max(0.0);
    if ctx.host.store().panel_scroll(id) > max_scroll {
        ctx.host.store_mut().set_panel_scroll(id, max_scroll);
    }
    let scroll = ctx.host.store().panel_scroll(id).clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
    let scrollbar_active = matches!(ctx.host.store().scrollbar_drag(), Some(d) if d.panel == id);
    paint_dropdown_popover_scrolled(
        &dd,
        pending.chip_rect,
        panel,
        scroll,
        scrollbar_active,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    {
        let store = ctx.host.store_mut();
        for opt in dd.options.iter() {
            store.register_if_absent(
                opt.id,
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
    }
    // Register only the VISIBLE part of each option row (scrolled-out = no hit).
    for (i, opt) in dd.options.iter().enumerate() {
        let r = dd.option_rect_in_scrolled(pending.chip_rect, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            ctx.host
                .hit_index_mut()
                .register(opt.id, Rect::new(r.x, top, r.w, bot - top));
        }
    }
    if scrollbar_is_needed(content_h, visible_h) {
        ctx.host
            .hit_index_mut()
            .register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(panel));
    }
}
