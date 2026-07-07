// ph2d-loc-cap: Onda 2C multi-select dispatch + hit_map routing +
// click-vs-drag + group-translate snapshot capture grew this file
// past the HR-18 600-LOC cap (currently ~900 LOC). The MouseInput
// Down/Up arms are the bulk; the natural decomposition is to move
// each Down sub-path (modifier override / pivot tool / gizmo handle /
// canvas pick) and the Up resolver into siblings under
// `input_dispatch/`, parallel to the existing eyedropper / gizmo_drag
// / keyboard / protect_brush splits. That refactor lands as a
// follow-up to Onda 2 once the gizmo polish is locked.
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
pub(crate) mod fill_drag;
mod gizmo_drag;
mod keyboard;
pub(crate) mod painter_canvas_input;
pub(crate) mod painter_falloff_input;
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
        // Fase 0c: also push the Cmd (macOS super) / Ctrl modifier
        // OR'd together — used by hierarchy + canvas multi-select to
        // map a click into `SelectModifier::Toggle`.
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            hero.store.set_shift_held(self.modifiers.shift_key());
            hero.store
                .set_cmd_held(self.modifiers.super_key() || self.modifiers.control_key());
            // Motion Nodes M0.T3 — Alt cache, folded into `GestureMods.alt` for
            // graph gestures (mirror of shift/cmd; pointer events carry no mods).
            hero.store.set_alt_held(self.modifiers.alt_key());
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

    /// Reflect the colour-picker eyedropper state in the OS cursor — a crosshair "target" while a
    /// pick is armed, the default arrow otherwise. Called each CursorMoved (winit dedups the icon).
    fn update_eyedropper_cursor(&self) {
        let Some(win) = self.window.as_ref() else {
            return;
        };
        let armed = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.store.eyedropper_pending().is_some());
        win.set_cursor(if armed {
            winit::window::CursorIcon::Crosshair
        } else {
            winit::window::CursorIcon::Default
        });
    }

    /// ADR-0108 Fase 1: booleana das DUAS últimas regiões fechadas da cena nova
    /// (destrutivo — consome os operandos, insere o resultado). Modo de teste.
    fn vec_boolean(&mut self, op: ph2d_vec_boolean::BoolOp) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let closed: Vec<u64> = gfx
            .vec_scene
            .paths()
            .iter()
            .filter(|p| p.closed)
            .map(|p| p.id)
            .collect();
        if closed.len() < 2 {
            eprintln!("[ph2d-vec] boolean: precisa de 2 regiões FECHADAS");
            return;
        }
        let (ida, idb) = (closed[closed.len() - 2], closed[closed.len() - 1]);
        let a = gfx.vec_scene.paths().iter().find(|p| p.id == ida).cloned();
        let b = gfx.vec_scene.paths().iter().find(|p| p.id == idb).cloned();
        if let (Some(a), Some(b)) = (a, b) {
            let results = ph2d_vec_boolean::apply(&a, &b, op);
            if results.is_empty() {
                eprintln!("[ph2d-vec] boolean {op:?}: resultado vazio");
                return;
            }
            let pre = gfx.vec_scene.clone(); // Fase 2: undo da booleana
            gfx.vec_scene.remove_path(ida);
            gfx.vec_scene.remove_path(idb);
            let mut last = None;
            for r in results {
                last = Some(gfx.vec_scene.push_path(r));
            }
            self.vec_history.push_undo(pre);
            self.vec_pen.select(last);
            eprintln!("[ph2d-vec] boolean {op:?}: ok");
        }
    }

    /// ADR-0108 Fase 1: apaga o path selecionado (Delete/Backspace no modo vetorial).
    fn vec_delete_selected(&mut self) -> bool {
        let Some(sel) = self.vec_pen.selected() else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let pre = gfx.vec_scene.clone();
        if gfx.vec_scene.remove_path(sel) {
            self.vec_history.push_undo(pre);
            self.vec_pen.clear();
            eprintln!("[ph2d-vec] path {sel} apagado");
            true
        } else {
            false
        }
    }

    /// ADR-0108 Fase 2: desfaz o último passo vetorial (Ctrl+Z).
    fn vec_undo(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        if let Some(prev) = self.vec_history.undo(&gfx.vec_scene) {
            gfx.vec_scene = prev;
            self.vec_pen.finish(); // limpa estado de desenho/arrasto pendente
            eprintln!("[ph2d-vec] undo");
        }
    }

    /// ADR-0108 Fase 2: refaz (Ctrl+Shift+Z / Ctrl+Y).
    fn vec_redo(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        if let Some(next) = self.vec_history.redo(&gfx.vec_scene) {
            gfx.vec_scene = next;
            self.vec_pen.finish();
            eprintln!("[ph2d-vec] redo");
        }
    }

    /// ADR-0108 Fase 2: salva a cena vetorial em `PH2D_VEC_SAVE_PATH` (default
    /// `ph2d_vec_scene.postcard` no CWD). Ctrl+S no modo vetorial.
    fn vec_save(&mut self) {
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        let path = Self::vec_save_path();
        match gfx.vec_scene.to_bytes() {
            Ok(bytes) => match std::fs::write(&path, &bytes) {
                Ok(()) => eprintln!("[ph2d-vec] salvo: {path} ({} bytes)", bytes.len()),
                Err(e) => eprintln!("[ph2d-vec] erro ao salvar {path}: {e}"),
            },
            Err(e) => eprintln!("[ph2d-vec] erro ao serializar: {e}"),
        }
    }

    /// ADR-0108 Fase 2: carrega a cena de `PH2D_VEC_SAVE_PATH` (Ctrl+O). O load é
    /// undoável (snapshot pré).
    fn vec_load(&mut self) {
        let path = Self::vec_save_path();
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[ph2d-vec] sem arquivo {path}: {e}");
                return;
            }
        };
        match ph2d_vec_scene::VecScene::from_bytes(&bytes) {
            Ok(scene) => {
                let Some(gfx) = self.gfx.as_mut() else {
                    return;
                };
                self.vec_history.push_undo(gfx.vec_scene.clone());
                gfx.vec_scene = scene;
                self.vec_pen.clear();
                eprintln!("[ph2d-vec] carregado: {path}");
            }
            Err(e) => eprintln!("[ph2d-vec] erro ao carregar {path}: {e}"),
        }
    }

    fn vec_save_path() -> String {
        std::env::var("PH2D_VEC_SAVE_PATH")
            .unwrap_or_else(|_| "ph2d_vec_scene.postcard".to_string())
    }

    /// ADR-0108 cutover: is the Vector drawing tool the active tool? Gates the
    /// Pen input hooks (replaces the retired `PH2D_VEC_PEN` test flag).
    pub(crate) fn vector_tool_active(&self) -> bool {
        self.gfx.as_ref().is_some_and(|g| {
            g.tools
                .active()
                .is_some_and(|t| t.id() == ph2d_editor::ToolId::new("vector"))
        })
    }

    /// ADR-0108: enquanto o Pen arrasta um handle, projeta o cursor pra world e
    /// puxa os handles Bézier do último vértice. No-op barato quando não há
    /// arrasto — chamado a cada CursorMoved.
    fn vec_pen_drag_move(&mut self, x: f32, y: f32) -> bool {
        if !self.vector_tool_active() || !self.vec_pen.is_dragging() {
            return false;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let w = gfx.camera.screen_to_world((x, y), win);
        self.vec_pen
            .on_drag(&mut gfx.vec_scene, [w[0] as f64, w[1] as f64])
    }

    pub(crate) fn on_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        // Diagnostics: count every raw winit move (input rate), paired with `paint_stamps_this_frame`
        // in the HUD so the coalescing is visible (high events → 1 stamp).
        self.input_events_this_frame = self.input_events_this_frame.saturating_add(1);
        let prev = self.last_pointer;
        self.last_pointer = (position.x as f32, position.y as f32);
        // M14.4e: cache the latest cursor for DroppedFile — winit's
        // DroppedFile carries no position, so we project the most-
        // recently-seen cursor to world.
        self.last_cursor = self.last_pointer;
        // Reflect the colour-picker eyedropper in the OS cursor (a crosshair "target" while armed).
        self.update_eyedropper_cursor();
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
        // Painter Falloff add-drag (SHELL-only): while a freshly click-added
        // control point is grabbed, motion drags it. Early-return so it doesn't
        // pan / drive a gizmo. No-ops unless an add-drag is live.
        if self.painter_falloff_drag(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Painter brush stroke (SHELL-only): while a canvas stroke is open, every
        // motion feeds another `CanvasPointer` to the active PainterTool. Early-
        // return so it doesn't also drive a gizmo drag / pan / slider.
        if self.painter_canvas_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Fill (Bucket) ColorDrop drag (SHELL-only): while a colour is being dragged from the Fill rail
        // button onto the canvas, deliver it to the painter's Fill. Early-return so it doesn't pan.
        if self.fill_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // Fill "Fill adjust" modal title-band drag (SHELL-only): while the card is grabbed, motion moves
        // it. Early-return so it doesn't pan / drive a gizmo. No-ops unless a modal drag is armed.
        if self.fill_modal_drag_move(self.last_pointer.0, self.last_pointer.1) {
            return;
        }
        // ADR-0108 Fase 1.2: Pen NOVO — arrastar após a âncora puxa os handles
        // Bézier (simétricos). Early-return: não pan/gizmo. No-op sem drag ativo.
        if self.vec_pen_drag_move(self.last_pointer.0, self.last_pointer.1) {
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
        // Fase 0f: extend the active rubber-band rect, if any.
        if let Some(rb) = self.rubber_band.as_mut() {
            rb.current_screen = self.last_pointer;
        }
        let evt = PointerEvent {
            x: self.last_pointer.0,
            y: self.last_pointer.1,
            pressure: 1.0,
            kind: PointerKind::Move,
            source: PointerSource::Mouse,
            // Motion Nodes M0.T1: carry the REAL held button (winit's Move has
            // none). A middle/right drag now reaches editor-core with its
            // identity intact — the graph channel needs it (pan/box-select).
            button: self
                .held_button
                .unwrap_or(ph2d_host::PointerButton::Primary),
            timestamp_ns: Self::timestamp_ns(),
        };
        self.handler.on_pointer(evt);
        // A reparent only fires on pointer-Up (handled in on_mouse_input);
        // Move never emits one.
        let _ = forward_to_hero(self.gfx.as_mut(), evt);
        // M14.7 C: advance an open gizmo drag against the latest cursor
        // (MovePivot / scale / rotate / translate). Extracted to the
        // `gizmo_drag` sibling to keep this dispatch hub readable.
        self.advance_gizmo_drag();
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
        // Motion Nodes M0.T1: track the held button so `CursorMoved` can carry
        // its identity (winit Move events don't). Held between Down and Up.
        self.held_button = match kind {
            PointerKind::Down => Some(mapped_button),
            PointerKind::Up => None,
            PointerKind::Move => self.held_button,
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
        // Was a right-click context menu (or the Fill "Fill adjust" modal) open when this click
        // arrived? If so the click belongs to that overlay (its slider/buttons/items) — chrome dispatch
        // in `forward_to_hero` handles it, so the canvas-consume arms below (paint / gizmo / select /
        // pan) must NOT also fire on a click LANDING on the overlay (which sits over the canvas). The
        // Fill modal counts as a modal exactly like the new-image dialog — without this, clicking its
        // threshold slider started a fresh flood-fill on the canvas underneath (mirror of the
        // new-image-modal "leaked a dab" fix). Captured now because `forward_to_hero` may close it.
        let menu_open_before = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| {
                h.store.context_menu().is_some() || h.store.fill_modal_pos().is_some()
            });
        // Colour-picker eyedropper armed when this click arrived? `forward_to_hero` services the pick
        // (sampling the pixel) AND clears the pending flag, so by the time the consume arms below run
        // it reads as disarmed. Capture it now so the Painter brush does NOT also paint where the user
        // sampled — the eyedropper must inhibit the brush (the sampled click is consumed, not painted).
        let eyedropper_armed_before = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.store.eyedropper_pending().is_some());

        // ADR-0108 cutover: the Vector tool's Pen draws ONLY on empty canvas.
        // A press over ANY UI — a docked panel body, a topbar pill, an open
        // menu, or this tool's own Style panel controls — MUST fall through to
        // the chrome dispatch below, never the pen; otherwise the whole UI is
        // unclickable while drawing (can't even deactivate the tool). Guard
        // mirrors the sprite-pick path: no panel under the cursor AND no
        // interactive widget hit (`hit_index` covers pills / menus / panel
        // controls; `panel_at` covers panel bodies incl. the vector panel).
        let on_canvas = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map(|h| {
                h.store.panel_at(evt.x, evt.y).is_none() && h.hit_index.hit(evt.x, evt.y).is_none()
            })
            .unwrap_or(false);
        if self.vector_tool_active() && !menu_open_before {
            match (mapped_button, kind) {
                (ph2d_host::PointerButton::Primary, PointerKind::Down) if on_canvas => {
                    if let Some(gfx) = self.gfx.as_mut() {
                        let win = gfx.surface.size();
                        let w = gfx.camera.screen_to_world(self.last_pointer, win);
                        // world-units por pixel (delta de 1px) → limiar/traço em px.
                        let w0 = gfx.camera.screen_to_world((0.0, 0.0), win);
                        let w1 = gfx.camera.screen_to_world((1.0, 0.0), win);
                        let px_to_world =
                            (((w1[0] - w0[0]).powi(2) + (w1[1] - w0[1]).powi(2)).sqrt()) as f64;
                        // Fase 2: snapshot pré-interação (vira passo de undo no Up
                        // só se a cena mudar de fato).
                        self.vec_history.begin(&gfx.vec_scene);
                        self.vec_pen.on_press(
                            &mut gfx.vec_scene,
                            [w[0] as f64, w[1] as f64],
                            px_to_world,
                        );
                        return;
                    }
                }
                (ph2d_host::PointerButton::Primary, PointerKind::Up) => {
                    let consumed = self.vec_pen.on_release();
                    if let Some(gfx) = self.gfx.as_mut() {
                        self.vec_history.commit_if_changed(&gfx.vec_scene);
                    }
                    if consumed {
                        return;
                    }
                }
                (ph2d_host::PointerButton::Secondary, PointerKind::Down) if on_canvas => {
                    self.vec_pen.finish();
                    return;
                }
                _ => {}
            }
        }

        // Painter layers drag-reparent (W3 T3.8): the dispatch emits a
        // PainterLayerReparent on Up of an active layer-row drag; route it to
        // the active PainterTool, which reverses NodeId→LayerId and applies
        // move_into_group / reorder. The concrete-tool downcast lives in the
        // allowlisted painter bridge so central dispatch stays downcast-free
        // (architecture_no_downcast_to_concrete_tool_in_shell gate).
        if let Some((dragged, drop)) = forward_to_hero(self.gfx.as_mut(), evt)
            && let Some(gfx) = self.gfx.as_mut()
        {
            crate::render_loop::painter_bridge_queries::apply_layer_reparent(
                &mut gfx.tools,
                dragged,
                drop,
            );
        }

        // BgRemoval eyedropper (SHELL-only). A Secondary Down on an
        // extra-colour swatch deletes it; a Primary Down/drag over the
        // sprite samples colours. Both consume the event so the normal
        // canvas/gizmo/context-menu logic below does not run.
        // Fill (Bucket) ColorDrop: a Primary Down on the Fill rail button arms the drag-to-canvas gesture
        // + activates Fill. Self-gates on the hit id; the normal Up-click still selects the tool when the
        // press is released ON the button, and is suppressed when it drags off (release outside the rect).
        if matches!(mapped_button, ph2d_host::PointerButton::Primary)
            && matches!(kind, PointerKind::Down)
        {
            // A Down on the C&F button arms the ColorDrop drag AND consumes the event — otherwise it fell
            // through to `painter_canvas_down` below and the active shape tool dropped a stray point on the
            // canvas behind the button (Enio 2026-07-03). The rail button's own press/click already ran in
            // `forward_to_hero` above; the picker opens on release, Fill activates only if the drag reaches
            // the canvas.
            if self.arm_fill_drag_if_on_button(evt.x, evt.y) {
                return;
            }
            // A Primary Down on the Fill modal's title band starts a modal-move (the card follows the
            // cursor via CursorMoved) — consume it so it doesn't click through / start anything else.
            if self.arm_fill_modal_drag_if_on_handle(evt.x, evt.y) {
                return;
            }
        }
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
            // Painter Falloff curve: right-click a control point → open the
            // handle-type menu (Vector / Auto). No-op off a point.
            (ph2d_host::PointerButton::Secondary, PointerKind::Down)
                if self.painter_falloff_open_point_menu(evt.x, evt.y) =>
            {
                return;
            }
            // On-canvas Curve / Free Hand: right-click a control point → open the
            // handle-kind menu (Free / Aligned / Vector / Auto). No-op off a point.
            (ph2d_host::PointerButton::Secondary, PointerKind::Down)
                if self.painter_curve_open_point_menu(evt.x, evt.y) =>
            {
                return;
            }
            // On-canvas Line polyline: right-click ENDS point-creation (Blender/CAD
            // convention). No-op when no Line is being drawn (falls through to the
            // context menu).
            (ph2d_host::PointerButton::Secondary, PointerKind::Down)
                if self.painter_line_finish_points() =>
            {
                return;
            }
            (ph2d_host::PointerButton::Secondary, PointerKind::Up) => {
                // End any erase drag (no-op when not erasing).
                self.end_protect_paint();
            }
            (ph2d_host::PointerButton::Primary, PointerKind::Down)
                if !menu_open_before && self.try_eyedropper_sample(evt.x, evt.y) =>
            {
                self.eyedropper_dragging = true;
                return;
            }
            // Painter Falloff curve: left-click the empty graph (Custom preset) →
            // add a control point where clicked. A press on a handle falls through
            // (the panel's drag dispatch grabs it); a click on an open context menu
            // is the menu's, not a canvas-add (`menu_open_before`).
            (ph2d_host::PointerButton::Primary, PointerKind::Down)
                if !menu_open_before && self.painter_falloff_canvas_add(evt.x, evt.y) =>
            {
                return;
            }
            // Protection brush: a Primary Down with the brush armed paints
            // the first dab + starts the drag (drag continues in
            // CursorMoved). Consumes the event so it doesn't pick/move the
            // sprite.
            (ph2d_host::PointerButton::Primary, PointerKind::Down)
                if !menu_open_before && self.try_protect_paint(evt.x, evt.y) =>
            {
                return;
            }
            // "Add area" automatic selector: a Primary Down with the
            // selector armed runs a single-click flood-fill from the
            // clicked source pixel into the force-remove mask
            // (Enio 2026-05-26). Mirror of the eyedropper sample dispatch.
            (ph2d_host::PointerButton::Primary, PointerKind::Down)
                if !menu_open_before && self.try_add_area_click(evt.x, evt.y) =>
            {
                return;
            }
            // Colour-picker eyedropper sample: when the picker eyedropper was armed, `forward_to_hero`
            // already sampled the pixel — consume the click so the Painter brush does NOT paint there
            // and the sprite isn't picked/moved. Must precede the painter brush arm below.
            (ph2d_host::PointerButton::Primary, PointerKind::Down) if eyedropper_armed_before => {
                return;
            }
            // Painter brush: a Primary Down with the Painter active + a sprite
            // selected, inside the footprint, starts a stroke (the first dab) and
            // arms the drag (continues in CursorMoved). Consumes the event so it
            // doesn't pick / move the sprite. A click on an open modal / context
            // menu is the menu's (`menu_open_before`) — never a stroke on the
            // canvas below it (Enio 2026-06-24: new-image modal leaked a dab).
            (ph2d_host::PointerButton::Primary, PointerKind::Down)
                if !menu_open_before && self.painter_canvas_down(evt.x, evt.y, evt.pressure) =>
            {
                return;
            }
            (ph2d_host::PointerButton::Primary, PointerKind::Up) => {
                self.eyedropper_dragging = false;
                self.end_protect_paint();
                // End a Falloff add-drag (no-op when not dragging).
                self.painter_falloff_release();
                // Close an open painter brush stroke (no-op when not painting).
                self.painter_canvas_up();
                // Finish a Fill ColorDrop drag (fill on the canvas, or open the picker for a plain click
                // on the Fill button). No-op when no fill drag is armed.
                self.fill_drag_up();
                // End a Fill "Fill adjust" modal title-band drag. No-op when not dragging the modal.
                self.fill_modal_drag_up();
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
                    // Onda 1 hotfix: Shift/Cmd in the canvas ALWAYS means
                    // selection-adjustment. Pre-empt the gizmo-handle /
                    // pivot-tool / canvas-pick cascade so a modifier
                    // click never accidentally opens a scale-handle drag
                    // (gizmo handles overlap the sprite bbox corners —
                    // bare Shift+click was landing on a handle and
                    // entering the `is_specific_handle` branch which
                    // bypasses the canvas pick where toggle lives).
                    let shift_held_early = self.modifiers.shift_key();
                    let cmd_held_early = self.modifiers.super_key() || self.modifiers.control_key();
                    if (shift_held_early || cmd_held_early)
                        && hero.store.panel_at(evt.x, evt.y).is_none()
                        && !menu_open_before
                    {
                        let window_size = gfx.surface.size();
                        let world_pos = gfx.camera.screen_to_world((evt.x, evt.y), window_size);
                        let hits =
                            ph2d_render::pick_sprites_at_world(gfx.present.world_mut(), world_pos);
                        if let Some(bits) = hits.first().copied() {
                            hero.gizmo.toggle_in_selection(bits);
                            let primary = hero.gizmo.selection;
                            if let Some(entry) = resolve_live_entry(gfx.hero_live.as_ref(), primary)
                            {
                                hero.selection = Some(ph2d_editor::HeroSelection {
                                    label: entry.name.clone(),
                                    kind: entry.badge.clone().unwrap_or_else(|| "ENT".to_string()),
                                    world_pos: (0.0, 0.0),
                                });
                            } else if primary.is_none() {
                                hero.selection = None;
                            }
                            self.title_dirty = true;
                            return;
                        }
                        // Modifier on empty canvas → fall through to
                        // existing cascade so a Shift-drag can still
                        // open an additive rubber-band.
                    }
                    let hit_id = hero.hit_index.hit(evt.x, evt.y);
                    let gizmo_kind = hit_id.and_then(ph2d_editor::gizmo_kind_for_id);
                    // Onda 2C: hit_map fills in for handles whose ids
                    // aren't canonical — extras + global. The primary
                    // keeps canonical IDs (matches the legacy
                    // `gizmo_kind_for_id` lookup above so the primary
                    // path runs unchanged when it's the only sprite
                    // selected).
                    let hit_map_entry: Option<ph2d_editor::GizmoHit> =
                        hit_id.and_then(|id| hero.gizmo.gizmo_hit_map.get(&id).copied());
                    let effective_target = hit_map_entry
                        .map(|h| h.target)
                        .unwrap_or(ph2d_editor::GizmoTarget::PrimaryIndividual);
                    let effective_kind = hit_map_entry.map(|h| h.kind).or(gizmo_kind);
                    let is_specific_handle = matches!(
                        effective_kind,
                        Some(ph2d_editor::GizmoDragKind::ScaleCorner { .. })
                            | Some(ph2d_editor::GizmoDragKind::ScaleEdge { .. })
                            | Some(ph2d_editor::GizmoDragKind::Rotate)
                    );
                    // Also recognize Translate from a keyed bbox-interior
                    // hit — clicking the interior of an extra or the global
                    // gizmo should open a group translate via the
                    // `effective_target` route (the canvas-pick path below
                    // skips keyed ids since they aren't None / Translate /
                    // PIVOT canonical, so without this guard those clicks
                    // would fall through to nothing).
                    // Keyed Translate = click on the bbox interior of an
                    // extra or the global gizmo (whose interior IDs are
                    // hashed, so `gizmo_kind_for_id` doesn't recognise
                    // them). Treated as a multi-select translate
                    // through the canvas-pick branch below — that
                    // branch resolves the world position to a sprite
                    // via `pick_sprites_at_world` and opens a group
                    // translate drag.
                    let is_keyed_translate = hit_map_entry
                        .map(|h| matches!(h.kind, ph2d_editor::GizmoDragKind::Translate))
                        .unwrap_or(false);
                    // TOOL_PIVOT begin: when the Pivot transform tool is
                    // the active radio selection and the click lands on
                    // the selected sprite (or its pivot dot), open a
                    // MovePivot drag instead of the pick / scale path.
                    let pivot_tool_active = hero.store.button_state(ph2d_editor::ids::TOOL_PIVOT)
                        == Some(ph2d_editor::widget::ButtonState::Pressed);
                    let mut began_pivot = false;
                    if pivot_tool_active
                        && hero.store.panel_at(evt.x, evt.y).is_none()
                        && !menu_open_before
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
                            && !ph2d_ecs::is_locked_for_edit(gfx.sim.world(), entity)
                            && let Some(t) = gfx.sim.world().get::<Transform>(entity)
                        {
                            let snap_t = ph2d_editor::TransformSnapshot {
                                translation: [t.translation.x, t.translation.y],
                                rotation: t.rotation,
                                scale: [t.scale.x, t.scale.y],
                            };
                            let pw = ph2d_ecs::parent_world_transform(gfx.sim.world(), entity);
                            let parent_world = ph2d_editor::TransformSnapshot {
                                translation: [pw.translation.x, pw.translation.y],
                                rotation: pw.rotation,
                                scale: [pw.scale.x, pw.scale.y],
                            };
                            let sprite = gfx.sim.world().get::<ph2d_render::Sprite>(entity);
                            let anchor = sprite.map(|s| s.anchor).unwrap_or([0.0, 0.0]);
                            let half = sprite
                                .map(|s| [s.size[0] * 0.5, s.size[1] * 0.5])
                                .unwrap_or([0.0, 0.0]);
                            // Invariant quad center = pivot + R·(anchor ⊙ scale).
                            let ax = anchor[0] * snap_t.scale[0];
                            let ay = anchor[1] * snap_t.scale[1];
                            // T1.3.5 cross-OS bit-identical.
                            let (sin_r, cos_r) = libm::sincosf(snap_t.rotation);
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
                                target: ph2d_editor::GizmoTarget::PrimaryIndividual,
                                parent_world,
                            });
                            began_pivot = true;
                        }
                    }
                    if began_pivot {
                        // MovePivot drag opened; Move events drive it.
                    } else if is_specific_handle
                        && let Some(gkind) = effective_kind
                        && let Some(entity_bits) = match effective_target {
                            ph2d_editor::GizmoTarget::ExtraIndividual(bits) => Some(bits),
                            _ => hero.gizmo.selection,
                        }
                    {
                        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
                        // 2026-05-26 — bloqueia drag se entidade tem
                        // `Locked` OU ancestral tem `GroupedChildren`.
                        if ph2d_ecs::is_locked_for_edit(gfx.sim.world(), entity) {
                            return;
                        }
                        let window_size = gfx.surface.size();
                        let start_world = gfx.camera.screen_to_world((evt.x, evt.y), window_size);
                        if let Some(t) = gfx.sim.world().get::<Transform>(entity) {
                            let snap = ph2d_editor::TransformSnapshot {
                                translation: [t.translation.x, t.translation.y],
                                rotation: t.rotation,
                                scale: [t.scale.x, t.scale.y],
                            };
                            // Enio 2026-05-26 fix: capture parent's world
                            // transform so compute_gizmo_transform can
                            // unrotate/unscale the delta before writing
                            // back to the entity's LOCAL Transform.
                            let pw = ph2d_ecs::parent_world_transform(gfx.sim.world(), entity);
                            let parent_world = ph2d_editor::TransformSnapshot {
                                translation: [pw.translation.x, pw.translation.y],
                                rotation: pw.rotation,
                                scale: [pw.scale.x, pw.scale.y],
                            };
                            let use_center_anchor =
                                self.modifiers.control_key() || self.modifiers.super_key();
                            let sprite_half_intrinsic = gfx
                                .sim
                                .world()
                                .get::<ph2d_render::Sprite>(entity)
                                .map(|s| [s.size[0] * 0.5, s.size[1] * 0.5])
                                .unwrap_or([0.0, 0.0]);
                            // Onda 2C: pivot world depends on target.
                            // PrimaryIndividual / ExtraIndividual use the
                            // sprite's own anchor (transforms local to it).
                            // Global overrides pivot to the global bbox
                            // center so group transforms rotate/scale every
                            // sprite around a single shared point.
                            let pivot = if let ph2d_editor::GizmoTarget::Global = effective_target
                                && let Some(gv) = hero.gizmo.global_view.as_ref()
                            {
                                [
                                    (gv.bbox_min_world[0] + gv.bbox_max_world[0]) * 0.5,
                                    (gv.bbox_min_world[1] + gv.bbox_max_world[1]) * 0.5,
                                ]
                            } else {
                                // Composição parent×local pra que o
                                // pivot world seja correto mesmo com pai
                                // rotacionado/escalonado (Enio 2026-05-26
                                // fix: child de pai rotacionado tinha
                                // pivot calculado como root).
                                let world_snap = ph2d_editor::compose_snapshot(parent_world, snap);
                                ph2d_editor::anchor_pivot_world(
                                    gkind,
                                    sprite_half_intrinsic,
                                    world_snap,
                                    use_center_anchor,
                                )
                            };
                            // Onda 2 polish: capture the global view at
                            // drag start so snapshots::publish can keep
                            // the global gizmo's visual orientation /
                            // scale in lockstep with the live group
                            // transform (otherwise it would be the
                            // axis-aligned union of rotated sprites,
                            // which grows during rotation instead of
                            // rotating).
                            if matches!(effective_target, ph2d_editor::GizmoTarget::Global) {
                                hero.gizmo.global_view_start = hero.gizmo.global_view;
                            } else {
                                hero.gizmo.global_view_start = None;
                            }
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
                                target: effective_target,
                                parent_world,
                            });
                            // Onda 1 + 2C.4: snapshot every OTHER selected
                            // sprite's full start_transform so
                            // advance_gizmo_drag can apply translate /
                            // local-scale / local-rotate / global-scale /
                            // global-rotate to the whole group. Captured
                            // for ANY drag kind that touches multi-select
                            // (Translate / Scale / Rotate) so the math
                            // branches can fire uniformly later.
                            self.group_drag_starts.clear();
                            if hero.gizmo.selected_len() > 1 {
                                for sel in hero.gizmo.iter_selected() {
                                    if sel == entity_bits {
                                        continue;
                                    }
                                    let e = ph2d_ecs::Entity::from_bits(sel);
                                    if let Some(t) = gfx.sim.world().get::<Transform>(e) {
                                        let epw =
                                            ph2d_ecs::parent_world_transform(gfx.sim.world(), e);
                                        self.group_drag_starts.push(
                                            crate::app_state::GroupDragSnapshot {
                                                entity_bits: sel,
                                                start_transform: ph2d_editor::TransformSnapshot {
                                                    translation: [t.translation.x, t.translation.y],
                                                    rotation: t.rotation,
                                                    scale: [t.scale.x, t.scale.y],
                                                },
                                                parent_world: ph2d_editor::TransformSnapshot {
                                                    translation: [
                                                        epw.translation.x,
                                                        epw.translation.y,
                                                    ],
                                                    rotation: epw.rotation,
                                                    scale: [epw.scale.x, epw.scale.y],
                                                },
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    } else if hero.store.panel_at(evt.x, evt.y).is_none()
                        && !menu_open_before
                        && (hit_id.is_none()
                            || matches!(gizmo_kind, Some(ph2d_editor::GizmoDragKind::Translate))
                            || hit_id == Some(ph2d_editor::gizmo::ids::GIZMO_PIVOT)
                            || is_keyed_translate)
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
                            // No sprite under the cursor. (The old vector-scene
                            // object pick fell back here; retired with ADR-0108.)
                            None
                        } else {
                            hits.get(self.cycle_pick_idx).copied()
                        };
                        // Fase 0d: read modifier state at click time —
                        // Shift adds to the selection, Cmd/Ctrl toggles,
                        // bare click replaces (legacy default). Modifier
                        // clicks skip drag setup since the user is
                        // adjusting selection, not moving sprites.
                        let shift_held = self.modifiers.shift_key();
                        let cmd_held = self.modifiers.super_key() || self.modifiers.control_key();
                        // Smart-click preservation: bare click on a
                        // sprite that's already inside an active multi-
                        // selection KEEPS the whole set (user intends
                        // to interact with the group — e.g. drag the
                        // group or run a tool — not collapse to single).
                        let preserves_multi = picked.is_some_and(|bits| {
                            hero.gizmo.selected_len() > 1 && hero.gizmo.is_selected(bits)
                        });
                        // Drag-setup skip: modifier clicks adjust the
                        // selection but should not start a gizmo drag
                        // (the user is curating, not moving). Bare-
                        // click in a multi-selection DOES start a drag
                        // (group translate via the clicked sprite as
                        // pivot, Onda 1).
                        let is_modifier_click = picked.is_some() && (shift_held || cmd_held);
                        if let Some(bits) = picked {
                            if cmd_held || shift_held {
                                // Onda 1: unify Shift + Cmd as toggle on
                                // the canvas. Click on a sprite already
                                // in the selection → removes JUST that
                                // one. Click on a sprite outside → adds.
                                // The Hierarchy panel keeps Shift = range
                                // (list-style UX); the canvas has no
                                // natural linear order, so toggle is the
                                // sane semantic for both modifiers.
                                hero.gizmo.toggle_in_selection(bits);
                            } else if preserves_multi {
                                // Onda 2 hotfix: bare click on a sprite
                                // already in the multi-selection DEFERS
                                // the decision to PointerUp. If the user
                                // drags from here, the open Translate
                                // drag becomes a group translate (Onda 1
                                // semantics preserved). If they release
                                // without dragging, Up `replace_selection`
                                // collapses the multi to just this sprite
                                // (Enio: "se há multiplas sprites
                                // selecionas e eu clicar com botão
                                // esquerdo em uma delas, todas as outras
                                // devem ser desselecionadas").
                                self.pending_single_replace = Some((bits, (evt.x, evt.y)));
                            } else {
                                hero.gizmo.replace_selection(Some(bits));
                            }
                        } else {
                            // Empty click — Fase 0f: defer to PointerKind::Up
                            // so we can distinguish "bare click on empty"
                            // (= clear selection) from "start of a rubber-
                            // band box-select drag" (= keep selection
                            // until release, then resolve against the
                            // dragged rect). Cmd on empty stays a no-op
                            // (preserves built-up multi-selection). Shift
                            // on empty starts an additive rubber-band.
                            if !cmd_held {
                                self.rubber_band = Some(crate::app_state::RubberBandState {
                                    anchor_screen: (evt.x, evt.y),
                                    current_screen: (evt.x, evt.y),
                                    add_mode: shift_held,
                                });
                            }
                        }
                        if let Some(bits) = picked
                            && !is_modifier_click
                        {
                            let entity = ph2d_ecs::Entity::from_bits(bits);
                            if !ph2d_ecs::is_locked_for_edit(gfx.sim.world(), entity)
                                && let Some(t) = gfx.sim.world().get::<Transform>(entity)
                            {
                                let snap_t = ph2d_editor::TransformSnapshot {
                                    translation: [t.translation.x, t.translation.y],
                                    rotation: t.rotation,
                                    scale: [t.scale.x, t.scale.y],
                                };
                                let pw = ph2d_ecs::parent_world_transform(gfx.sim.world(), entity);
                                let parent_world = ph2d_editor::TransformSnapshot {
                                    translation: [pw.translation.x, pw.translation.y],
                                    rotation: pw.rotation,
                                    scale: [pw.scale.x, pw.scale.y],
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
                                    target: ph2d_editor::GizmoTarget::PrimaryIndividual,
                                    parent_world,
                                });
                                // Onda 1 + 2C.4: snapshot every OTHER
                                // selected sprite's full start_transform
                                // (skip the drag's own primary — its
                                // snapshot lives on GizmoDragState).
                                // Canvas pick always opens a Translate
                                // drag; the same snapshots also feed
                                // future scale/rotate handles on extras +
                                // global (advance_gizmo_drag dispatches
                                // by drag.kind + drag.target).
                                self.group_drag_starts.clear();
                                if hero.gizmo.selected_len() > 1 {
                                    for sel in hero.gizmo.iter_selected() {
                                        if sel == bits {
                                            continue;
                                        }
                                        let e = ph2d_ecs::Entity::from_bits(sel);
                                        if let Some(t) = gfx.sim.world().get::<Transform>(e) {
                                            let epw = ph2d_ecs::parent_world_transform(
                                                gfx.sim.world(),
                                                e,
                                            );
                                            self.group_drag_starts.push(
                                                crate::app_state::GroupDragSnapshot {
                                                    entity_bits: sel,
                                                    start_transform:
                                                        ph2d_editor::TransformSnapshot {
                                                            translation: [
                                                                t.translation.x,
                                                                t.translation.y,
                                                            ],
                                                            rotation: t.rotation,
                                                            scale: [t.scale.x, t.scale.y],
                                                        },
                                                    parent_world: ph2d_editor::TransformSnapshot {
                                                        translation: [
                                                            epw.translation.x,
                                                            epw.translation.y,
                                                        ],
                                                        rotation: epw.rotation,
                                                        scale: [epw.scale.x, epw.scale.y],
                                                    },
                                                },
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        // ADR-0029 Phase C.2: live entries owned by the
                        // Hierarchy panel crate; reach via the public
                        // thread-local snapshot. With multi-select the
                        // label mirrors the primary; the count is
                        // surfaced via hero.gizmo.selected_len() at
                        // paint time (Fase 0e polish).
                        let primary = hero.gizmo.selection;
                        if let Some(entry) = resolve_live_entry(gfx.hero_live.as_ref(), primary) {
                            hero.selection = Some(ph2d_editor::HeroSelection {
                                label: entry.name.clone(),
                                kind: entry.badge.clone().unwrap_or_else(|| "ENT".to_string()),
                                world_pos: (0.0, 0.0),
                            });
                        } else if primary.is_none() {
                            hero.selection = None;
                        }
                        self.title_dirty = true;
                    }
                }
                PointerKind::Up => {
                    // Fase 0f: resolve the rubber-band rect — pick every
                    // sprite whose world bbox intersects, then apply
                    // replace or add depending on `add_mode` (Shift held
                    // at Down). A click that didn't drift more than 4 px
                    // is treated as a bare click on empty: clear
                    // selection if !add_mode, else preserve.
                    if let Some(rb) = self.rubber_band.take() {
                        let dx = rb.current_screen.0 - rb.anchor_screen.0;
                        let dy = rb.current_screen.1 - rb.anchor_screen.1;
                        let moved = (dx * dx + dy * dy) > 16.0; // > 4 px
                        if moved {
                            let window_size = gfx.surface.size();
                            let world_a = gfx.camera.screen_to_world(rb.anchor_screen, window_size);
                            let world_b =
                                gfx.camera.screen_to_world(rb.current_screen, window_size);
                            let rmin = [world_a[0].min(world_b[0]), world_a[1].min(world_b[1])];
                            let rmax = [world_a[0].max(world_b[0]), world_a[1].max(world_b[1])];
                            let bits = ph2d_render::pick_sprites_in_world_rect(
                                gfx.present.world_mut(),
                                rmin,
                                rmax,
                            );
                            if !rb.add_mode {
                                hero.gizmo.clear_all_selection();
                            }
                            for b in bits {
                                hero.gizmo.add_to_selection(b);
                            }
                            // Sync the panel header label to the new
                            // primary (Fase 0e parity).
                            let primary = hero.gizmo.selection;
                            if let Some(entry) = resolve_live_entry(gfx.hero_live.as_ref(), primary)
                            {
                                hero.selection = Some(ph2d_editor::HeroSelection {
                                    label: entry.name.clone(),
                                    kind: entry.badge.clone().unwrap_or_else(|| "ENT".to_string()),
                                    world_pos: (0.0, 0.0),
                                });
                            } else if primary.is_none() {
                                hero.selection = None;
                            }
                            self.title_dirty = true;
                        } else if !rb.add_mode {
                            // Bare click on empty = clear selection.
                            hero.gizmo.clear_all_selection();
                            hero.selection = None;
                            self.title_dirty = true;
                        }
                    }
                    // Onda 2 hotfix: resolve a pending click-vs-drag
                    // decision. `pending_single_replace` is Some when
                    // the user Down'd on a multi-selected sprite. If
                    // the cursor stayed within ~4 px of the Down point
                    // until now (a click, not a drag), collapse the
                    // multi-selection to just that sprite. If it moved
                    // past the threshold, the open Translate drag has
                    // already group-translated the selection; just
                    // clear the pending state.
                    if let Some((bits, (dx0, dy0))) = self.pending_single_replace.take() {
                        let dx = evt.x - dx0;
                        let dy = evt.y - dy0;
                        // 12 px tolerance — trackpads have micro
                        // tremor and acceleration that can move
                        // the cursor a few px even on what feels
                        // like a stationary click.
                        if (dx * dx + dy * dy) <= 144.0 {
                            hero.gizmo.replace_selection(Some(bits));
                            // Sync the panel header label to the new
                            // primary so the Hierarchy highlight
                            // matches the canvas immediately.
                            if let Some(entry) =
                                resolve_live_entry(gfx.hero_live.as_ref(), Some(bits))
                            {
                                hero.selection = Some(ph2d_editor::HeroSelection {
                                    label: entry.name.clone(),
                                    kind: entry.badge.clone().unwrap_or_else(|| "ENT".to_string()),
                                    world_pos: (0.0, 0.0),
                                });
                            }
                            self.title_dirty = true;
                        }
                    }
                    // Drop the drag — Transform is already committed
                    // up to the latest Move position.
                    hero.gizmo.drag = None;
                    // Onda 1: release the group-translate snapshot so
                    // the next single-select drag doesn't accidentally
                    // pull stale extras along.
                    self.group_drag_starts.clear();
                    // Onda 2 polish: release the global drag-start view
                    // so snapshots::publish reverts to the live-union
                    // computation for the next frame.
                    hero.gizmo.global_view_start = None;
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
