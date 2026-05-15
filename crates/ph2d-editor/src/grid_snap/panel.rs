//! Floating-panel paint + event handler for the grid-snap subsystem.
//!
//! Layout: vertical stack of title bar / kind section / snap section /
//! display section. Drag / resize handles match
//! [`crate::screens::hero::widget_gallery`] so the existing
//! `BlenderHitKind::DragHandle` / `ResizeHandle` dispatch moves it.
//!
//! v1 simplifies interactive widgets: kind / snap target use cycling
//! [`crate::widget::Button`]s ("Kind: Square \u{25B6}") in place of a
//! full [`crate::widget::Dropdown`]; opacity surfaces as a label
//! ("Opacity: 75%") instead of a slider. Coordenador's integration
//! wires v2 widgets after Inspect (Stage 11) lands.

use super::ids;
use super::state::{GridKind, GridSnapState};
use crate::interaction::{BlenderHitKind, HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::widget::{
    Button, ButtonKind, ButtonState, SectionHeader, Toggle, ToggleState, paint_button,
    paint_section_header, paint_toggle,
};
use crate::zones::Rect;
use ph2d_grid::snap::SnapTarget;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

const ROW_H: f32 = 28.0;
const SECTION_HEADER_H: f32 = 22.0;
const TITLE_BAR_H: f32 = 32.0;
const PAD: f32 = 8.0;
const ROW_GAP: f32 = 4.0;
const LABEL_FONT_SIZE: f32 = 13.0;

/// Register the panel's interactive nodes in `store`. Called once
/// from `HeroScreen::pre_populate_store` (Coordenador wiring).
pub fn populate(store: &mut WidgetStore) {
    // Panel chrome — Blender drag/resize handles same as Widget Gallery.
    store.register(
        ids::GS_DRAG_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::GS_PANEL,
            kind: BlenderHitKind::DragHandle,
        },
    );
    store.register(
        ids::GS_RESIZE_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::GS_PANEL,
            kind: BlenderHitKind::ResizeHandle,
        },
    );
    // Plain buttons (close + cycling kind/target).
    for id in [ids::GS_CLOSE, ids::GS_KIND_DROPDOWN] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Toggles — start values match GridSnapState::default() so
    // the painted toggle thumb matches state until the first event.
    store.register(
        ids::GS_SNAP_ENABLED,
        InteractiveState::Toggle {
            state: ToggleState::Normal,
            on: false,
        },
    );
    store.register(
        ids::GS_SHOW_OVERLAY,
        InteractiveState::Toggle {
            state: ToggleState::Normal,
            on: true,
        },
    );
}

/// Default panel rect when first opened — centered horizontally on
/// the viewport, fixed width, height sized for title + 3 config
/// sections + inspect.
pub fn default_rect(viewport_w: f32, viewport_h: f32) -> Rect {
    let w = 320.0_f32.min(viewport_w - 16.0);
    let h = 540.0_f32.min(viewport_h - 16.0).max(380.0);
    let x = ((viewport_w - w) * 0.5).max(8.0);
    let y = ((viewport_h - h) * 0.5).max(8.0);
    Rect::new(x, y, w, h)
}

/// Paint the panel into `rect`. Reads `state` for current values;
/// mutations flow through [`apply_event`] from the dispatcher.
pub fn paint(
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    _hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) {
    // Body fill + border.
    fill_rounded_rect(
        scene,
        rect,
        Radius::Md.px(),
        resolve(ColorToken::Bg2, theme),
    );
    stroke_rounded_rect(
        scene,
        rect,
        Radius::Md.px(),
        1.0,
        resolve(ColorToken::Border, theme),
    );

    let inner_x = rect.x + PAD;
    let inner_w = rect.w - PAD * 2.0;
    let mut y = rect.y + PAD;

    // ─── Title bar ───────────────────────────────────────────────
    paint_title_bar(
        Rect::new(inner_x, y, inner_w, TITLE_BAR_H),
        scene,
        text_system,
        theme,
        store,
    );
    y += TITLE_BAR_H + ROW_GAP;

    // ─── Section: Grid Kind ─────────────────────────────────────
    y = paint_section_label("Grid Kind", inner_x, inner_w, y, scene, text_system, theme);
    paint_kind_row(
        Rect::new(inner_x, y, inner_w, ROW_H),
        scene,
        text_system,
        theme,
        store,
        state,
    );
    y += ROW_H + ROW_GAP * 2.0;

    // ─── Section: Snap ──────────────────────────────────────────
    y = paint_section_label("Snap", inner_x, inner_w, y, scene, text_system, theme);
    paint_snap_enabled_row(
        Rect::new(inner_x, y, inner_w, ROW_H),
        scene,
        text_system,
        theme,
        store,
        state,
    );
    y += ROW_H + ROW_GAP;
    paint_snap_target_row(
        Rect::new(inner_x, y, inner_w, ROW_H),
        scene,
        text_system,
        theme,
        store,
        state,
    );
    y += ROW_H + ROW_GAP * 2.0;

    // ─── Section: Display ───────────────────────────────────────
    y = paint_section_label("Display", inner_x, inner_w, y, scene, text_system, theme);
    paint_show_overlay_row(
        Rect::new(inner_x, y, inner_w, ROW_H),
        scene,
        text_system,
        theme,
        store,
        state,
    );
    y += ROW_H + ROW_GAP;
    paint_opacity_label_row(
        Rect::new(inner_x, y, inner_w, ROW_H),
        scene,
        text_system,
        theme,
        state,
    );
    y += ROW_H + ROW_GAP * 2.0;

    // ─── Section: Inspect ───────────────────────────────────────
    let inspect_h = super::inspect::height();
    super::inspect::paint(
        Rect::new(inner_x, y, inner_w, inspect_h),
        scene,
        text_system,
        theme,
        state,
    );

    // ─── Resize grip (bottom-right) ─────────────────────────────
    let grip = Rect::new(rect.x + rect.w - 14.0, rect.y + rect.h - 14.0, 12.0, 12.0);
    stroke_rounded_rect(
        scene,
        grip,
        Radius::Sm.px(),
        1.0,
        resolve(ColorToken::BorderEmph, theme),
    );
}

