//! Equalize Sizes panel paint.
//!
//! Per-frame logic (mirrors the other typed panels):
//! - Visibility gate via [`PanelHostInternal::panel_visible`] +
//!   stale-rect cleanup on hide.
//! - Right-dock rect from `ctx.layout.padding` (Inspector slot).
//! - Chrome publish (`set_panel_rect`) so dispatch can hit-test it.
//! - Canonical chrome: dark-glass surface + corner dot, panel title,
//!   then sections:
//!     1. **Target** — 3 mode buttons (accent the active one); when
//!        Fixed → W and H chips; when GridUnit → slider + chip.
//!     2. **Upscale** — Upscale-if-smaller toggle; when on → 3
//!        algorithm buttons (accent active).
//!     3. **Rasterize** — single toggle.
//!     4. **Actions** — Cancel + Apply.
//! - Every painter is the SHARED source-of-truth from
//!   `panel_chrome` / `widget` — no panel-local widget look.

use crate::state::{self, EqualizeSizesPanelState, set_last_content_h, set_last_visible_h};
use crate::{EqualizeSizesPanel, ids};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{paint_text, paint_text_centered, rect_to_vello, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_corner_dot, paint_panel_corner_dot_bl, paint_panel_surface, paint_panel_title,
    panel_drag_handle_rect, panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, EQUALIZE_SIZES_SCROLLBAR_ID, TextInputState, paint_button,
    paint_number_chip, paint_scrollbar, paint_slider_with_chip_layout_adaptive,
    scrollbar_is_needed, scrollbar_thumb_rect, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};
use ph2d_tool_equalize_sizes::params::{EqualizeSizesUiSnapshot, TargetMode, UpscaleAlgorithm};
use ph2d_vector::VectorScene;

/// Label column width for Grid-mode slider rows.
const LABEL_COL_W: f32 = 72.0; // LITERAL-PX-OK: panel grid metric (per-panel label gutter width)

