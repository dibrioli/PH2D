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
        // Eyedropper pick. The generic readback samples `vello_pass`'s intermediate texture — the
        // Vello UI layer, which is TRANSPARENT over the canvas (sprites live in the surface, not the
        // intermediate), so picking a painted pixel always returned transparent. When the Painter is
        // active and the click lands on the selected sprite, sample the painted layer COMPOSITE
        // instead (`PainterTool::sample_composite_at_uv`) so the eyedropper reads the real colour;
        // otherwise fall back to the rendered-overlay pixel.
        if let WidgetEvent::EyedropperPick { parent, px, py } = e {
            // Disjoint-field borrows (`hero` already holds `&mut gfx.hero_screen`): read the selection
            // + panel hit from `hero` first, then pass the render fields by value/ref to the helper.
            let selection = hero.gizmo.selection;
            let on_panel = hero.store.panel_at(px as f32, py as f32).is_some();
            let picked = painter_eyedropper_sample(
                &mut gfx.tools,
                &gfx.sim,
                &gfx.camera,
                gfx.surface.size(),
                selection,
                on_panel,
                px as f32,
                py as f32,
            )
            .or_else(|| gfx.vello_pass.read_pixel(gfx.surface.gpu(), px, py));
            if let Some([r, g, b, a]) = picked {
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
            // Focus/Blur land unhandled BY DESIGN (the store tracks focus internally; the hero has no
            // per-widget arm for them) — logging each number-input click drowned the console (Enio
            // 2026-07-07). Every OTHER unhandled event still prints: it's the dead-seam detector (a
            // painted-but-unwired widget shows up here first).
            if !matches!(e, WidgetEvent::Focus(_) | WidgetEvent::Blur(_)) {
                eprintln!("[hero] unhandled event: {e:?}");
            }
        }
    }
    // Palette Import / Export: a click on the picker's button flagged a host file-I/O request (the
    // picker can't open files). Import REPLACES the active palette from a chosen file; Export saves
    // it. The format is the file extension — .gpl / .hex / .ase / .aco (via `ph2d_color::palette`).
    if let Some((parent, io_kind)) = hero.store.take_palette_io_pending() {
        handle_palette_io(hero, parent, io_kind);
    }
    // Cross-session persistence: the palette CRUD (new / delete / select / import / swatch edits) is
    // pointer-driven, so this pointer-dispatch hook catches every change. Cheap hash-gate → save only
    // when the set actually changed.
    persist_palettes_if_changed(hero);
    reparent
}

