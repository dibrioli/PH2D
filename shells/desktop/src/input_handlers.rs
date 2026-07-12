//! Large `impl App` input-handler methods extracted from `main.rs`
//! as a split impl block (Wave 3.2 stage B).
//!
//! Each method is a `&mut self` call from `ApplicationHandler` /
//! `render_frame` — see `main.rs` for the call sites. Lifted
//! verbatim; behavior-preserving.

use crate::App;
use ph2d_editor::interaction::InteractiveState;
use ph2d_editor::zones::Rect as EditorRect;
use ph2d_editor::{PanelControl, PanelEvent, Toast};
use ph2d_render::Camera2d;
use winit::keyboard::KeyCode;

impl App {
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
    /// Whether the Audio Editor owns Cmd/Ctrl+X / +C / +V right now: its panel is open and a clip
    /// is loaded.
    ///
    /// Read **before** the `&mut gfx` borrow below (the same dance `over_motion_graph` does), and
    /// used as a match **guard** rather than a check inside the arm. An arm that matched
    /// unconditionally would *consume* the chord even with the editor closed — swallowing it so
    /// that nothing at all happens, which is the most confusing possible outcome and exactly how a
    /// clipboard shortcut ends up "sometimes not working". Not owning it means not matching it.
    #[cfg(feature = "panel-audio-editor")]
    fn audio_editor_owns_clipboard(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.is_panel_visible("audio_editor"))
            && self.audio.as_ref().is_some_and(|a| a.editor_loaded())
    }

    pub(crate) fn handle_editor_key(&mut self, code: KeyCode) {
        // Computed before the `&mut gfx` borrow (Motion Nodes M1): F over the
        // graph fits the graph, not the scene. Same for the timeline (W2.E6).
        let over_motion_graph = self.cursor_over_motion_graph();
        let over_timeline = self.cursor_over_timeline();
        #[cfg(feature = "panel-audio-editor")]
        let audio_clipboard = self.audio_editor_owns_clipboard();
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
            // Motion Nodes F2: Ctrl+D duplicates the selection, `K` arms the knife,
            // `P` arms the probe. Pushed HERE, on the same proven cursor check as
            // F / Delete / A / Space below — `dispatch/key.rs` also maps these three,
            // but that path goes through the focus gate this file exists to bypass,
            // and the panel never saw them (Enio, smoke 2026-07-12: "Ctrl+D não
            // duplica"). The panel's handling is idempotent, so a double push from
            // both paths is harmless.
            //
            // **These arms must sit ABOVE the global `K`** (timeline insert-key)
            // — a match arm below it is unreachable, and the knife would silently
            // never arm. Clippy's `unreachable_pattern` is what caught it.
            KeyCode::KeyD if over_motion_graph && cmd_chord => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    hero.store
                        .push_graph_key(ph2d_editor::interaction::GraphKey::Duplicate);
                }
            }
            KeyCode::KeyK if over_motion_graph && !cmd_chord => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    hero.store
                        .push_graph_key(ph2d_editor::interaction::GraphKey::Knife);
                }
            }
            KeyCode::KeyP if over_motion_graph && !cmd_chord => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    hero.store
                        .push_graph_key(ph2d_editor::interaction::GraphKey::Probe);
                }
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
            // The Audio Editor owns Cmd/Ctrl+X / +C / +V while its panel is open with a clip
            // loaded — the same ownership rule as its Ctrl+Z below, and for the same reason: a
            // focused modal editor that does not answer to the clipboard chords is an editor
            // people assume is broken. They are the first thing anyone tries.
            //
            // Consumed unconditionally (the op runs only when it can), so the chord never falls
            // through to a global handler and does something surprising to the scene.
            #[cfg(feature = "panel-audio-editor")]
            KeyCode::KeyX | KeyCode::KeyC | KeyCode::KeyV
                if (self.modifiers.super_key() || self.modifiers.control_key())
                    && audio_clipboard =>
            {
                use ph2d_panel_audio_editor::AudioEditCmd as Cmd;
                let cmd = match code {
                    KeyCode::KeyX => Cmd::Cut,
                    KeyCode::KeyC => Cmd::Copy,
                    _ => Cmd::Paste,
                };
                if let Some(a) = self.audio.as_mut() {
                    a.editor_apply(cmd);
                }
            }
            KeyCode::KeyZ if self.modifiers.super_key() || self.modifiers.control_key() => {
                // Audio Editor owns Cmd/Ctrl+Z (undo) / +Shift (redo) while its WAVE
                // panel is open with a clip loaded: the user is editing audio there, so
                // keyboard undo steps the `EditClip` timeline, not the painter/image bus
                // or the global object undo.
                //
                // It consumes the chord UNCONDITIONALLY here (applies the op only when
                // there is one to apply, but always returns). The old code fell through
                // to the global undo once the audio timeline was exhausted "so global
                // undo still works when there's no audio edit" — but that made one extra
                // Ctrl+Z JUMP THE WHOLE SCENE (the global undo restores the WorldSnapshot
                // + clears selection). The audio-timeline boundary is invisible to the
                // user, so it read as "undo sometimes doesn't work / does something
                // weird" (2026-07-11 multi-agent audit, A1). A focused modal editor owns
                // its own Ctrl+Z, like the painter/motion/timeline undos do.
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
                        }
                        return;
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
            KeyCode::Comma | KeyCode::Period => {
                let back = code == KeyCode::Comma;
                // Com a tool Flip ativa o passo é UM QUADRO DO OBJETO (12/24 fps),
                // não um tick de simulação (60 Hz): senão "avançar um quadro" andaria
                // um quinto de desenho e o animador nunca cairia numa chave.
                let fps = self
                    .flip_active
                    .then(|| self.flip_fps())
                    .flatten()
                    .unwrap_or_else(|| 1.0 / self.playhead.fixed_dt());
                self.playhead.pause();
                let f = self.playhead.frame(fps);
                let to = if back { (f - 1).max(0) } else { f + 1 };
                self.playhead.seek_frame(to, fps);
            }
            // ADR-0114 W3.T3.5 — **o flip do animador**: as setas pulam por DESENHO
            // (não por quadro), atravessando os holds. É o inner loop da profissão:
            // ir e voltar entre os dois desenhos que se está comparando.
            KeyCode::ArrowUp | KeyCode::ArrowDown if self.flip_active => {
                self.flip_step_drawing(code == KeyCode::ArrowDown);
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
