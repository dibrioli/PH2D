//! Painter sidebar paint — T2.1 Day-7 functional + chrome canon.
//!
//! Render canon (mirror BgRemoval/Padding sidebar pattern):
//! - Visibility gate via `PanelHostInternal::panel_visible`
//! - Right-dock rect de `ctx.layout.painter_sidebar`
//! - Chrome publish (`set_panel_rect`) pra dispatch hit-test
//! - Canon chrome: dark-glass surface + corner dot + title "Painter"
//!   + close (X) button (PANEL_HEADER_CLOSE_RESERVE)
//! - Drag handle + 2 resize handles (Inspector slot shared canon)
//! - 2 sliders via `paint_slider_with_chip_layout_adaptive` (label demota
//!   pra linha própria em dock estreito — iPad portrait):
//!   * Size (0..1 → px display via `ph2d_tool_painter::size01_to_px`)
//!   * Opacity (0..1 → 0..100% display)
//! - Body inside scroll clip; `content_h` / `visible_h` publish
//!
//! W2.T2.2 (undo/redo buttons) e T2.4 (modifier square) virão em commits
//! seguintes.

use crate::PainterSidebarPanel;
use crate::state::{self, PainterSidebarPanelState, set_last_content_h, set_last_visible_h};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::{paint_text, rect_to_vello, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_corner_dot, paint_panel_corner_dot_bl, paint_panel_surface, paint_panel_title,
    panel_close_button_rect, panel_drag_handle_rect, panel_resize_handle_rect,
    panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    Button, ButtonState, ColorSwatch, SwatchSize, paint_button, paint_color_swatch,
    paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

const SLIDER_LABEL_W: f32 = 70.0; // LITERAL-PX-OK: sidebar slider label column width (component-specific layout, not a global Spacing-scale step)
const SLIDER_CHIP_W: f32 = 64.0; // LITERAL-PX-OK: sidebar slider value-chip column width (component-specific layout, not a global Spacing-scale step)

pub(crate) fn paint(_state: &mut PainterSidebarPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(PainterSidebarPanel::ID) {
        ctx.host
            .store_mut()
            .clear_panel_rect(core_ids::PAINTER_SIDEBAR_PANEL);
        set_last_content_h(0.0);
        set_last_visible_h(0.0);
        return;
    }

    let rect: Rect = ctx.layout.painter_sidebar;
    let theme = ctx.host.theme();
    let snapshot = state::current_snapshot();

    ctx.host
        .store_mut()
        .set_panel_rect(core_ids::PAINTER_SIDEBAR_PANEL, rect);

    // Chrome: dark-glass surface + corner accent.
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);

    // Dock-slot drag + resize handles (shared canon — Inspector right-dock
    // slot persistence). BgRemoval/Padding pattern.
    {
        let drag_rect = panel_drag_handle_rect(
            rect,
            ph2d_editor_core::widget::panel_chrome::PANEL_HEADER_H_DEFAULT,
            PANEL_HEADER_CLOSE_RESERVE,
        );
        let resize_rect = panel_resize_handle_rect(rect);
        let resize_bl_rect = panel_resize_handle_rect_bl(rect);
        let hit_index = ctx.host.hit_index_mut();
        hit_index.register(ph2d_editor_core::ids::INSP_DRAG_HANDLE, drag_rect);
        hit_index.register(ph2d_editor_core::ids::INSP_RESIZE_HANDLE, resize_rect);
        hit_index.register(ph2d_editor_core::ids::INSP_RESIZE_HANDLE_BL, resize_bl_rect);
    }

    // Title — reserve room pra close button.
    let title_size = paint_panel_title(
        rect,
        "Painter",
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );

    // Dock-toggle button ("Layers") — mode C: swaps the shared dock slot to
    // the layers panel. Sits just left of the close (X) button. Mirror do
    // `ph2d-panel-painter-layers` header toggle ("Brush").
    paint_dock_toggle(ctx, rect, theme);

    // Close (X) button — routes pra CancelActiveTool (canon BgRemoval).
    paint_panel_close_button(
        rect,
        core_ids::PAINTER_SIDEBAR_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    // Body region (clipped) — sliders dentro.
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    let row_pad = Spacing::Md.px();

    ctx.scene.push_clip(&rect_to_vello(body_rect));

    let mut y = body_top;

    // Color row (top-most — Procreate "current color"). Helper keeps this
    // fn under the panel-fn LOC cap.
    y = paint_color_row(ctx, rect, y, &snapshot, theme, row_pad);

    // Size slider — size_px display via display_override + SSOT map.
    // Adaptive layout (audit W-4): demotes the label to its own row when
    // the dock is too narrow (iPad portrait) instead of collapsing the
    // track to a ~1 px sliver.
    let size_px = ph2d_tool_painter::size01_to_px(snapshot.size01);
    let size_display = format!("{size_px:.0} px");
    let size_rect = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        ROW_H_PX,
    );
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let size_h = paint_slider_with_chip_layout_adaptive(
        size_rect,
        "Size",
        snapshot.size01,
        size_px as f64,
        Some(&size_display),
        core_ids::PAINTER_SIDEBAR_SIZE_SLIDER,
        core_ids::PAINTER_SIDEBAR_SIZE_CHIP,
        SLIDER_LABEL_W,
        SLIDER_CHIP_W,
        store,
        hit_index,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    y += size_h + row_pad;

    // Opacity slider (display 0..100%, SSOT map).
    let opacity_pct = ph2d_tool_painter::opacity01_to_pct(snapshot.opacity01);
    let opacity_display = format!("{opacity_pct:.0}%");
    let opacity_rect = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        ROW_H_PX,
    );
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let opacity_h = paint_slider_with_chip_layout_adaptive(
        opacity_rect,
        "Opacity",
        snapshot.opacity01,
        opacity_pct as f64,
        Some(&opacity_display),
        core_ids::PAINTER_SIDEBAR_OPACITY_SLIDER,
        core_ids::PAINTER_SIDEBAR_OPACITY_CHIP,
        SLIDER_LABEL_W,
        SLIDER_CHIP_W,
        store,
        hit_index,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    y += opacity_h + row_pad;

    // Apply CTA — commit the painting into the active sprite (accent, full
    // width). The only discoverable commit; Cmd/Ctrl+Enter is the hidden
    // shortcut. Shared id + tool routing with the layers panel.
    let apply_rect = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        ROW_H_PX,
    );
    let apply_st = ctx
        .host
        .store()
        .button_state(core_ids::PAINTER_APPLY)
        .unwrap_or(ButtonState::Normal);
    let apply_btn = Button::new(core_ids::PAINTER_APPLY, "Apply")
        .accent()
        .state(apply_st);
    paint_button(&apply_btn, apply_rect, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(core_ids::PAINTER_APPLY, apply_rect);
    y += ROW_H_PX + row_pad;

    let content_h = (y - body_top + PANEL_HEAD_PAD).max(0.0);
    set_last_content_h(content_h);
    set_last_visible_h(body_h);

    ctx.scene.pop_layer();

    // Bottom-LEFT resize corner dot (mirror canon BR). Painted AFTER body
    // widgets pra ficar visualmente em cima de qualquer drift no canto.
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);

    // Re-register close button no fim do frame pra scrolled body widgets
    // não shadowarem o close (canon panel_chrome doc).
    ctx.host.hit_index_mut().register(
        core_ids::PAINTER_SIDEBAR_CLOSE,
        panel_close_button_rect(rect),
    );
}