thread_local! {
    /// Last-saved FNV hash of the named-palette set, so [`persist_palettes_if_changed`] writes only on
    /// a real change instead of every pointer event.
    static LAST_PALETTE_HASH: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Save the picker's named palettes to `~/.ph2d/palettes.txt` when they changed since the last call.
fn persist_palettes_if_changed(hero: &ph2d_editor::HeroScreen) {
    let Some(set) = hero
        .store
        .blender_palette_set(ph2d_editor::ids::INSP_BLENDER_PICKER)
    else {
        return;
    };
    let data: Vec<(String, Vec<[u8; 4]>)> = set
        .iter()
        .map(|p| (p.name.clone(), p.swatches.iter().map(|c| c.rgba).collect()))
        .collect();
    let h = crate::palette_persist::hash(&data);
    let changed = LAST_PALETTE_HASH.with(|c| {
        let changed = c.get() != h;
        c.set(h);
        changed
    });
    if changed {
        crate::palette_persist::save(&data);
    }
}

/// Service a pending palette import/export (opens an `rfd` file dialog, then applies via the
/// `ph2d_color::palette` engine). Split out of [`forward_to_hero`] to keep that hot path readable.
fn handle_palette_io(
    hero: &mut ph2d_editor::HeroScreen,
    parent: ph2d_editor::NodeId,
    io_kind: ph2d_editor::interaction::PaletteIoKind,
) {
    use ph2d_color::palette::{self, PaletteData, PaletteFormat};
    use ph2d_editor::interaction::PaletteIoKind;
    let fmt_of = |path: &std::path::Path| {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(PaletteFormat::from_extension)
            .unwrap_or(PaletteFormat::Gpl)
    };
    match io_kind {
        PaletteIoKind::Import => {
            let Some(path) = rfd::FileDialog::new()
                .add_filter(
                    "Colour palette",
                    &["gpl", "hex", "txt", "css", "ase", "aco"],
                )
                .pick_file()
            else {
                return;
            };
            match std::fs::read(&path).map(|b| palette::parse(fmt_of(&path), &b)) {
                Ok(Ok(p)) => {
                    let colors = p
                        .colors
                        .iter()
                        .map(|c| ph2d_tokens::ColorValue::from_rgba8(c[0], c[1], c[2], c[3]))
                        .collect();
                    // Import ADDS a named palette (the file's name) + activates it.
                    hero.store.blender_import_palette(parent, &p.name, colors);
                    hero.store.sync_blender_palette_name_buffer(parent);
                }
                Ok(Err(e)) => eprintln!("[ph2d] palette import: {e}"),
                Err(e) => eprintln!("[ph2d] palette read: {e}"),
            }
        }
        PaletteIoKind::Export => {
            let colors: Vec<[u8; 4]> = hero
                .store
                .blender_palette(parent)
                .map(|s| s.iter().map(|c| c.rgba).collect())
                .unwrap_or_default();
            if colors.is_empty() {
                return;
            }
            let Some(path) = rfd::FileDialog::new()
                .add_filter("GIMP palette", &["gpl"])
                .add_filter("Hex list", &["hex"])
                .add_filter("Adobe Swatch Exchange", &["ase"])
                .add_filter("Adobe Color", &["aco"])
                .set_file_name("palette.gpl")
                .save_file()
            else {
                return;
            };
            let data = PaletteData {
                name: "Palette".to_string(),
                colors,
            };
            if let Err(e) = std::fs::write(&path, palette::write(fmt_of(&path), &data)) {
                eprintln!("[ph2d] palette export: {e}");
            }
        }
    }
}

/// Sample the painted layer COMPOSITE under the screen pixel `(px, py)` for the colour-picker
/// eyedropper, when the Painter is active and the click lands on the selected sprite (not a panel).
/// Returns the displayed RGBA there, or `None` to fall back to the rendered-overlay readback — which
/// only has the Vello UI layer (transparent over the canvas). This is what integrates the eyedropper
/// with the layer system. Mirrors the footprint mapping in `painter_canvas_input` / the BgRemoval
/// eyedropper; takes disjoint `AppGfx` fields by ref so it composes with the live `&mut hero_screen`.
#[allow(clippy::too_many_arguments)]
fn painter_eyedropper_sample(
    tools: &mut ph2d_editor::ToolRegistry,
    sim: &ph2d_ecs::SimWorld,
    camera: &ph2d_render::Camera2d,
    window: ph2d_host::WindowSize,
    selection: Option<u64>,
    on_panel: bool,
    px: f32,
    py: f32,
) -> Option<[u8; 4]> {
    if on_panel {
        return None; // a click on a panel is panel-targeted, not a canvas sample
    }
    let painter_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("painter"))
        .unwrap_or(false);
    if !painter_active {
        return None;
    }
    let entity = ph2d_ecs::Entity::from_bits(selection?);
    let tr = sim.world().get::<crate::Transform>(entity)?;
    let sprite = sim.world().get::<ph2d_render::Sprite>(entity)?;
    let painter = tools
        .active_mut()?
        .as_any_mut()
        .downcast_mut::<ph2d_tool_painter::PainterTool>()?;
    let (iw, ih) = painter.canvas_size();
    if iw == 0 || ih == 0 {
        return None;
    }
    // Screen → image-px via the FULL sprite affine (size · scale · rotation · anchor · camera) — the
    // same geometry the brush uses, so the eyedropper tracks the sprite under any resize, AR change OR
    // rotation. `u`/`v` is the image fraction; not clamped (a Repeat-Image neighbour lands outside `[0,1]`).
    let affine = crate::render_loop::bgremoval_preview::sprite_image_to_screen_affine(
        iw, ih, tr, sprite, camera, window,
    );
    let img = affine.inverse() * ph2d_vector::Point::new(f64::from(px), f64::from(py));
    let (u, v) = (
        (img.x / f64::from(iw)) as f32,
        (img.y / f64::from(ih)) as f32,
    );
    // **Repeat Image**: the preview tiles the sprite 3×3, so the eyedropper works on any of the 8
    // neighbour tiles AND the original — accept the 3×3 UV grid and WRAP the sample back onto the
    // canvas (`rem_euclid`), exactly like the neighbour-paint hit region. Without Repeat, only the
    // central sprite is sampleable.
    let repeat = painter.repeat_image();
    let (lo, hi) = if repeat { (-1.0, 2.0) } else { (0.0, 1.0) };
    if !((lo..=hi).contains(&u) && (lo..=hi).contains(&v)) {
        return None; // outside the sampleable region → fall back to the rendered-overlay readback
    }
    let (su, sv) = if repeat {
        (u.rem_euclid(1.0), v.rem_euclid(1.0))
    } else {
        (u, v)
    };
    painter.sample_composite_at_uv(su, sv)
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
        AUDIO_EDITOR_PANEL, AUDIO_MIXER_PANEL, BGR_PANEL, CEQ_PANEL, EQS_PANEL, FLIP_PANEL,
        FLIP_STRIP_PANEL, GAL_PANEL, HIER_PANEL, INSP_PANEL, PAD_PANEL, PAINTER_LAYERS_PANEL,
        UPS_PANEL, VECTOR_PANEL,
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
        // Vector Style panel (right-dock takeover, ADR-0108). Same reason as
        // painter-layers: the canvas extends under the docked panel.
        || inside(VECTOR_PANEL)
        // Flip Style panel (right-dock takeover, ADR-0114 W2). Publishes a
        // scrollbar thumb (the Layers stack overflows), so it must intercept the
        // wheel — same bug otherwise (wheel zooms the camera under the panel).
        || inside(FLIP_PANEL)
        // Flip frame strip (bottom dock, W3): sem isto um clique numa CÉLULA cairia
        // no canvas atrás dela (o objeto Flip se estende por baixo da faixa) e o
        // gesto viraria um traço — a tira ficaria intocável.
        || inside(FLIP_STRIP_PANEL)
        // Audio Mixer + Audio Editor (Inspector-slot docks). Both publish a
        // scrollbar thumb, so both must intercept the wheel — a panel that scrolls
        // by thumb but zooms the camera on the wheel is the same bug twice. Missing
        // until 2026-07-09: the mixer's wheel silently zoomed the camera.
        // `shells/desktop/tests/scrollable_panels_intercept_the_wheel.rs` gates this.
        || inside(AUDIO_MIXER_PANEL)
        || inside(AUDIO_EDITOR_PANEL)
        // Motion Nodes graph panel (M1) — the bottom half of the center split.
        // Without it, wheel-over-graph zooms the camera instead of the graph
        // (the anchored graph zoom the M0 dispatch routes via `set_graph_canvas`).
        || inside(ph2d_editor::ids::MOTION_GRAPH_PANEL)
        // General timeline dock (W2.E6) — same reason as the graph: without it a
        // wheel over the dope-sheet zooms the CAMERA behind the panel instead of
        // the time axis (`set_timeline_canvas`), and the panel's zoom/pan is dead.
        || inside(ph2d_editor::ids::TIMELINE_PANEL)
}