pub(crate) fn paint(_state: &mut EqualizeSizesPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(EqualizeSizesPanel::ID) {
        // Symmetric stale-rect cleanup so `panel_at` stops returning
        // EQS_PANEL once the tool is deactivated.
        ctx.host.store_mut().clear_panel_rect(ids::EQS_PANEL);
        return;
    }

    let rect: Rect = ctx.layout.padding;
    let theme = ctx.host.theme();
    let snapshot = state::current_snapshot();

    // Publish the rect so wheel/click dispatch can route to this panel.
    ctx.host.store_mut().set_panel_rect(ids::EQS_PANEL, rect);

    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);

    // Dock-slot drag + resize handles (shared with Inspector).
    {
        let drag_rect =
            panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
        let resize_rect = panel_resize_handle_rect(rect);
        let resize_bl_rect = panel_resize_handle_rect_bl(rect);
        let hit_index = ctx.host.hit_index_mut();
        hit_index.register(ph2d_editor_core::ids::INSP_DRAG_HANDLE, drag_rect);
        hit_index.register(ph2d_editor_core::ids::INSP_RESIZE_HANDLE, resize_rect);
        hit_index.register(ph2d_editor_core::ids::INSP_RESIZE_HANDLE_BL, resize_bl_rect);
    }

    let inner_x = rect.x + PANEL_HEAD_PAD;
    let inner_w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    let row_h = ROW_H_PX;
    let row_gap = Spacing::Sm.px();

    let title_size = paint_panel_title(
        rect,
        "Equalize Sizes",
        ph2d_editor_core::widget::panel_chrome::PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );

    // X close button → EQS_CANCEL (painted before clip so it sits on
    // chrome, not inside the scrollable body).
    ph2d_editor_core::widget::panel_chrome::paint_panel_close_button(
        rect,
        ids::EQS_CANCEL,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );
    // Color dot + notes intentionally NOT broadcast to image-tool panels.

    // Body region clipped + scrolled — conditional sub-sections (Fixed
    // chips, Grid offset + arrange toggle, upscale algorithm row) push
    // the body past dock height. Enio 2026-05-26 "padrão central do
    // app é painel com scroll". Wheel + scrollbar route via
    // `EQUALIZE_SIZES_SCROLLBAR_ID` → `EQS_PANEL`.
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    let scroll = ctx.host.store().panel_scroll(ids::EQS_PANEL);

    ctx.scene.push_clip(&rect_to_vello(body_rect));
    let y_after = paint_body_sections(
        ctx,
        &snapshot,
        inner_x,
        inner_w,
        row_h,
        row_gap,
        body_top - scroll,
    );
    let content_h = (y_after + scroll) - body_top + PANEL_HEAD_PAD;
    set_last_content_h(content_h);
    set_last_visible_h(body_h);
    ctx.scene.pop_layer();

    paint_scrollbar_and_publish(ctx, body_rect, content_h, body_h, scroll, theme);

    ctx.host.hit_index_mut().register(
        ids::EQS_CANCEL,
        ph2d_editor_core::widget::panel_chrome::panel_close_button_rect(rect),
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_body_sections(
    ctx: &mut PaintCtx,
    snapshot: &EqualizeSizesUiSnapshot,
    inner_x: f32,
    inner_w: f32,
    row_h: f32,
    row_gap: f32,
    y_in: f32,
) -> f32 {
    let theme = ctx.host.theme();
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let mut y = y_in;

    // ── Section: Target mode (3-way radio) ──────────────────────────
    let mode_row = Rect::new(inner_x, y, inner_w, row_h);
    paint_radio_row(
        mode_row,
        &[
            (
                "Max",
                ids::EQS_MODE_MAX,
                snapshot.target_mode == TargetMode::MaxOfSelection,
            ),
            (
                "Fixed",
                ids::EQS_MODE_FIXED,
                snapshot.target_mode == TargetMode::Fixed,
            ),
            (
                "Grid",
                ids::EQS_MODE_GRID,
                snapshot.target_mode == TargetMode::GridUnit,
            ),
        ],
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y += row_h + row_gap;

    // ── Mode-conditional rows ───────────────────────────────────────
    match snapshot.target_mode {
        TargetMode::Fixed => {
            let chip_gap = Spacing::Sm.px();
            let half = ((inner_w - chip_gap) * 0.5).max(0.0);
            // Manually paint a simple labeled chip pair (W=…, H=…). The
            // editor's shared `paint_slider_with_chip_layout` expects a
            // slider, so we inline a small chip row painter here.
            let used_w = paint_labeled_chip(
                Rect::new(inner_x, y, half, row_h),
                "W",
                ids::EQS_FIXED_W,
                snapshot.fixed_w as f64,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            );
            let used_h = paint_labeled_chip(
                Rect::new(inner_x + half + chip_gap, y, half, row_h),
                "H",
                ids::EQS_FIXED_H,
                snapshot.fixed_h as f64,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            );
            // ⚠️ **Avança pelo que foi USADO, e pelo MAIOR dos dois.** A versão anterior somava um
            // `row_h` fixo e ignorava o retorno do painter — e foi por aí que a linha seguinte
            // caiu por cima destes campos. *Uma altura devolvida e deitada fora é um layout que
            // acerta por coincidência.*
            y += used_w.max(used_h) + row_gap;
        }
        TargetMode::GridUnit => {
            // Cell size is owned by the Grid Snap tool. The shell
            // bridge syncs `snapshot.grid_unit` (px) from
            // `GridSnapState::square_cfg.cell_size * pixels_per_meter`
            // each frame, so this label always reflects the live cell.
            let info_text = format!("Cell: {} px (from Grid Snap)", snapshot.grid_unit);
            paint_text_centered(
                text_system,
                scene,
                &info_text,
                Rect::new(inner_x, y, inner_w, row_h),
                TypeToken::Xs.px(),
                resolve(ColorToken::Text2, theme),
            );
            y += row_h + row_gap;

            // Offset slider + chip — slider track `0..1` maps to
            // `0..(cell/2) px`; chip displays the raw px. Manual mirror
            // (different storage domains) lives in `event::apply_event`.
            // `Final size: (cell - offset) x (cell - offset)` row sits
            // right under the slider as in the legacy `EqualizeModal`.
            let max_off = (snapshot.grid_unit / 2).max(1);
            let track = store
                .slider(ids::EQS_GRID_OFFSET)
                .map(|(_, v)| v)
                .unwrap_or_else(|| snapshot.grid_offset as f32 / max_off as f32);
            let chip_value = store
                .number_value(ids::EQS_GRID_OFFSET_NUM)
                .unwrap_or(snapshot.grid_offset as f64);
            // Canonical chip width — 72 px (was 32, user 2026-05-24).
            let chip_w = ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX;
            let display = format!("{} px", chip_value.round() as i64);
            let used = paint_slider_with_chip_layout_adaptive(
                Rect::new(inner_x, y, inner_w, row_h),
                "Offset",
                track,
                chip_value,
                Some(&display),
                ids::EQS_GRID_OFFSET,
                ids::EQS_GRID_OFFSET_NUM,
                LABEL_COL_W,
                chip_w,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            );
            y += used + row_gap;

            let final_dim = snapshot
                .grid_unit
                .saturating_sub(snapshot.grid_offset.min(max_off))
                .max(1);
            let final_text = format!("Final size: {final_dim} x {final_dim} px");
            paint_text_centered(
                text_system,
                scene,
                &final_text,
                Rect::new(inner_x, y, inner_w, row_h),
                TypeToken::Xs.px(),
                resolve(ColorToken::Text2, theme),
            );
            y += row_h + row_gap;

            // "Arrange on Grid (1 per cell)" toggle — port of legacy
            // `EqualizeModal.arrangeOnGrid`. When on, Apply lays the
            // selection out 1-sprite-per-cell sorted by world `(y, x)`.
            paint_toggle_button(
                Rect::new(inner_x, y, inner_w, row_h),
                "Arrange on Grid (1 per cell)",
                ids::EQS_ARRANGE_ON_GRID,
                snapshot.arrange_on_grid,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            );
            y += row_h + row_gap;
        }
        TargetMode::MaxOfSelection => {
            // No extra row — preview text (Final size: …) is optional v2.
        }
    }

    y += row_gap;

    // ── Section: Upscale if smaller (accent toggle) ─────────────────
    let upscale_on = snapshot.upscale_if_smaller;
    paint_toggle_button(
        Rect::new(inner_x, y, inner_w, row_h),
        "Upscale if smaller",
        ids::EQS_UPSCALE_IF_SMALLER,
        upscale_on,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y += row_h + row_gap;

    if upscale_on {
        let alg_row = Rect::new(inner_x, y, inner_w, row_h);
        paint_radio_row(
            alg_row,
            &[
                (
                    "Lanczos",
                    ids::EQS_ALG_LANCZOS,
                    snapshot.upscale_algorithm == UpscaleAlgorithm::Lanczos3,
                ),
                (
                    "Nearest",
                    ids::EQS_ALG_NEAREST,
                    snapshot.upscale_algorithm == UpscaleAlgorithm::Nearest,
                ),
                (
                    "xBR",
                    ids::EQS_ALG_XBR,
                    snapshot.upscale_algorithm == UpscaleAlgorithm::Xbr,
                ),
            ],
            store,
            hit_index,
            scene,
            text_system,
            theme,
        );
        y += row_h + row_gap;
    }

    y += row_gap;

    // ── Section: Rasterize after (accent toggle) ────────────────────
    paint_toggle_button(
        Rect::new(inner_x, y, inner_w, row_h),
        "Rasterize after",
        ids::EQS_RASTERIZE_AFTER,
        snapshot.rasterize_after,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y += row_h + row_gap;

    y += row_gap;

    // As duas linhas de AÇÃO moram num IRMÃO — ver [`crate::paint_actions`].
    crate::paint_actions::paint_action_rows(
        scene,
        text_system,
        theme,
        store,
        hit_index,
        inner_x,
        inner_w,
        row_h,
        row_gap,
        y,
    )
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
        paint_scrollbar(
            body_rect,
            scroll,
            content_h,
            body_h,
            ctx.host
                .store()
                .scrollbar_visual(EQUALIZE_SIZES_SCROLLBAR_ID),
            ctx.scene,
            theme,
        );
        ctx.host
            .hit_index_mut()
            .register(EQUALIZE_SIZES_SCROLLBAR_ID, thumb);
    }
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::EQS_PANEL, content_h);
    store.set_panel_visible_h(ids::EQS_PANEL, body_h);
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(ids::EQS_PANEL) > max_scroll {
        store.set_panel_scroll(ids::EQS_PANEL, max_scroll);
    }
}

/// Paint a horizontal row of N equal-width buttons that behave as a
/// radio group (active one is `ButtonKind::Accent`, others
/// `ButtonKind::Default`). The Tool's `handle_panel_event` does the
/// actual selection — this is paint-only.
#[allow(clippy::too_many_arguments)]
fn paint_radio_row(
    rect: Rect,
    items: &[(&str, NodeId, bool)],
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    if items.is_empty() {
        return;
    }
    let gap = Spacing::Sm.px();
    let n = items.len() as f32;
    let item_w = ((rect.w - gap * (n - 1.0)) / n).max(0.0);
    for (i, (label, id, active)) in items.iter().enumerate() {
        let item_rect = Rect::new(rect.x + (item_w + gap) * i as f32, rect.y, item_w, rect.h);
        let kind = if *active {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        };
        let btn_state = if *active {
            (ButtonState::Pressed, ph2d_editor_core::motion::SETTLED)
        } else {
            store.button_visual(*id)
        };
        let b = Button::new(*id, *label).kind(kind).visual(btn_state);
        paint_button(&b, item_rect, scene, text_system, theme);
        hit_index.register(*id, item_rect);
    }
}

/// Paint a single accent-when-on toggle button.
#[allow(clippy::too_many_arguments)]
fn paint_toggle_button(
    rect: Rect,
    label: &str,
    id: NodeId,
    on: bool,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let kind = if on {
        ButtonKind::Accent
    } else {
        ButtonKind::Default
    };
    let btn_state = if on {
        (ButtonState::Pressed, ph2d_editor_core::motion::SETTLED)
    } else {
        store.button_visual(id)
    };
    let b = Button::new(id, label).kind(kind).visual(btn_state);
    paint_button(&b, rect, scene, text_system, theme);
    hit_index.register(id, rect);
}

/// Paint a label + NumberInput chip pair on one row (no slider). The
/// chip uses the stored number_value (already mirrored by the host on
/// **Um rótulo por cima do seu campo numérico**, e a altura que ele de facto usou.
///
/// ⚠️ **Isto reusava o painter de SLIDER com um track de largura zero**, e o comentário de então
/// admitia o truque: *"o slider colapsa atrás da coluna do rótulo"*. Ele colapsava **enquanto a
/// row coubesse numa linha**. Num painel estreito — que é o caso destas duas metades — o painter
/// adaptativo EMPILHA, e aí o truque desmonta-se de duas maneiras ao mesmo tempo (Enio,
/// 2026-08-19: *"o painel de equalize sizes: fixed está todo embolado"*):
///
/// 1. o track deixa de estar escondido atrás do rótulo e desenha-se **à largura toda** — o
///    retângulo preto à esquerda do `256`;
/// 2. a row passa a ocupar **duas** linhas, e quem a chamava avançava `y` por **uma** — a linha
///    seguinte (`Upscale if smaller`) caía por cima dos campos.
///
/// A cura não é medir melhor a altura: é **não pedir um slider quando não há slider**. O
/// `paint_number_chip` existe exatamente para isto — o doc dele diz *"callable directly when a
/// chip needs to live somewhere a slider row layout doesn't fit"*.
///
/// *Um componente reusado com um dos seus eixos posto a zero não é reuso; é um caso especial à
/// espera do primeiro layout que não o respeite.*
#[allow(clippy::too_many_arguments)]
fn paint_labeled_chip(
    rect: Rect,
    label: &str,
    chip_id: NodeId,
    value: f64,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let font = TypeToken::Xs.px();
    let label_h = font + Spacing::Xs.px();
    paint_text(
        text_system,
        scene,
        label,
        rect.x,
        rect.y,
        font,
        rect.w,
        resolve(ColorToken::Text2, theme),
    );
    let chip_rect = Rect::new(rect.x, rect.y + label_h, rect.w, rect.h);
    // O estado vem do store para que a escrita, o cursor e a seleção sejam vivos — a mesma
    // leitura que o `paint_slider_with_chip_layout` faz do seu chip.
    let (state, buffer, caret, anchor) = match store.get(chip_id) {
        Some(InteractiveState::NumberInput {
            state,
            buffer,
            caret,
            selection_anchor,
            ..
        }) => (*state, Some(buffer.as_str()), *caret, *selection_anchor),
        _ => (TextInputState::Normal, None, 0, None),
    };
    let display = value.round().to_string();
    paint_number_chip(
        chip_rect,
        state,
        value,
        Some(&display),
        buffer,
        caret,
        anchor,
        scene,
        text_system,
        theme,
    );
    hit_index.register(chip_id, chip_rect);
    label_h + rect.h
}
