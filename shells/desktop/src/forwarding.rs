//! Forward winit-derived input events into the hero screen's
//! interaction dispatcher.
//!
//! Extracted from [`main`] (Track B7). Five thin orchestrators that
//! each take `Option<&mut AppGfx>`, call into the editor's
//! `dispatch_*` entry point, and apply the emitted `WidgetEvent`s.
//! Also bridges Cmd+C/X/V → `arboard` (OS clipboard) for the key
//! forwarder.

use crate::AppGfx;
use ph2d_editor::WidgetEvent;
use ph2d_editor::interaction::PainterLayerDrop;
use ph2d_host::{KeyEvent, PointerEvent};

/// Forward a pointer event to the hero screen's interaction
/// dispatcher when the hero is active. Drains emitted
/// [`WidgetEvent`]s into `HeroScreen::apply_event` (consumed events
/// drive hero-level state mutations) and logs unconsumed ones to
/// stderr for the developer to verify wiring.
///
/// Returns a `PainterLayerReparent` payload `(dragged, drop)` when the
/// drag dispatch emitted one — the caller (which holds the `ToolRegistry`)
/// routes it to the active `PainterTool::handle_layer_reparent`. `None`
/// otherwise. (The hero/chrome can't own this: it's a tool mutation, and
/// `forward_to_hero` has no `ToolRegistry`.)
#[must_use]
pub fn forward_to_hero(
    gfx: Option<&mut AppGfx>,
    event: PointerEvent,
) -> Option<(ph2d_editor::NodeId, PainterLayerDrop)> {
    let gfx = gfx?;
    let hero = gfx.hero_screen.as_mut()?;
    // Snapshot events before applying — apply_event may mutate hero,
    // but the events slice itself lives in the arena (immutable view).
    // Threads the live TextSystem so click→caret on text widgets
    // snaps to the nearest glyph boundary (real measurement) instead
    // of the dispatch's char-count heuristic.
    let snapshot: Vec<WidgetEvent> = hero
        .handle_pointer_with_text(event, &mut gfx.text_system, &gfx.hero_arena)
        .to_vec();
    let mut reparent = None;
    for e in snapshot {
        // Eyedropper pick — read the rendered pixel at the click
        // position from vello_pass's intermediate texture and apply
        // it to the picker. Only the host can do this (the dispatch
        // has no GPU access); intercept before `apply_event`.
        if let WidgetEvent::EyedropperPick { parent, px, py } = e {
            if let Some([r, g, b, a]) = gfx.vello_pass.read_pixel(gfx.surface.gpu(), px, py) {
                hero.store
                    .set_blender_value(parent, ph2d_tokens::ColorValue::from_rgba8(r, g, b, a));
            }
            continue;
        }
        // Painter layers drag-reparent (W3 T3.8): surface to the caller,
        // which holds the `ToolRegistry`, to apply on the active PainterTool.
        if let WidgetEvent::PainterLayerReparent { dragged, drop } = e {
            reparent = Some((dragged, drop));
            continue;
        }
        if !hero.apply_event(e) {
            eprintln!("[hero] unhandled event: {e:?}");
        }
    }
    reparent
}

/// Forward a translated [`KeyEvent`] (with editor-canonical
/// `keycode` from [`crate::keymap::winit_to_editor_keycode`]) into
/// the hero dispatcher so focused widgets see Tab/Enter/Backspace/
/// arrows etc. Also drains any clipboard copy/paste requests the
/// dispatcher set for this key event and bridges to `arboard`.
pub fn forward_key_to_hero(gfx: Option<&mut AppGfx>, event: KeyEvent) {
    let Some(gfx) = gfx else { return };
    let Some(hero) = gfx.hero_screen.as_mut() else {
        return;
    };
    let snapshot: Vec<WidgetEvent> = hero.handle_key(event, &gfx.hero_arena).to_vec();
    for e in snapshot {
        if !hero.apply_event(e) {
            eprintln!("[hero] unhandled key event: {e:?}");
        }
    }
    // Drain clipboard requests set by Cmd+C / Cmd+X / Cmd+V.
    if let Some(text) = hero.store.take_clipboard_copy()
        && let Some(cb) = gfx.clipboard.as_mut()
        && let Err(err) = cb.set_text(text)
    {
        eprintln!("[ph2d] clipboard set_text failed: {err}");
    }
    if let Some(target) = hero.store.take_clipboard_paste_request() {
        let text = gfx
            .clipboard
            .as_mut()
            .and_then(|cb| cb.get_text().ok())
            .unwrap_or_default();
        if !text.is_empty()
            && ph2d_editor::interaction::apply_clipboard_paste(&mut hero.store, target, &text)
        {
            // Mimic the TextChanged path so sliders/links update.
            let _ = hero.apply_event(WidgetEvent::TextChanged(target));
        }
    }
}