/// Paint the top-most "Color" row — current brush color via the canonical
/// [`ColorSwatch`] (Widget Gallery `widget/showcase/color.rs`): a left
/// "Color" label, then the eyedropper icon + the swatch pinned to the
/// row's right edge, mirroring Procreate's "current color" + pick
/// affordance at the top of the tool sidebar.
///
/// Swatch fill comes from the live snapshot (`active_color_srgb8`) so it
/// tracks the picker in real time. The user RGBA is data — no hex literal
/// here; `paint_color_swatch` carries the single justified
/// `LITERAL-COLOR-OK`. Label/chrome use tokens.
///
/// Two hits are registered:
/// - `PAINTER_COLOR_THUMB` (swatch) → editor-core dispatch opens the
///   Blender picker seeded with this color; the shell `painter_bridge`
///   round-trips the picked color back into the Painter (placement-agnostic).
/// - `PAINTER_SIDEBAR_MODIFIER_SQUARE` (eyedropper icon, T2.4) → arms/
///   disarms the eyedropper-while-held gesture (Pressed/AccentSoft when
///   armed). Routes `Click` → `PanelEvent::Click` →
///   `PainterUiEdit::ToggleEyedropper`. The on-canvas sample itself is
///   shell/foundational (Coordinator).
///
/// Returns the `y` advanced past this row.
fn paint_color_row(
    ctx: &mut PaintCtx,
    rect: Rect,
    y: f32,
    snapshot: &ph2d_tool_painter::PainterUiSnapshot,
    theme: ph2d_tokens::Theme,
    row_pad: f32,
) -> f32 {
    let color_rect = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        ROW_H_PX,
    );
    // Swatch pinned to the right edge; eyedropper icon just left of it.
    let swatch_w = Spacing::Xl3.px();
    let swatch_rect = Rect::new(
        color_rect.x + color_rect.w - swatch_w,
        color_rect.y,
        swatch_w,
        ROW_H_PX,
    );
    let eye_w = ROW_H_PX; // square icon chip, row-height
    let eye_rect = Rect::new(
        swatch_rect.x - Spacing::Sm.px() - eye_w,
        color_rect.y,
        eye_w,
        ROW_H_PX,
    );
    // Pigment toggle (W5) — square chip left of the eyedropper. Pressed = the
    // active brush is in Subtractive (pigment) mode.
    let pigment_rect = Rect::new(
        eye_rect.x - Spacing::Sm.px() - eye_w,
        color_rect.y,
        eye_w,
        ROW_H_PX,
    );
    // Eyedropper armed visual: Pressed (AccentSoft chip) while a pick is
    // pending (the picker's `eyedropper_pending`, shared with the picker's
    // own eyedropper button — one mechanism), else the dispatcher-tracked
    // hover/press state.
    let eye_state = if ctx.host.store().eyedropper_pending().is_some() {
        ButtonState::Pressed
    } else {
        ctx.host
            .store()
            .button_state(core_ids::PAINTER_SIDEBAR_MODIFIER_SQUARE)
            .unwrap_or(ButtonState::Normal)
    };

    let label_font = TypeToken::Base.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        "Color",
        color_rect.x,
        color_rect.y + (ROW_H_PX - label_font) * 0.5,
        label_font,
        SLIDER_LABEL_W,
        resolve(ColorToken::Text1, theme),
    );

    let swatch = ColorSwatch::new(
        core_ids::PAINTER_COLOR_THUMB,
        "Brush color",
        snapshot.active_color_srgb8(),
    )
    .size(SwatchSize::Md);
    paint_color_swatch(&swatch, swatch_rect, ctx.scene, theme);

    let eye = Button::new(core_ids::PAINTER_SIDEBAR_MODIFIER_SQUARE, "Eyedropper")
        .icon_only(IconId::EyePencil)
        .state(eye_state);
    paint_button(&eye, eye_rect, ctx.scene, ctx.text_system, theme);

    // Pigment toggle: Pressed (AccentSoft) while the brush is in Subtractive
    // (pigment) mode, else the dispatcher-tracked hover/press state.
    let pigment_state = if snapshot.pigment_enabled {
        ButtonState::Pressed
    } else {
        ctx.host
            .store()
            .button_state(core_ids::PAINTER_SIDEBAR_PIGMENT_TOGGLE)
            .unwrap_or(ButtonState::Normal)
    };
    let pigment = Button::new(core_ids::PAINTER_SIDEBAR_PIGMENT_TOGGLE, "Pigment")
        .icon_only(IconId::Palette)
        .state(pigment_state);
    paint_button(&pigment, pigment_rect, ctx.scene, ctx.text_system, theme);

    let hit_index = ctx.host.hit_index_mut();
    hit_index.register(core_ids::PAINTER_COLOR_THUMB, swatch_rect);
    hit_index.register(core_ids::PAINTER_SIDEBAR_MODIFIER_SQUARE, eye_rect);
    hit_index.register(core_ids::PAINTER_SIDEBAR_PIGMENT_TOGGLE, pigment_rect);
    // The brush-color swatch opens the Blender picker on Down via the generic
    // `is_picker_swatch` dispatch (pointer.rs) — register it each paint
    // (idempotent). Replaces the former per-id `PAINTER_COLOR_THUMB` special-case.
    ctx.host
        .store_mut()
        .register_picker_swatch(core_ids::PAINTER_COLOR_THUMB);
    y + ROW_H_PX + row_pad
}

/// Header dock-toggle ("Layers") — swaps the shared dock slot to the layers
/// panel (mode C). Placed left of the close button. Mirror do header toggle
/// ("Brush") em `ph2d-panel-painter-layers`. The toggle state lives on the
/// `PainterTool` (`dock_shows_layers`), flipped via `handle_panel_event`; the
/// shell `painter_bridge` reads it to drive panel visibility.
fn paint_dock_toggle(ctx: &mut PaintCtx, rect: Rect, theme: ph2d_tokens::Theme) {
    const TOGGLE_BTN_W: f32 = 52.0; // LITERAL-PX-OK: header dock-toggle button width
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
        .button_state(core_ids::PAINTER_SIDEBAR_TOGGLE_DOCK)
        .unwrap_or(ButtonState::Normal);
    let btn = Button::new(core_ids::PAINTER_SIDEBAR_TOGGLE_DOCK, "Layers").state(st);
    paint_button(&btn, btn_rect, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(core_ids::PAINTER_SIDEBAR_TOGGLE_DOCK, btn_rect);
}