/// **Is this point the editor's CHROME, rather than the artwork?** — the one door, and the pure half of
/// it (the `hit_plan` pattern: the decision is a tested function, the dispatch just asks it).
///
/// `panel` is what [`InteractionState::panel_at`] says, `widget` what [`HitIndex::hit`] says. The
/// question has **two halves and asking one is a bug that has now shipped three times**:
///
/// * `panel_at` covers panel **bodies** — the Inspector, the Hierarchy, the docked Layers/Vector panels.
///   It answers about *rects panels publish*.
/// * `hit_index` covers every interactive widget the chrome **painted, wherever it painted it**. The left
///   rail's buttons and the top bars are **not inside any panel rect**, so `panel_at` alone answers "not
///   chrome" over them — and the click falls straight through to the canvas behind. That is Enio's report
///   of 2026-07-16 (*"o clique sobre alguns botões pinta a pintura atrás de si"*), and it is the same bug
///   the C&F button was patched for one-off on 2026-07-03 (*"caiu em `painter_canvas_down` e a shape tool
///   largou um ponto no canvas atrás do botão"*) — patched **for that one id**, by hard-coding it.
///
/// ## The gizmo is NOT chrome, and that is the whole subtlety
///
/// A blanket *"the hit index claims this pixel ⇒ the UI owns it"* would be wrong and worse than the bug:
/// the gizmo is drawn **on the artwork**, around the very sprite being painted, and its handles sit on the
/// sprite's own corners and edges. Refusing there would carve dead zones out of the picture exactly where
/// the artist paints most. The gizmo answers for itself
/// ([`ph2d_editor::gizmo::is_gizmo_id`]) — the module that owns the ids owns the classification.
#[must_use]
pub fn chrome_claims(
    panel: Option<ph2d_editor::NodeId>,
    widget: Option<ph2d_editor::NodeId>,
) -> bool {
    if panel.is_some() {
        return true;
    }
    widget.is_some_and(|id| !ph2d_editor::gizmo::is_gizmo_id(id))
}

