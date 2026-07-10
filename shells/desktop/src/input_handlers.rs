//! Large `impl App` input-handler methods extracted from `main.rs`
//! as a split impl block (Wave 3.2 stage B).
//!
//! Each method is a `&mut self` call from `ApplicationHandler` /
//! `render_frame` — see `main.rs` for the call sites. Lifted
//! verbatim; behavior-preserving.

use crate::App;
use crate::cursor_pos::live_cursor_in_window;
use crate::image_import::{ImportItemResult, import_images_grid};
use ph2d_editor::interaction::InteractiveState;
use ph2d_editor::zones::Rect as EditorRect;
use ph2d_editor::{PanelControl, PanelEvent, Toast};
use ph2d_render::Camera2d;
use winit::keyboard::KeyCode;

impl App {
    /// Imports each dropped path that resolves to an image, anchoring
    /// the spawn at the cursor's world position (CoreGraphics
    /// override on macOS where winit 0.30 doesn't update `last_cursor`
    /// during drag). Falls back to `self.last_cursor` on other
    /// platforms. Honors `grid_snap_state` so multi-drop forms a
    /// tidy grid.
    pub(crate) fn handle_dropped_files(&mut self, paths: &[std::path::PathBuf]) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(hero) = gfx.hero_screen.as_ref() else {
            return;
        };
        let pixels_per_meter = hero.project.pixels_per_meter;
        let win = gfx.surface.size();
        // macOS-only: winit 0.30 does NOT emit `CursorMoved` during
        // external file drag operations (see
        // winit-0.30.13/src/platform_impl/macos/window_delegate.rs —
        // `draggingEntered:` doesn't extract `draggingLocation`, no
        // `draggingUpdated:` is implemented at all). So `last_cursor`
        // is whatever it was BEFORE the drag started, not where the
        // file was actually dropped. Query the live cursor from
        // CoreGraphics (`macos_cursor_query`) to override; fall back
        // to `last_cursor` on other platforms.
        let cursor_px = self
            .host
            .as_ref()
            .and_then(|h| live_cursor_in_window(h.window()))
            .unwrap_or(self.last_cursor);
        let drop_world_raw = gfx.camera.screen_to_world(cursor_px, win);
        // Grid-snap apply (drag-drop site). When snap is enabled in
        // `grid_snap_state`, align the drop position to the active
        // grid before spawning so a multi-sprite drop forms a tidy
        // grid rather than scattering at sub-pixel offsets. No-op
        // when snap_enabled = false or active kind has no snap target.
        let drop_world: [f32; 2] = if let Some(hero) = gfx.hero_screen.as_mut() {
            // Drag-drop: sprite hasn't been imported yet, so half-size
            // is unknown. Pass [0.0, 0.0] — Corner-family modes
            // degenerate to point-Intersection snap in that case.
            hero.grid.snap_state.snap_world(drop_world_raw, [0.0, 0.0])
        } else {
            drop_world_raw
        };
        // Filter to image files up front (warning toast per skip), then
        // hand the survivors to the batch importer, which lays them out
        // in a near-square grid anchored at the drop point (`drop_world`
        // = first cell's center; the grid grows right + down). This
        // replaces the old per-file `camera.center` shuffle — the
        // importer takes the world anchor directly.
        let mut valid_paths: Vec<std::path::PathBuf> = Vec::new();
        for path in paths {
            if ph2d_asset::is_supported_image_extension(path) {
                valid_paths.push(path.clone());
            } else {
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("(unnamed)");
                gfx.toasts
                    .push(Toast::warning(format!("Skipped non-image: {name}")));
                self.title_dirty = true;
            }
        }
        if valid_paths.is_empty() {
            return;
        }
        let results = import_images_grid(
            &mut gfx.sim,
            &mut gfx.renderer,
            &gfx.asset_db,
            drop_world,
            &mut gfx.next_import_cell,
            &valid_paths,
            pixels_per_meter,
            &mut gfx.atlas_asset_map,
        );
        // Seat the selection: the first imported sprite replaces the
        // selection, the rest join it as extras so a multi-file drop
        // ends up fully selected (same shape as Shift-clicking each on
        // the canvas). The per-frame snapshot sync
        // (render_loop/snapshots.rs) derives both the canvas gizmo view
        // and the Hierarchy row highlight from `hero.gizmo`, so this one
        // write covers both surfaces.
        let mut selected_any = false;
        for r in results {
            match r {
                ImportItemResult::Ok { label, bits } => {
                    if let Some(hero) = gfx.hero_screen.as_mut() {
                        if selected_any {
                            hero.gizmo.add_to_selection(bits);
                        } else {
                            hero.gizmo.replace_selection(Some(bits));
                            selected_any = true;
                        }
                    }
                    gfx.toasts.push(Toast::success(format!("Imported {label}")));
                    self.title_dirty = true;
                }
                ImportItemResult::Err { name, error } => {
                    eprintln!("M14.4e drop failed ({name}): {error}");
                    gfx.toasts
                        .push(Toast::error(format!("Drop failed: {error}")));
                    self.title_dirty = true;
                }
            }
        }
    }

    /// M12 demo control router.
    ///   Tab — toggle ZenMode (debounced 30 frames)
    ///   M   — flip theme Dark↔Light
    ///   T   — push info toast
    ///   Cmd+Z / Ctrl+Z — image-edit undo
    ///   F / Home — frame the current selection
    ///   G — toggle grid visibility
    ///
    /// **Focus gate:** when a text-editable widget (TextInput /
    /// NumberInput / Combobox) holds keyboard focus AND no Cmd/Ctrl
    /// modifier is held, the entire match below is short-circuited —
    /// otherwise typing "M" / "T" / "G" / "1" into a chip would also
    /// flip the theme / push a toast / toggle the grid / activate a
    /// tool. Chord shortcuts (Cmd+Z undo, etc.) still pass through so
    /// editing-time undo keeps working.
    ///
    /// Tool-switch digits (1/2/3) were retired in favour of the
    /// canvas tool palette + Image Tools chrome pills — they were the
    /// loudest source of the text-input conflict (Color Equalization
    /// "Tile Grid" chip swallows digits all day).
    pub(crate) fn handle_editor_key(&mut self, code: KeyCode) {
        // Computed before the `&mut gfx` borrow (Motion Nodes M1): F over the
        // graph fits the graph, not the scene. Same for the timeline (W2.E6).
        let over_motion_graph = self.cursor_over_motion_graph();
        let over_timeline = self.cursor_over_timeline();
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let cmd_chord = self.modifiers.super_key() || self.modifiers.control_key();
        if !cmd_chord
            && let Some(hero) = gfx.hero_screen.as_ref()
            && let Some(focused) = hero.store.focus_id()
            && matches!(
                hero.store.get(focused),
                Some(InteractiveState::TextInput { .. })
                    | Some(InteractiveState::NumberInput { .. })
                    | Some(InteractiveState::Combobox { .. })
            )
        {
            // A text input is collecting keystrokes — let it own the
            // event entirely. The dispatch already pushed the buffer
            // mutation upstream; we just refuse to interpret the same
            // key as a global shortcut.
            return;
        }
        match code {
            KeyCode::Tab if gfx.zen.try_toggle() => {
                let msg = if gfx.zen.is_active() {
                    "Zen mode ON (zones collapsed)"
                } else {
                    "Zen mode OFF (zones restored)"
                };
                gfx.toasts.push(Toast::info(msg));
                self.title_dirty = true;
            }
            KeyCode::KeyM => {
                gfx.theme = gfx.theme.next();
                gfx.toasts
                    .push(Toast::info(format!("Theme · {}", gfx.theme.id())));
                self.title_dirty = true;
            }
            KeyCode::KeyT => {
                gfx.toasts.push(Toast::info("Toast key (T) pressed"));
                self.title_dirty = true;
            }
            // Toggle the bottom-docked general timeline panel (W2.E0). Transport
            // (Space / , / .) already drives the Playhead; this shows/hides the
            // editor. Free key — audited against the existing shortcut set.
            KeyCode::KeyL => {
                let shown = if let Some(hero) = gfx.hero_screen.as_mut() {
                    let v = !hero.is_panel_visible("timeline");
                    hero.panel_visibility.insert("timeline", v);
                    v
                } else {
                    false
                };
                gfx.toasts.push(Toast::info(if shown {
                    "Timeline shown (L)"
                } else {
                    "Timeline hidden (L)"
                }));
                self.title_dirty = true;
            }
            // Insert a keyframe at the playhead on every track bound to the
            // selected sprite (captures its current pose). Processed next frame
            // in the render loop, which has the world to sample from.
            KeyCode::KeyK => {
                self.timeline_insert_key = true;
            }
            // Cmd+Z / Ctrl+Z — context-sensitive undo. With the Painter
            // tool active it undoes the last brush stroke (W2.T2.2;
            // Cmd+Shift+Z redoes); with any other tool it falls back to
            // the single-level image-edit undo (Trim, Make Square, Bg
            // Removal — Wave 2.5 PR 11.8b3 bus migration). Tool identity
            // is matched by id only (no concrete downcast) so the
            // shell-downcast arch gate stays green; the actual stroke
            // undo runs in `painter_bridge::dispatch`, the downcast-
            // allowed site, via the transient flags set here.
            KeyCode::KeyZ if self.modifiers.super_key() || self.modifiers.control_key() => {
                // Audio Editor owns Cmd/Ctrl+Z (undo) / +Shift (redo) while its
                // WAVE panel is open with a clip loaded: the user is editing audio
                // there, so keyboard undo must step the `EditClip` timeline rather
                // than the painter/image bus. Only consumes the key when the editor
                // can actually act — otherwise it falls through to the painter/image
                // undo below, so global undo still works when there's no audio edit.
                #[cfg(feature = "panel-audio-editor")]
                {
                    let audio_open = gfx
                        .hero_screen
                        .as_ref()
                        .is_some_and(|h| h.is_panel_visible("audio_editor"));
                    if audio_open
                        && let Some(a) = self.audio.as_mut()
                        && a.editor_loaded()
                    {
                        let redo = self.modifiers.shift_key();
                        let can = if redo {
                            a.editor_can_redo()
                        } else {
                            a.editor_can_undo()
                        };
                        if can {
                            a.editor_apply(if redo {
                                ph2d_panel_audio_editor::AudioEditCmd::Redo
                            } else {
                                ph2d_panel_audio_editor::AudioEditCmd::Undo
                            });
                            return;
                        }
                    }
                }
                let painter_active = gfx
                    .tools
                    .active()
                    .map(|t| t.id() == ph2d_editor::ToolId::new("painter"))
                    .unwrap_or(false);
                let redo = self.modifiers.shift_key();
                // Undo GLOBAL de objetos (fila única, ADR-0110+): o fallback do Ctrl+Z
                // quando nenhum domínio (painter/audio/motion/timeline/vetor) o
                // consumiu. Só arma o request se há um passo naquele sentido; senão
                // cai para o image-edit undo de sempre (compat).
                let global_has = if redo {
                    self.undo.can_redo()
                } else {
                    self.undo.can_undo()
                };
                if painter_active {
                    if redo {
                        self.painter_redo_requested = true;
                    } else {
                        self.painter_undo_requested = true;
                    }
                } else if global_has {
                    self.undo_request = Some(redo);
                } else if let Some(hero) = gfx.hero_screen.as_mut() {
                    hero.bus
                        .push(ph2d_editor::action_bus::EditorAction::UndoImageEdit);
                }
            }
            // Cmd/Ctrl+Y — redo in the Audio Editor (the Windows/Linux redo chord,
            // alongside Cmd/Ctrl+Shift+Z). No-op unless the WAVE panel is open with
            // a clip that has something to redo.
            #[cfg(feature = "panel-audio-editor")]
            KeyCode::KeyY if cmd_chord => {
                let audio_open = gfx
                    .hero_screen
                    .as_ref()
                    .is_some_and(|h| h.is_panel_visible("audio_editor"));
                if audio_open
                    && let Some(a) = self.audio.as_mut()
                    && a.editor_loaded()
                    && a.editor_can_redo()
                {
                    a.editor_apply(ph2d_panel_audio_editor::AudioEditCmd::Redo);
                }
            }
            // Cmd+Enter / Ctrl+Enter — commit the active Painter stroke into
            // the sprite WITHOUT switching tools (W2.T2.5). Sets a transient
            // flag consumed by `painter_bridge::dispatch` (the only downcast-
            // allowed site), which calls `PainterTool::request_commit`. If
            // Painter isn't active the flag is just taken and ignored. No
            // concrete-tool downcast here — keeps
            // `architecture_no_downcast_to_concrete_tool_in_shell` green.
            KeyCode::Enter if self.modifiers.super_key() || self.modifiers.control_key() => {
                self.painter_commit_requested = true;
            }
            // Cmd/Ctrl+N — open the New-image modal (square size + background). The render loop polls
            // `store.take_new_image_request()` and spawns the chosen blank canvas (see
            // `painter_bridge::service_new_image_request`). The demo's atlas sprites are 64px, which
            // distorts the brush↔canvas ratio; a freshly-sized canvas is the canonical brush smoke target.
            KeyCode::KeyN if cmd_chord => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    hero.store.open_new_image_dialog();
                }
            }
            // Digit shortcuts (1=Brush, 2=Move, 3=BgRemoval) retired
            // — they collided with every numeric chip in the Image
            // Tools panels (Color EQ Tile Grid, Equalize Sizes Fixed
            // W/H, Upscale Scale, …). Tool switching now goes through
            // the canvas tool palette + Image Tools chrome pills.
            // M14.7 polish: F / Home = frame the currently selected
            // sprite. Falls back to (0, 0) when nothing is selected
            // (Blender / Maya "frame view" semantics). Raises a
            // pending intent on the hero — the render_frame drain
            // resolves the selection and updates `gfx.camera`.
            // Motion Nodes M1: over the graph panel, F fits the GRAPH. Pushed
            // directly (not via the graph_focused/focus_id dispatch gate, which
            // could be blocked by a stale focus) using the proven cursor check —
            // and the arm suppresses the scene frame so the two don't both fit
            // (Blender per-area focus).
            KeyCode::KeyF if over_motion_graph => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    hero.store
                        .push_graph_key(ph2d_editor::interaction::GraphKey::Fit);
                }
            }
            // Timeline W2.E6: over the dope sheet, F fits the TIME AXIS to the
            // keys. Same per-area focus rule as the graph, and it likewise
            // suppresses the scene frame below so only one thing fits. The view
            // transform is panel state, so this raises a request the panel's
            // `paint` consumes (it alone knows the time area's pixel width).
            KeyCode::KeyF if over_timeline => ph2d_panel_timeline::request_fit(),
            // Motion Nodes M1.E7: over the graph, Delete/Backspace removes the
            // selected nodes (+ orphan edges) and `A` opens the add-node menu.
            // Pushed directly on the proven cursor check (same rationale as F);
            // the panel's Delete/Add handling is idempotent, so the parallel M0
            // focus-gated dispatch pushing the same verb is harmless.
            KeyCode::Delete | KeyCode::Backspace if over_motion_graph => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    hero.store
                        .push_graph_key(ph2d_editor::interaction::GraphKey::Delete);
                }
            }
            KeyCode::KeyA if over_motion_graph && !cmd_chord => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    hero.store
                        .push_graph_key(ph2d_editor::interaction::GraphKey::Add);
                }
            }
            // Motion Nodes M1: over the graph, Space toggles transport play/pause
            // (so time-driven behaviours animate). The render output is the
            // `Output` node — wire a chain into one, no keyboard verb needed.
            KeyCode::Space if over_motion_graph => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    hero.store
                        .push_graph_key(ph2d_editor::interaction::GraphKey::TogglePlay);
                }
            }
            KeyCode::Home | KeyCode::KeyF => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    // Wave 2.5 PR 11.8d: bus migration (was
                    // `hero.pending_view_focus = Some(...)`).
                    hero.bus
                        .push(ph2d_editor::action_bus::EditorAction::SetViewFocus {
                            kind: ph2d_editor::ViewFocusKind::Selected,
                        });
                } else {
                    // No hero panel — fall back to legacy "reset
                    // camera" so the non-editor demo mode still has
                    // a way to recover from a bad pan/zoom.
                    gfx.camera = Camera2d::default();
                    gfx.toasts.push(Toast::info("Camera · reset"));
                }
                self.title_dirty = true;
            }
            // M14.4b: toggle grid visibility. The context-menu entry
            // promises "Show Grid · G" — this is the shortcut. Affects
            // only the hero's grid_visible flag; grid_view publishing
            // by the host continues regardless.
            KeyCode::KeyG => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    hero.view.grid_visible = !hero.view.grid_visible;
                    let msg = if hero.view.grid_visible {
                        "Grid · on"
                    } else {
                        "Grid · off"
                    };
                    gfx.toasts.push(Toast::info(msg));
                    self.title_dirty = true;
                }
            }
            // Timeline transport (M0 general timeline): drive the engine
            // Playhead. Space toggles play/pause; ',' / '.' step one frame
            // back / forward (pausing, so you land on the frame). Every
            // animatable system samples the Playhead for the current frame.
            KeyCode::Space => {
                let playing = self.playhead.toggle_play();
                gfx.toasts.push(Toast::info(if playing {
                    "Timeline · play"
                } else {
                    "Timeline · pause"
                }));
            }
            // Bind a demo spin animation to every sprite so the REAL general-
            // timeline path (SpriteAnimation -> apply at Playhead -> Transform ->
            // propagate -> render) can be eyeballed without the authoring panel.
            // Press Space to play. Rotation-only, so it also shows on the demo's
            // bouncing sprites (their translation keeps bouncing; they spin too).
            KeyCode::KeyB => {
                let clip = demo_spin_clip();
                let world = gfx.sim.world_mut();
                let ids: Vec<ph2d_ecs::Entity> = world
                    .query_filtered::<ph2d_ecs::Entity, ph2d_ecs::With<ph2d_ecs::Transform>>()
                    .iter(world)
                    .collect();
                let n = ids.len();
                for e in ids {
                    world
                        .entity_mut(e)
                        .insert(ph2d_timeline::SpriteAnimation::new(clip.clone()));
                }
                self.playhead.play();
                gfx.toasts.push(Toast::info(format!(
                    "Timeline · bound spin to {n} sprite(s) — Space to pause"
                )));
            }
            KeyCode::Comma => {
                self.playhead.pause();
                let fps = 1.0 / self.playhead.fixed_dt();
                let f = self.playhead.frame(fps);
                self.playhead.seek_frame((f - 1).max(0), fps);
            }
            KeyCode::Period => {
                self.playhead.pause();
                let fps = 1.0 / self.playhead.fixed_dt();
                let f = self.playhead.frame(fps);
                self.playhead.seek_frame(f + 1, fps);
            }
            _ => {}
        }
    }

    /// Hit-test the active tool's panel at `(px, py)` and dispatch a
    /// [`PanelEvent`] into the tool. `is_press` distinguishes the
    /// initial mouse-down (which may start a drag) from continued
    /// move-while-dragging (which only updates an in-progress slider).
    pub(crate) fn dispatch_panel_pointer(&mut self, px: f32, py: f32, is_press: bool) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        if gfx.zen.is_active() {
            return; // panels hidden
        }
        let Some(tool) = gfx.tools.active() else {
            return;
        };
        let panel = tool.build_panel();
        let viewport = EditorRect::new(
            0.0,
            0.0,
            gfx.surface.size().width as f32,
            gfx.surface.size().height as f32,
        );
        let widget_rects = panel.control_widget_rects(viewport);

        // Existing drag → re-emit SetValue against the same node. Done
        // even if pointer left the original cell (slider-style "live
        // drag" feel).
        if let Some(dragging_id) = self.dragging
            && let Some((idx, ctrl)) = panel
                .controls
                .iter()
                .enumerate()
                .find(|(_, c)| matches!(c, PanelControl::Slider(s) if s.id == dragging_id))
            && let Some(rect) = widget_rects.get(idx)
            && let PanelControl::Slider(_) = ctrl
        {
            let v = ((px - rect.x) / rect.w).clamp(0.0, 1.0) as f64;
            if let Some(active) = gfx.tools.active_mut() {
                active.handle_panel_event(PanelEvent::SetValue(dragging_id, v));
            }
            return;
        }

        if !is_press {
            return; // not a click and not a drag — nothing to do
        }

        // Find the cell containing (px, py).
        let Some((idx, _)) = widget_rects
            .iter()
            .enumerate()
            .find(|(_, r)| r.contains(px, py))
        else {
            return;
        };
        let ctrl = &panel.controls[idx];
        let rect = widget_rects[idx];

        let event = match ctrl {
            PanelControl::Slider(s) => {
                self.dragging = Some(s.id);
                let v = ((px - rect.x) / rect.w).clamp(0.0, 1.0) as f64;
                Some(PanelEvent::SetValue(s.id, v))
            }
            PanelControl::Toggle(t) => Some(PanelEvent::Toggle(t.id, !t.on)),
            PanelControl::RadioGroup(g) if !g.options.is_empty() => {
                // Horizontal split — pick option by which sub-rect
                // contains the pointer.
                let opt_w = rect.w / g.options.len() as f32;
                let opt_idx = (((px - rect.x) / opt_w) as usize).min(g.options.len() - 1);
                Some(PanelEvent::SelectOption(
                    g.id,
                    g.options[opt_idx].value.clone(),
                ))
            }
            PanelControl::ColorSwatch(s) => Some(PanelEvent::Click(s.id)),
            PanelControl::Action(_) | PanelControl::RadioGroup(_) => None,
        };

        if let Some(event) = event
            && let Some(active) = gfx.tools.active_mut()
        {
            active.handle_panel_event(event);
            self.title_dirty = true;
            // Wave 10 / Etapa 3 audit fix [C3]: removed the BgRemoval
            // Apply Toggle drain that was here. Two reasons:
            //
            // 1. **Multi-select regression**: it pushed a single
            //    `OneShotImageOp { entity_bits: hero.gizmo.selection }`
            //    (primary only), but the canonical bridge in
            //    `render_loop/bgremoval_preview.rs` uses
            //    `drive_pending_commit(bg, hero.gizmo.iter_selected())`
            //    — multi-sprite. Two competing drains of the same
            //    `take_pending_apply` (destructive) meant the input-
            //    handlers path won and the bridge always saw `false`,
            //    so multi-select Apply via panel toggle only baked the
            //    primary sprite. Regression silently introduced before
            //    Wave 10.
            //
            // 2. **Trait surface coverage**: with BgR (Etapa 1.B) +
            //    CEQ + Upscale (Etapa 2) all on RasterEditTool, the
            //    bridges' `drive_pending_commit` is the canonical
            //    drain for every raster Apply path. Keeping a parallel
            //    drain here would re-introduce the bug for the new
            //    tools too.
            //
            // The bridge runs every frame BEFORE `paint_hero_screen`
            // (per `render_loop/mod.rs::dispatch_bus_drain` order), so
            // latency between the click and the OneShotImageOp is at
            // most 1 frame — visually equivalent to the old path.
        }
    }
}

/// A demo `Clip` for the `KeyB` visual bind: one full CCW turn over 2 s, looping
/// (2π wraps back to 0 seamlessly). Rotation-only so it shows on any sprite.
fn demo_spin_clip() -> ph2d_anim::Clip {
    use ph2d_anim::{AnimValue, Clip, Interp, Key, RationalTime, Track};
    let rot = Track::new(vec![
        Key {
            t: RationalTime::from_seconds(0.0),
            value: AnimValue::Float(0.0),
            interp: Interp::Linear,
        },
        Key {
            t: RationalTime::from_seconds(2.0),
            value: AnimValue::Float(std::f32::consts::TAU),
            interp: Interp::Hold,
        },
    ]);
    Clip::new(RationalTime::from_seconds(2.0))
        .with_track(ph2d_timeline::SpriteProp::Rotation.target(), rot)
}