fn paint_title_bar(
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &WidgetStore,
) {
    fill_rounded_rect(
        scene,
        rect,
        Radius::Sm.px(),
        resolve(ColorToken::Bg3, theme),
    );

    // Title text.
    paint_text(
        text_system,
        scene,
        "Grid Settings", // TODO(i18n)
        rect.x + Spacing::Sm.px(),
        rect.y + (rect.h - LABEL_FONT_SIZE) * 0.5,
        LABEL_FONT_SIZE,
        rect.w - 40.0,
        resolve(ColorToken::Text1, theme),
    );

    // Close button on the right.
    let btn_size = 24.0;
    let close_rect = Rect::new(
        rect.x + rect.w - btn_size - 4.0,
        rect.y + (rect.h - btn_size) * 0.5,
        btn_size,
        btn_size,
    );
    let close_btn = Button {
        id: ids::GS_CLOSE,
        label: String::new(),
        state: button_state(store, ids::GS_CLOSE),
        kind: ButtonKind::IconOnly {
            icon: crate::icons::IconId::Close,
        },
    };
    paint_button(&close_btn, close_rect, scene, text_system, theme);
}

fn paint_section_label(
    label: &str,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let rect = Rect::new(x, y, w, SECTION_HEADER_H);
    let header = SectionHeader {
        id: crate::NodeId(0),
        label: label.to_string(),
        count: None,
        collapsible: None,
        color: None,
    };
    paint_section_header(&header, rect, scene, text_system, theme);
    y + SECTION_HEADER_H + ROW_GAP
}

fn paint_kind_row(
    row: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &WidgetStore,
    state: &GridSnapState,
) {
    let btn = Button {
        id: ids::GS_KIND_DROPDOWN,
        label: format!("Kind: {} \u{25B6}", state.kind.label()),
        state: button_state(store, ids::GS_KIND_DROPDOWN),
        kind: ButtonKind::Default,
    };
    paint_button(&btn, row, scene, text_system, theme);
}

fn paint_snap_enabled_row(
    row: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &WidgetStore,
    state: &GridSnapState,
) {
    paint_labeled_toggle(
        "Snap",
        ids::GS_SNAP_ENABLED,
        state.snap_enabled,
        row,
        scene,
        text_system,
        theme,
        store,
    );
}

fn paint_snap_target_row(
    row: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &WidgetStore,
    state: &GridSnapState,
) {
    // Cycling between Center / Intersection — wired in apply_event
    // under GS_SNAP_CENTER and GS_SNAP_INTERSECTION (registered for
    // hit-test even though only one is "active" per click cycle).
    let target_label = match state.snap_target {
        SnapTarget::Center => "Center",
        SnapTarget::Intersection => "Intersection",
    };
    // Reuse the kind-dropdown id slot conceptually; here use a
    // dedicated cycling button at the snap-center id.
    let btn = Button {
        id: ids::GS_SNAP_CENTER,
        label: format!("Target: {target_label} \u{25B6}"),
        state: button_state(store, ids::GS_SNAP_CENTER),
        kind: ButtonKind::Default,
    };
    paint_button(&btn, row, scene, text_system, theme);
}

fn paint_show_overlay_row(
    row: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &WidgetStore,
    state: &GridSnapState,
) {
    paint_labeled_toggle(
        "Show overlay",
        ids::GS_SHOW_OVERLAY,
        state.show_overlay,
        row,
        scene,
        text_system,
        theme,
        store,
    );
}

