//! Window-event dispatch — one method per `WindowEvent` variant.
//!
//! PR 9b of `docs/Migracao/2026-05-convention-by-discovery.md`:
//! `window_event()` in `main.rs` used to inline ~700 LOC across 13
//! `WindowEvent` arms, with single arms (CursorMoved 166 LOC,
//! MouseInput 325 LOC, KeyboardInput 83 LOC) violating HR-18's 200-LOC
//! per-function cap. This module hosts each arm as a `pub(crate) fn
//! on_<arm>(&mut self, …)` method on `App` — bodies are verbatim
//! former arms (no behaviour change), so smoke parity is
//! byte-for-byte.
//!
//! `window_event()` in `main.rs` becomes a 13-line dispatch table.
//! Adding a new arm: one method here + one line in the table.
//!
//! Rust allows `impl App` to be split across files within the same
//! crate as long as both files are reachable via `mod`. `App` is
//! `private` to `main.rs`, but submodules see their parent's private
//! items — so this `impl` block compiles without exposing any field
//! visibility upstream.

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::event_loop::ActiveEventLoop;

use ph2d_editor::Toast;
use ph2d_host::{
    CloseAction, HostHandler, Lifecycle, PlatformHost, PointerEvent, PointerKind, PointerSource,
    WindowSize,
};

use crate::App;
use crate::Transform;
use crate::forwarding::{
    cursor_over_hero_panel, forward_text_to_hero, forward_to_hero, forward_wheel_to_hero,
    resolve_live_entry,
};

// `impl App` is split across sibling modules (see the eyedropper /
// keyboard handlers) to keep this file under the HR-18 LOC cap.
mod eyedropper;
mod keyboard;
pub(crate) mod protect_brush;

impl App {
    pub(crate) fn on_close_request(&mut self, event_loop: &ActiveEventLoop) {
        match self.handler.on_close_request() {
            CloseAction::Close => {
                self.handler.on_lifecycle(Lifecycle::WillTerminate);
                event_loop.exit();
            }
            CloseAction::Cancel => {}
        }
    }

    pub(crate) fn on_resized(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.pending_resize = Some(WindowSize::new(size.width, size.height));
    }