/// Forward a wheel / trackpad scroll into the hero dispatcher.
/// Routes to whichever panel registered its rect under the cursor.
pub fn forward_wheel_to_hero(gfx: Option<&mut AppGfx>, event: ph2d_host::WheelEvent) {
    let Some(gfx) = gfx else { return };
    let Some(hero) = gfx.hero_screen.as_mut() else {
        return;
    };
    let _ = hero.handle_wheel(event, &gfx.hero_arena);
}

/// Forward a single printable character into the hero text-input
/// dispatcher (focused TextInput/NumberInput/Combobox buffer).
pub fn forward_text_to_hero(gfx: Option<&mut AppGfx>, ch: char) {
    let Some(gfx) = gfx else { return };
    let Some(hero) = gfx.hero_screen.as_mut() else {
        return;
    };
    let snapshot: Vec<WidgetEvent> = hero.handle_text_input(ch, &gfx.hero_arena).to_vec();
    for e in snapshot {
        if !hero.apply_event(e) {
            eprintln!("[hero] unhandled text-input event: {e:?}");
        }
    }
}

/// M14.4b.bis: true when `(x, y)` lies inside either the Inspector
/// or Hierarchy panel rect published by the most-recent
/// `paint_hero_screen` pass. Used to decide whether a mouse-wheel
/// event should zoom the camera (over canvas) or scroll a panel
/// (over a panel).
///
/// Returns false when no hero is active — the demo's fixture mode
/// shows raw sprites with no panels, so the whole window is "canvas"
/// and wheel zooms the camera.
pub fn cursor_over_hero_panel(gfx: Option<&AppGfx>, x: f32, y: f32) -> bool {
    let Some(gfx) = gfx else { return false };
    let Some(hero) = gfx.hero_screen.as_ref() else {
        return false;
    };
    use ph2d_editor::screens::hero::ids::{
        BGR_PANEL, CEQ_PANEL, EQS_PANEL, GAL_PANEL, HIER_PANEL, INSP_PANEL, PAD_PANEL,
        PAINTER_LAYERS_PANEL, UPS_PANEL,
    };
    let inside = |panel_id| {
        hero.store
            .panel_rect(panel_id)
            .map(|r| r.contains(x, y))
            .unwrap_or(false)
    };
    // Every panel that publishes a rect intercepts the wheel so it
    // scrolls the panel body instead of zooming the camera underneath.
    // `panel_rect(...)` is only published while the panel is visible,
    // so each check is false in the panel's closed state. Image-tool
    // panels (CEQ/BGR/PAD/UPS/EQS) added 2026-05-24 — without them
    // CEQ's wheel routed to camera zoom instead of panel scroll.
    inside(INSP_PANEL)
        || inside(HIER_PANEL)
        || inside(GAL_PANEL)
        || inside(ph2d_editor::grid_snap::ids::GS_PANEL)
        || inside(BGR_PANEL)
        || inside(PAD_PANEL)
        || inside(CEQ_PANEL)
        || inside(UPS_PANEL)
        || inside(EQS_PANEL)
        // Painter layers panel (right-dock takeover). Without it, a Primary
        // Down / wheel over the panel falls through to the canvas behind it
        // (the sprite footprint extends UNDER the docked panel), so the click
        // would interact with the sprite "through" the panel chrome.
        || inside(PAINTER_LAYERS_PANEL)
}

/// ADR-0029 Phase C.2: resolve canvas-picked entity bits to a live
/// Hierarchy entry via the panel-owned thread-local snapshot. Takes
/// the `hero_live` field directly (not `&AppGfx`) so the caller can
/// keep a `&mut gfx.hero_screen` borrow live alongside this read.
#[cfg(feature = "panel-hierarchy")]
pub(crate) fn resolve_live_entry(
    hero_live: Option<&crate::HeroLive>,
    picked: Option<u64>,
) -> Option<ph2d_editor::screens::hero::fixture::HierarchyEntity> {
    let node = hero_live?.bridge.node_for(picked?)?;
    ph2d_panel_hierarchy::current_live_entries()?
        .get(&node)
        .cloned()
}
#[cfg(not(feature = "panel-hierarchy"))]
pub(crate) fn resolve_live_entry(
    _hero_live: Option<&crate::HeroLive>,
    _picked: Option<u64>,
) -> Option<ph2d_editor::screens::hero::fixture::HierarchyEntity> {
    None
}