/// [`chrome_claims`], asked of the live hero screen at `(x, y)`.
///
/// Returns `false` with no hero (the demo's fixture mode paints no chrome, so the whole window is canvas).
#[must_use]
pub fn pointer_over_chrome(gfx: Option<&AppGfx>, x: f32, y: f32) -> bool {
    let Some(hero) = gfx.and_then(|g| g.hero_screen.as_ref()) else {
        return false;
    };
    chrome_claims(hero.store.panel_at(x, y), hero.hit_index.hit(x, y))
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

#[cfg(test)]
mod chrome_claims_tests {
    use super::chrome_claims;
    use ph2d_editor::NodeId;
    use ph2d_editor::gizmo::ids as gz;

    /// A left-rail button: no panel rect (the rail is not a panel), but the hit index claims it.
    /// This is Enio's report — `panel_at` alone answers "not the UI" here and the brush paints behind
    /// the button.
    #[test]
    fn a_rail_button_is_chrome_even_though_it_is_in_no_panel() {
        let rail_button = NodeId(ph2d_editor::ids::PAINTER_RAIL_FILL.0);
        assert!(
            chrome_claims(None, Some(rail_button)),
            "a button that belongs to no panel is still the editor's UI — the left rail and the top \
             bars publish no panel rect, and asking `panel_at` alone is what let the click through to \
             the artwork behind them"
        );
    }

    /// A panel body: the background between a panel's widgets. No widget under the cursor, but the
    /// press is still the panel's.
    #[test]
    fn a_panel_body_is_chrome_with_no_widget_under_the_cursor() {
        assert!(chrome_claims(Some(NodeId(1)), None));
    }

    /// **The gizmo is NOT chrome** — and this is the assertion that keeps the fix from being worse than
    /// the bug. The gizmo is drawn ON the artwork, around the very sprite being painted; its handles sit
    /// on the sprite's own corners and edges. A blanket "the hit index claims this pixel ⇒ the UI owns
    /// it" would carve dead zones out of the picture exactly where the artist paints most.
    ///
    /// The pivot dot rides along: `is_gizmo_handle_id` reports `false` for it (it begins no drag), which
    /// is why the door asks `is_gizmo_id` instead.
    #[test]
    fn the_gizmo_is_not_chrome_so_the_brush_can_paint_over_its_handles() {
        for (name, id) in [
            ("bbox interior", gz::GIZMO_BBOX_INTERIOR),
            ("corner handle", gz::GIZMO_HANDLE_TL),
            ("edge handle", gz::GIZMO_HANDLE_R),
            ("rotate region", gz::GIZMO_ROTATE_BR),
            ("pivot dot", gz::GIZMO_PIVOT),
        ] {
            assert!(
                !chrome_claims(None, Some(id)),
                "the gizmo's {name} was called chrome, so the brush would refuse to paint there — a dead \
                 zone on the sprite's own edge, which is worse than the bug this door exists to fix"
            );
        }
    }

    /// Bare canvas: nothing claims it.
    #[test]
    fn bare_canvas_is_not_chrome() {
        assert!(!chrome_claims(None, None));
    }

    /// A panel body wins even when the widget under the cursor is a gizmo id — the gizmo is never drawn
    /// inside a panel, so this is a nonsense state, and the honest answer to a nonsense state is the
    /// conservative one (the press is the panel's).
    #[test]
    fn a_panel_wins_over_a_gizmo_id() {
        assert!(chrome_claims(Some(NodeId(1)), Some(gz::GIZMO_HANDLE_TL)));
    }
}