    /// M14.4e drag-and-drop. winit emits one HoveredFile per path when
    /// multiple files are dragged together. Buffer paths into
    /// `self.hovered_files` and push to the hero (for the overlay) on
    /// every HoveredFile event.
    pub(crate) fn on_hovered_file(&mut self, path: std::path::PathBuf) {
        self.hovered_files.push(path);
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.dragging_files = Some((self.hovered_files.clone(), self.last_cursor));
        }
        self.handler.on_file_hover(&self.hovered_files);
        if let Some(host) = self.host.as_ref() {
            host.request_redraw();
        }
    }

    pub(crate) fn on_hovered_file_cancelled(&mut self) {
        self.hovered_files.clear();
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.dragging_files = None;
        }
        self.handler.on_file_hover_cancel();
        if let Some(host) = self.host.as_ref() {
            host.request_redraw();
        }
    }

    pub(crate) fn on_dropped_file(&mut self, path: std::path::PathBuf) {
        // M14.7 polish (7.3 fix): winit fires `DroppedFile` once PER
        // FILE on macOS but the events arrive across multiple loop
        // iterations. Importing inline on each event was racy — some
        // imports silently dropped when an event came in mid-render.
        // Buffer the path here; `render_frame` drains `pending_drops`
        // atomically.
        self.pending_drops.push(path);
        if let Some(host) = self.host.as_ref() {
            host.request_redraw();
        }
    }

    pub(crate) fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        if let Some(host) = &self.host {
            host.scale().set(scale_factor as f32);
            if let Some(gfx) = self.gfx.as_ref() {
                self.pending_resize = Some(gfx.surface.size());
            }
        }
    }

    pub(crate) fn on_modifiers_changed(&mut self, mods: winit::event::Modifiers) {
        self.modifiers = mods.state();
        // M14.A: push the Shift state to the hero's WidgetStore so
        // `dispatch_pointer` Move can scale the NumberInput drag delta
        // correctly (Shift = fine adjustment). The ph2d-host
        // `PointerEvent` schema doesn't carry modifiers natively — the
        // store cache is the canonical bridge for now.
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            hero.store.set_shift_held(self.modifiers.shift_key());
        }
    }

    /// IME composition commits — PT-BR / Spanish / French accent
    /// dead-key sequences arrive here on macOS, NOT in `KeyEvent::text`
    /// (the system text-input service swallows the dead-key keystroke
    /// and emits the composed char via `Ime::Commit`).
    pub(crate) fn on_ime_commit(&mut self, text: String) {
        for ch in text.chars() {
            if !ch.is_control() {
                forward_text_to_hero(self.gfx.as_mut(), ch);
            }
        }
        // `Preedit` (in-progress composition) is ignored for now — no
        // visible preedit caret yet. Future: render the preedit text
        // in italics at the caret.
    }

    pub(crate) fn on_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        let prev = self.last_pointer;
        self.last_pointer = (position.x as f32, position.y as f32);
        // M14.4e: cache the latest cursor for DroppedFile — winit's
        // DroppedFile carries no position, so we project the most-
        // recently-seen cursor to world.
        self.last_cursor = self.last_pointer;
        // BgRemoval eyedropper drag (SHELL-only): while the primary
        // button is held with the eyedropper armed, every motion
        // samples another colour. Early-return so the move does not
        // also drive a gizmo drag / panel slider.
        if self.eyedropper_dragging {
            self.try_eyedropper_sample(self.last_pointer.0, self.last_pointer.1);
            return;
        }
        // Keep the brush-size ring gizmo following the cursor while the
        // protection brush is armed (published for the on-canvas overlay).
        self.update_protect_brush_cursor(self.last_pointer.0, self.last_pointer.1);
        // BgRemoval protection brush drag (SHELL-only): while a dab is in
        // progress, every motion paints/erases another disc into the keep
        // mask. Early-return so it doesn't also drive a gizmo drag / slider.
        if self.protect_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // M14.4b.bis: middle-drag camera pan. Applied BEFORE pointer
        // forwarding so widgets receive the move event but the camera
        // also follows.
        if let Some(anchor) = self.pan_anchor
            && let Some(gfx) = self.gfx.as_mut()
        {
            let dx = self.last_pointer.0 - anchor.0;
            let dy = self.last_pointer.1 - anchor.1;
            let size = gfx.surface.size();
            gfx.camera
                .pan_screen_delta(dx, dy, size.width as f32, size.height as f32);
            self.pan_anchor = Some(self.last_pointer);
            let _ = prev; // silence unused warning when feature shifts
        }
        let evt = PointerEvent {
            x: self.last_pointer.0,
            y: self.last_pointer.1,
            pressure: 1.0,
            kind: PointerKind::Move,
            source: PointerSource::Mouse,
            button: ph2d_host::PointerButton::Primary,
            timestamp_ns: Self::timestamp_ns(),
        };
        self.handler.on_pointer(evt);
        forward_to_hero(self.gfx.as_mut(), evt);
        // M14.7 C: advance the gizmo drag if one is open. We update
        // the cursor on the snapshot, derive the new Transform via the
        // pure math in `compute_gizmo_transform`, and write it back to
        // SimWorld. The next frame's extract+paint mirror the change
        // visually.
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
            && let Some(mut drag) = hero.gizmo.drag
        {
            drag.cursor_screen = (self.last_pointer.0, self.last_pointer.1);
            hero.gizmo.drag = Some(drag);
            if matches!(drag.kind, ph2d_editor::GizmoDragKind::MovePivot) {
                // TOOL_PIVOT: relocate the pivot to the cursor while the
                // sprite's quad stays world-fixed (compensating anchor).
                // CTRL snaps to the quad center / corners / edge mids.
                let window_size = gfx.surface.size();
                let entity = ph2d_ecs::Entity::from_bits(drag.entity_bits);
                let raw_world = gfx.camera.screen_to_world(drag.cursor_screen, window_size);
                let ctrl = self.modifiers.control_key() || self.modifiers.super_key();
                let target = if ctrl {
                    let half_world = gfx
                        .sim
                        .world()
                        .get::<ph2d_render::Sprite>(entity)
                        .map(|s| {
                            [
                                s.size[0] * drag.start_transform.scale[0] * 0.5,
                                s.size[1] * drag.start_transform.scale[1] * 0.5,
                            ]
                        })
                        .unwrap_or([0.0, 0.0]);
                    let cands = ph2d_editor::pivot_snap_candidates(
                        drag.pivot_world,
                        drag.start_transform.rotation,
                        half_world,
                    );
                    // Snap when within ~14 px of a candidate, converted
                    // to world units at the current zoom.
                    let thresh = 14.0 * gfx.camera.height_world / window_size.height as f32;
                    let mut best = raw_world;
                    let mut best_d2 = thresh * thresh;
                    for c in cands {
                        let dx = c[0] - raw_world[0];
                        let dy = c[1] - raw_world[1];
                        let d2 = dx * dx + dy * dy;
                        if d2 <= best_d2 {
                            best_d2 = d2;
                            best = c;
                        }
                    }
                    best
                } else {
                    raw_world
                };
                let (new_translation, new_anchor) = ph2d_editor::move_pivot_transform(
                    drag.start_transform,
                    drag.pivot_world,
                    target,
                );
                if let Some(mut t) = gfx.sim.world_mut().get_mut::<Transform>(entity) {
                    t.translation = ph2d_core::Vec2::new(new_translation[0], new_translation[1]);
                }
                if let Some(mut s) = gfx.sim.world_mut().get_mut::<ph2d_render::Sprite>(entity) {
                    s.anchor = new_anchor;
                }
            } else {
                let window_size = gfx.surface.size();
                let cam = ph2d_editor::GizmoCamera {
                    center: gfx.camera.center,
                    height_world: gfx.camera.height_world,
                    window_w: window_size.width as f32,
                    window_h: window_size.height as f32,
                };
                // M14.7 D: sample winit's tracked modifier state (updated
                // on ModifiersChanged). Shift / Ctrl / Alt feed AR lock +
                // snap + mirror-anchor. On macOS we treat Cmd as Ctrl
                // (industry convention for snap-to-grid).
                let mods = ph2d_editor::GizmoModifiers {
                    shift: self.modifiers.shift_key(),
                    ctrl: self.modifiers.control_key() || self.modifiers.super_key(),
                    alt: self.modifiers.alt_key(),
                };
                let snap = ph2d_editor::GizmoSnap {
                    move_meters: hero.project.snap_move_meters,
                    rotate_deg: hero.project.snap_rotate_deg,
                };
                // Grid-snap apply (gizmo sites). The grid_snap subsystem's
                // `snap_world` is the canonical place to align world
                // positions to the active grid; it's a no-op when
                // `state.snap_enabled` is false or the active kind has no
                // snap target (Quadtree / Voronoi).
                let entity = ph2d_ecs::Entity::from_bits(drag.entity_bits);
                let sprite_half_rendered = gfx
                    .sim
                    .world()
                    .get::<ph2d_render::Sprite>(entity)
                    .map(|s| {
                        [
                            s.size[0] * drag.start_transform.scale[0] * 0.5,
                            s.size[1] * drag.start_transform.scale[1] * 0.5,
                        ]
                    })
                    .unwrap_or([0.0, 0.0]);
                let is_scale = matches!(
                    drag.kind,
                    ph2d_editor::GizmoDragKind::ScaleCorner { .. }
                        | ph2d_editor::GizmoDragKind::ScaleEdge { .. }
                );
                let new_t = if is_scale {
                    let snap_state = &mut hero.grid.snap_state;
                    let mut snap_closure = |w: [f32; 2]| -> [f32; 2] {
                        snap_state.snap_world(w, sprite_half_rendered)
                    };
                    ph2d_editor::compute_gizmo_transform(
                        &drag,
                        &cam,
                        mods,
                        snap,
                        Some(&mut snap_closure),
                    )
                } else {
                    ph2d_editor::compute_gizmo_transform(&drag, &cam, mods, snap, None)
                };
                let new_t = if is_scale {
                    new_t
                } else {
                    let mut new_t = new_t;
                    let sprite_half_new = gfx
                        .sim
                        .world()
                        .get::<ph2d_render::Sprite>(entity)
                        .map(|s| {
                            [
                                s.size[0] * new_t.scale[0] * 0.5,
                                s.size[1] * new_t.scale[1] * 0.5,
                            ]
                        })
                        .unwrap_or([0.0, 0.0]);
                    new_t.translation = hero
                        .grid
                        .snap_state
                        .snap_world(new_t.translation, sprite_half_new);
                    new_t
                };
                if let Some(mut t) = gfx.sim.world_mut().get_mut::<Transform>(entity) {
                    t.translation =
                        ph2d_core::Vec2::new(new_t.translation[0], new_t.translation[1]);
                    t.rotation = new_t.rotation;
                    t.scale = ph2d_core::Vec2::new(new_t.scale[0], new_t.scale[1]);
                }
            }
        }
        // Drag-in-progress: forward pointer to active tool panel
        // hit-test → updates slider value continuously.
        if self.dragging.is_some() {
            self.dispatch_panel_pointer(self.last_pointer.0, self.last_pointer.1, false);
        }
    }

    pub(crate) fn on_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let (dx, dy) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (x * 16.0, y * 16.0),
            MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
        };
        // M14.4b.bis: wheel over the canvas zooms the camera. Wheel
        // over a hero panel keeps the existing panel-scroll behavior
        // (forward to hero).
        let over_panel =
            cursor_over_hero_panel(self.gfx.as_ref(), self.last_pointer.0, self.last_pointer.1);
        if !over_panel && let Some(gfx) = self.gfx.as_mut() {
            // Wheel up (positive dy) zooms IN (smaller height_world).
            let factor = 0.9_f32.powf(dy / 16.0);
            gfx.camera.zoom(factor);
        } else {
            let evt = ph2d_host::WheelEvent {
                x: self.last_pointer.0,
                y: self.last_pointer.1,
                delta_x: dx,
                delta_y: dy,
                modifiers: Self::convert_modifiers(self.modifiers),
                timestamp_ns: Self::timestamp_ns(),
            };
            forward_wheel_to_hero(self.gfx.as_mut(), evt);
        }
    }

    pub(crate) fn on_mouse_input(&mut self, state: ElementState, button: MouseButton) {
        let kind = match state {
            ElementState::Pressed => PointerKind::Down,
            ElementState::Released => PointerKind::Up,
        };
        let mapped_button = match button {
            MouseButton::Left => ph2d_host::PointerButton::Primary,
            MouseButton::Right => ph2d_host::PointerButton::Secondary,
            MouseButton::Middle => ph2d_host::PointerButton::Middle,
            _ => ph2d_host::PointerButton::Primary,
        };
        let evt = PointerEvent {
            x: self.last_pointer.0,
            y: self.last_pointer.1,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: mapped_button,
            timestamp_ns: Self::timestamp_ns(),
        };
        self.handler.on_pointer(evt);
        forward_to_hero(self.gfx.as_mut(), evt);

        // BgRemoval eyedropper (SHELL-only). A Secondary Down on an
        // extra-colour swatch deletes it; a Primary Down/drag over the
        // sprite samples colours. Both consume the event so the normal
        // canvas/gizmo/context-menu logic below does not run.
        match (mapped_button, kind) {
            (ph2d_host::PointerButton::Secondary, PointerKind::Down)
                if self.try_eyedropper_delete(evt.x, evt.y) =>
            {
                return;
            }
            // Protection brush ERASE: a Secondary Down with the brush armed
            // erases the first dab + starts an erase drag (continued in
            // CursorMoved). Consumes so it doesn't open a context menu.
            (ph2d_host::PointerButton::Secondary, PointerKind::Down)
                if self.try_protect_erase(evt.x, evt.y) =>
            {
                return;
            }
            (ph2d_host::PointerButton::Secondary, PointerKind::Up) => {
                // End any erase drag (no-op when not erasing).
                self.end_protect_paint();
            }
            (ph2d_host::PointerButton::Primary, PointerKind::Down)
                if self.try_eyedropper_sample(evt.x, evt.y) =>
            {
                self.eyedropper_dragging = true;
                return;
            }
            // Protection brush: a Primary Down with the brush armed paints
            // the first dab + starts the drag (drag continues in
            // CursorMoved). Consumes the event so it doesn't pick/move the
            // sprite.
            (ph2d_host::PointerButton::Primary, PointerKind::Down)
                if self.try_protect_paint(evt.x, evt.y) =>
            {
                return;
            }
            (ph2d_host::PointerButton::Primary, PointerKind::Up) => {
                self.eyedropper_dragging = false;
                self.end_protect_paint();
            }
            _ => {}
        }

        // M14.7 C: gizmo drag begin/end. A Primary Down that lands on
        // a gizmo handle starts a drag (snapshot Transform + cursor
        // world pos); Up clears it. Move handling lives in CursorMoved
        // so every motion event gets the live cursor.
        if mapped_button == ph2d_host::PointerButton::Primary
            && let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            match kind {
                PointerKind::Down => {
                    let hit_id = hero.hit_index.hit(evt.x, evt.y);
                    let gizmo_kind = hit_id.and_then(ph2d_editor::gizmo_kind_for_id);
                    let is_specific_handle = matches!(
                        gizmo_kind,
                        Some(ph2d_editor::GizmoDragKind::ScaleCorner { .. })
                            | Some(ph2d_editor::GizmoDragKind::ScaleEdge { .. })
                            | Some(ph2d_editor::GizmoDragKind::Rotate)
                    );
                    // TOOL_PIVOT begin: when the Pivot transform tool is
                    // the active radio selection and the click lands on
                    // the selected sprite (or its pivot dot), open a
                    // MovePivot drag instead of the pick / scale path.
                    let pivot_tool_active = hero.store.button_state(ph2d_editor::ids::TOOL_PIVOT)
                        == Some(ph2d_editor::widget::ButtonState::Pressed);
                    let mut began_pivot = false;
                    if pivot_tool_active
                        && hero.store.panel_at(evt.x, evt.y).is_none()
                        && hero.store.context_menu().is_none()
                        && let Some(entity_bits) = hero.gizmo.selection
                    {
                        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
                        let window_size = gfx.surface.size();
                        let world_pos = gfx.camera.screen_to_world((evt.x, evt.y), window_size);
                        let on_pivot_dot = hit_id == Some(ph2d_editor::gizmo::ids::GIZMO_PIVOT);
                        let on_sprite =
                            ph2d_render::pick_sprite_at_world(gfx.present.world_mut(), world_pos)
                                == Some(entity_bits);
                        if (on_pivot_dot || on_sprite)
                            && let Some(t) = gfx.sim.world().get::<Transform>(entity)
                        {
                            let snap_t = ph2d_editor::TransformSnapshot {
                                translation: [t.translation.x, t.translation.y],
                                rotation: t.rotation,
                                scale: [t.scale.x, t.scale.y],
                            };
                            let sprite = gfx.sim.world().get::<ph2d_render::Sprite>(entity);
                            let anchor = sprite.map(|s| s.anchor).unwrap_or([0.0, 0.0]);
                            let half = sprite
                                .map(|s| [s.size[0] * 0.5, s.size[1] * 0.5])
                                .unwrap_or([0.0, 0.0]);
                            // Invariant quad center = pivot + R·(anchor ⊙ scale).
                            let ax = anchor[0] * snap_t.scale[0];
                            let ay = anchor[1] * snap_t.scale[1];
                            let (sin_r, cos_r) = snap_t.rotation.sin_cos();
                            let quad_center = [
                                snap_t.translation[0] + ax * cos_r - ay * sin_r,
                                snap_t.translation[1] + ax * sin_r + ay * cos_r,
                            ];
                            hero.gizmo.drag = Some(ph2d_editor::GizmoDragState {
                                kind: ph2d_editor::GizmoDragKind::MovePivot,
                                entity_bits,
                                start_screen: (evt.x, evt.y),
                                cursor_screen: (evt.x, evt.y),
                                start_transform: snap_t,
                                pivot_world: quad_center,
                                start_cursor_world: world_pos,
                                sprite_half_intrinsic: half,
                                anchor_is_center: false,
                            });
                            began_pivot = true;
                        }
                    }
                    if began_pivot {
                        // MovePivot drag opened; Move events drive it.
                    } else if is_specific_handle
                        && let Some(gkind) = gizmo_kind
                        && let Some(entity_bits) = hero.gizmo.selection
                    {
                        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
                        let window_size = gfx.surface.size();
                        let start_world = gfx.camera.screen_to_world((evt.x, evt.y), window_size);
                        if let Some(t) = gfx.sim.world().get::<Transform>(entity) {
                            let snap = ph2d_editor::TransformSnapshot {
                                translation: [t.translation.x, t.translation.y],
                                rotation: t.rotation,
                                scale: [t.scale.x, t.scale.y],
                            };
                            let use_center_anchor =
                                self.modifiers.control_key() || self.modifiers.super_key();
                            let sprite_half_intrinsic = gfx
                                .sim
                                .world()
                                .get::<ph2d_render::Sprite>(entity)
                                .map(|s| [s.size[0] * 0.5, s.size[1] * 0.5])
                                .unwrap_or([0.0, 0.0]);
                            let pivot = ph2d_editor::anchor_pivot_world(
                                gkind,
                                sprite_half_intrinsic,
                                snap,
                                use_center_anchor,
                            );
                            hero.gizmo.drag = Some(ph2d_editor::GizmoDragState {
                                kind: gkind,
                                entity_bits,
                                start_screen: (evt.x, evt.y),
                                cursor_screen: (evt.x, evt.y),
                                start_transform: snap,
                                pivot_world: pivot,
                                start_cursor_world: start_world,
                                sprite_half_intrinsic,
                                anchor_is_center: use_center_anchor,
                            });
                        }
                    } else if hero.store.panel_at(evt.x, evt.y).is_none()
                        && hero.store.context_menu().is_none()
                        && (hit_id.is_none()
                            || matches!(gizmo_kind, Some(ph2d_editor::GizmoDragKind::Translate))
                            || hit_id == Some(ph2d_editor::gizmo::ids::GIZMO_PIVOT))
                    {
                        // Canvas pick (M14.7 A) — see commit history
                        // for the four conditions enumerated.
                        let window_size = gfx.surface.size();
                        let world_pos = gfx.camera.screen_to_world((evt.x, evt.y), window_size);
                        let hits =
                            ph2d_render::pick_sprites_at_world(gfx.present.world_mut(), world_pos);
                        let same_list = !hits.is_empty() && hits == self.cycle_pick_hits;
                        if !same_list {
                            self.cycle_pick_world = Some(world_pos);
                            self.cycle_pick_hits = hits.clone();
                            self.cycle_pick_idx = 0;
                            self.cycle_pick_count = 1;
                        } else {
                            self.cycle_pick_count = self.cycle_pick_count.saturating_add(1);
                            if self.cycle_pick_count.is_multiple_of(2) {
                                // Even count → selection stays.
                            } else if !hits.is_empty() {
                                self.cycle_pick_idx = (self.cycle_pick_idx + 1) % hits.len();
                            }
                        }
                        let picked = if hits.is_empty() {
                            None
                        } else {
                            hits.get(self.cycle_pick_idx).copied()
                        };
                        hero.gizmo.selection = picked;
                        if let Some(bits) = picked {
                            let entity = ph2d_ecs::Entity::from_bits(bits);
                            if let Some(t) = gfx.sim.world().get::<Transform>(entity) {
                                let snap_t = ph2d_editor::TransformSnapshot {
                                    translation: [t.translation.x, t.translation.y],
                                    rotation: t.rotation,
                                    scale: [t.scale.x, t.scale.y],
                                };
                                let pivot = [t.translation.x, t.translation.y];
                                hero.gizmo.drag = Some(ph2d_editor::GizmoDragState {
                                    kind: ph2d_editor::GizmoDragKind::Translate,
                                    entity_bits: bits,
                                    start_screen: (evt.x, evt.y),
                                    cursor_screen: (evt.x, evt.y),
                                    start_transform: snap_t,
                                    pivot_world: pivot,
                                    start_cursor_world: world_pos,
                                    sprite_half_intrinsic: [0.0, 0.0],
                                    anchor_is_center: false,
                                });
                            }
                        }
                        // ADR-0029 Phase C.2: live entries owned by the
                        // Hierarchy panel crate; reach via the public
                        // thread-local snapshot.
                        if let Some(entry) = resolve_live_entry(gfx.hero_live.as_ref(), picked) {
                            hero.selection = Some(ph2d_editor::HeroSelection {
                                label: entry.name.clone(),
                                kind: entry.badge.clone().unwrap_or_else(|| "ENT".to_string()),
                                world_pos: (0.0, 0.0),
                            });
                        } else if picked.is_none() {
                            hero.selection = None;
                        }
                        self.title_dirty = true;
                    }
                }
                PointerKind::Up => {
                    // Drop the drag — Transform is already committed
                    // up to the latest Move position.
                    hero.gizmo.drag = None;
                }
                _ => {}
            }
        }
        // M14.4b.bis: middle button = camera pan anchor. Tracked here
        // so CursorMoved can drive the pan.
        if button == MouseButton::Middle {
            match state {
                ElementState::Pressed => {
                    self.pan_anchor = Some(self.last_pointer);
                }
                ElementState::Released => {
                    self.pan_anchor = None;
                }
            }
        }
        match state {
            ElementState::Pressed => {
                // Mirror-sidebar chip takes precedence over the panel
                // hit-test (different zone, no overlap).
                let mut consumed = false;
                if let Some(gfx) = self.gfx.as_mut()
                    && !gfx.zen.is_active()
                    && let Some(btn) = gfx.layout.mirror_button_rect()
                    && btn.contains(self.last_pointer.0, self.last_pointer.1)
                {
                    gfx.layout.mirror_sidebar();
                    gfx.toasts.push(Toast::info(format!(
                        "Sidebar · {:?}",
                        gfx.layout.sidebar_side
                    )));
                    self.title_dirty = true;
                    consumed = true;
                }
                // Tool palette icon click — switch active tool.
                //
                // CRITICAL: only hit-test the palette where it is actually
                // PAINTED — the legacy no-hero (demo) path. In the editor
                // (`hero_screen` is `Some`) the palette is NOT painted (the
                // editor switches tools via the LeftRail + Image Tools
                // pills), yet this hit-test used to run unconditionally.
                // Zone::TopRight is the right HALF of the toolbar strip —
                // exactly where the TopBar paints its right clusters incl.
                // the Settings gear — so a click on "Config" also landed on
                // an INVISIBLE palette slot and silently switched tools
                // ("Tool · Move"/"Tool · Padding"). Gating on
                // `hero_screen.is_none()` (the paint condition) makes the
                // top-right belong solely to the TopBar in the editor.
                //
                // The visible-tools filter below still applies in the demo
                // path so its indices match the paint mapping (no drift).
                if !consumed
                    && let Some(gfx) = self.gfx.as_mut()
                    && !gfx.zen.is_active()
                    && gfx.hero_screen.is_none()
                {
                    let mode_on = gfx
                        .hero_screen
                        .as_ref()
                        .map(|h| h.image_edit.mode_on)
                        .unwrap_or(false);
                    let visible = crate::palette_visible_tool_indices(&gfx.tools, mode_on);
                    let palette = gfx.layout.tool_palette_rects(visible.len());
                    let hit_idx = palette
                        .iter()
                        .position(|r| r.contains(self.last_pointer.0, self.last_pointer.1));
                    if let Some(slot) = hit_idx {
                        let tool_idx = visible[slot];
                        let tool_id = gfx.tools.tools()[tool_idx].id();
                        let tool_label = gfx.tools.tools()[tool_idx].label().to_string();
                        if gfx.tools.set_active(&tool_id) {
                            gfx.toasts.push(Toast::info(format!("Tool · {tool_label}")));
                            self.title_dirty = true;
                        }
                        consumed = true;
                    }
                }
                if !consumed {
                    // Mouse down — start hit-test against active panel.
                    self.dispatch_panel_pointer(self.last_pointer.0, self.last_pointer.1, true);
                }
            }
            ElementState::Released => {
                // End any drag-in-progress.
                self.dragging = None;
            }
        }
    }
}