fn paint_opacity_label_row(
    row: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    state: &GridSnapState,
) {
    let pct = (state.opacity * 100.0).round() as u32;
    paint_text(
        text_system,
        scene,
        &format!("Opacity: {pct}%"),
        row.x + Spacing::Sm.px(),
        row.y + (row.h - LABEL_FONT_SIZE) * 0.5,
        LABEL_FONT_SIZE,
        row.w,
        resolve(ColorToken::Text2, theme),
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_labeled_toggle(
    label: &str,
    id: crate::NodeId,
    on: bool,
    row: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &WidgetStore,
) {
    paint_text(
        text_system,
        scene,
        label,
        row.x + Spacing::Sm.px(),
        row.y + (row.h - LABEL_FONT_SIZE) * 0.5,
        LABEL_FONT_SIZE,
        row.w - 60.0,
        resolve(ColorToken::Text1, theme),
    );

    let toggle_w = 40.0;
    let toggle_h = 20.0;
    let toggle_rect = Rect::new(
        row.x + row.w - toggle_w - 4.0,
        row.y + (row.h - toggle_h) * 0.5,
        toggle_w,
        toggle_h,
    );
    let toggle = Toggle {
        id,
        label: String::new(),
        on,
        state: toggle_state(store, id),
    };
    paint_toggle(&toggle, toggle_rect, scene, theme);
}

fn button_state(store: &WidgetStore, id: crate::NodeId) -> ButtonState {
    match store.get(id) {
        Some(InteractiveState::Button { state }) => *state,
        _ => ButtonState::Normal,
    }
}

fn toggle_state(store: &WidgetStore, id: crate::NodeId) -> ToggleState {
    match store.get(id) {
        Some(InteractiveState::Toggle { state, .. }) => *state,
        _ => ToggleState::Normal,
    }
}

/// Handle a widget event for the grid-snap panel. Returns `true`
/// when the event mutates `state`; the caller (Coordenador-wired
/// `HeroScreen::apply_event`) uses the return to stop dispatch.
pub fn apply_event(state: &mut GridSnapState, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::GS_CLOSE {
        state.panel_visible = false;
        return true;
    }
    if id == ids::GS_KIND_DROPDOWN {
        state.kind = cycle_kind(state.kind);
        return true;
    }
    if id == ids::GS_SNAP_ENABLED {
        state.snap_enabled = !state.snap_enabled;
        return true;
    }
    if id == ids::GS_SHOW_OVERLAY {
        state.show_overlay = !state.show_overlay;
        return true;
    }
    if id == ids::GS_SNAP_CENTER {
        // Cycling toggle for the target row.
        state.snap_target = match state.snap_target {
            SnapTarget::Center => SnapTarget::Intersection,
            SnapTarget::Intersection => SnapTarget::Center,
        };
        return true;
    }
    false
}

fn cycle_kind(k: GridKind) -> GridKind {
    let all = GridKind::all();
    let idx = all.iter().position(|x| *x == k).unwrap_or(0);
    all[(idx + 1) % all.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_kind_wraps() {
        let mut k = GridKind::Square;
        for _ in 0..GridKind::all().len() {
            k = cycle_kind(k);
        }
        assert_eq!(k, GridKind::Square, "9 cycles returns to start");
    }

    #[test]
    fn apply_event_close_clears_visible() {
        let mut s = GridSnapState {
            panel_visible: true,
            ..Default::default()
        };
        assert!(apply_event(&mut s, WidgetEvent::Click(ids::GS_CLOSE)));
        assert!(!s.panel_visible);
    }

    #[test]
    fn apply_event_kind_cycles() {
        let mut s = GridSnapState::default();
        assert_eq!(s.kind, GridKind::Square);
        apply_event(&mut s, WidgetEvent::Click(ids::GS_KIND_DROPDOWN));
        assert_eq!(s.kind, GridKind::Hex);
    }

    #[test]
    fn apply_event_snap_toggle() {
        let mut s = GridSnapState::default();
        apply_event(&mut s, WidgetEvent::Click(ids::GS_SNAP_ENABLED));
        assert!(s.snap_enabled);
        apply_event(&mut s, WidgetEvent::Click(ids::GS_SNAP_ENABLED));
        assert!(!s.snap_enabled);
    }

    #[test]
    fn apply_event_unrelated_id_returns_false() {
        let mut s = GridSnapState::default();
        let snapshot_kind = s.kind;
        let snapshot_visible = s.panel_visible;
        let unrelated = crate::NodeId(42);
        assert!(!apply_event(&mut s, WidgetEvent::Click(unrelated)));
        assert_eq!(s.kind, snapshot_kind);
        assert_eq!(s.panel_visible, snapshot_visible);
    }
}
