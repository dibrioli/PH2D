//! Large `impl App` input-handler methods extracted from `main.rs`
//! as a split impl block (Wave 3.2 stage B).
//!
//! Each method is a `&mut self` call from `ApplicationHandler` /
//! `render_frame` — see `main.rs` for the call sites. Lifted
//! verbatim; behavior-preserving.

use crate::App;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::zones::Rect as EditorRect;
use ph2d_editor_core::{PanelControl, PanelEvent, Toast};
use ph2d_render::Camera2d;
use winit::keyboard::KeyCode;

/// Os ramos do `handle_editor_key` (os braços do `match code` depois do `T`).
#[path = "input_dispatch/handlers_teclas_editor.rs"]
mod handlers_teclas_editor;

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

        // ── **The graph owns its keys while the cursor is over it** ───────────────
        //
        // Enio, smoke 2026-07-13: *"se o mouse estiver em Motion, o tool motion deve
        // capturar os atalhos e não o canvas"* — `Ctrl+G` was toggling the scene GRID
        // instead of grouping.
        //
        // This used to be seven bespoke arms below (D / K / P / F / Delete / A /
        // Space), each re-stating a verb the graph's keymap already knew. That second
        // list is precisely the defect: the graph grew `Ctrl+G` and the shell had never
        // heard of it, so the key fell through to the global `G`. **So there is one map
        // now** ([`graph_key_for`], in editor-core) and this is its only other reader.
        //
        // It runs on the CURSOR rather than on the focus gate inside `dispatch_key`,
        // which is the whole reason this router exists: a stale focus elsewhere in the
        // editor used to swallow the graph's keys (the "Ctrl+D não duplica" smoke).
        // Both paths CAN push the same verb in one frame, so the panel COLLAPSES an
        // adjacent repeat on drain (`dedup_double_dispatch`) — otherwise a
        // non-idempotent verb (Paste, Duplicate) would run twice, two copies from one
        // Ctrl+V. The verbs need not be idempotent for the double to be harmless.
        if over_motion_graph
            && let Some(kc) = crate::keymap::winit_to_editor_keycode(code)
            && let Some(gk) = ph2d_editor_core::interaction::graph_key_for(
                kc,
                cmd_chord,
                self.modifiers.alt_key(),
            )
        {
            if let Some(hero) = gfx.hero_screen.as_mut() {
                hero.store.push_graph_key(gk);
            }
            // CONSUMED. Falling through would let the scene act on it too — which is
            // how `G` toggled the grid, and how `F` would fit the scene behind the
            // graph it just fitted.
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
            // ⚠️ **ANTES do `KeyCode::KeyK` de baixo, e a ordem é a lei.** O arm do `K` não olha
            // para os modificadores, então hoje `Ctrl+K` insere um keyframe por acidente — nada o
            // documenta, e a tecla documentada é o `K` nu. Um arm guardado colocado ACIMA vence
            // (o `match` do Rust é ordenado), e o `K` nu fica exactamente como estava.
            // ⭐ **O PIE MENU** (estudo de UI viva, E4): segurar `P` abre as ferramentas em OITO
            // direcções sob o cursor; soltar escolhe.
            //
            // ⚠️ **`P` foi MEDIDO, não escolhido.** O botão do meio é o pan da câmara (o idioma do
            // Blender, deliberado) e das 26 letras só **nove** estão livres sem modificador
            // (H I J M N P U V Y) — o `Q` do Blender está tomado. `P` é *pie menu*, o nome real do
            // widget, e sobrevive como mnemónica.
            //
            // ⚠️ Sobre o grafo do Motion o `P` é dele: aquele router já devolveu antes deste
            // `match`, e esta lei não o alcança.
            KeyCode::KeyP if !cmd_chord => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    let items = ph2d_editor_core::screens::hero::radial::build_radial_model(hero);
                    let center = [self.last_pointer.0, self.last_pointer.1];
                    if !hero.store.open_radial(center, items) {
                        gfx.toasts.push(Toast::info("No tools to show here (P)"));
                    }
                }
            }
            KeyCode::KeyK if cmd_chord => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    crate::global_palette_input::open_global_palette(hero);
                }
            }
            KeyCode::KeyM => {
                gfx.theme = gfx.theme.next();
                gfx.toasts
                    .push(Toast::info(format!("Theme · {}", gfx.theme.id())));
                self.title_dirty = true;
            }
            // Contorno dos colliders. `B` estava livre (a demo de SpriteAnimation
            // que a usava foi aposentada no W4.T5 da timeline). W2 põe um
            // checkbox no painel de física lendo o MESMO flag — duas portas
            // para a mesma pergunta divergem.
            KeyCode::KeyB => {
                self.show_colliders = !self.show_colliders;
                gfx.toasts.push(Toast::info(if self.show_colliders {
                    "Colliders shown (B)"
                } else {
                    "Colliders hidden (B)"
                }));
            }
            // Toggle the Physics world panel (ADR-0131 D8 / W2b). Mirror of the
            // timeline's `L`: a world panel is not tool-gated, so it needs an
            // opener of its own or it is a feature nobody can reach. `W` for
            // World — audited free against the shell's whole shortcut set.
            KeyCode::KeyW => {
                let shown = if let Some(hero) = gfx.hero_screen.as_mut() {
                    let v = !hero.is_panel_visible("physics");
                    hero.panel_visibility.insert("physics", v);
                    v
                } else {
                    false
                };
                gfx.toasts.push(Toast::info(if shown {
                    "Physics shown (W)"
                } else {
                    "Physics hidden (W)"
                }));
                self.title_dirty = true;
            }
            // Toggle do painel de TOKENS (plano UI/UX W6) — a tabela de cor do design system.
            // Irmão do `W` da física e do `L` da timeline: um painel de MUNDO não é tool-gated,
            // então sem abridor próprio é feature que ninguém alcança.
            //
            // ⚠️ **A `T` era um scaffold de debug** (`"Toast key (T) pressed"`), órfão como o
            // `KeyB` que a timeline aposentou no W4.T5 — varrido no repo inteiro antes de ser
            // tomado, e nada dependia dele. `T` de Tokens.
            KeyCode::KeyT => {
                let shown = if let Some(hero) = gfx.hero_screen.as_mut() {
                    let v = !hero.is_panel_visible("tokens");
                    hero.panel_visibility.insert("tokens", v);
                    v
                } else {
                    false
                };
                gfx.toasts.push(Toast::info(if shown {
                    "Tokens shown (T)"
                } else {
                    "Tokens hidden (T)"
                }));
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
                let redo = self.modifiers.shift_key();
                self.undo_or_redo(redo);
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
            _ => self.ramo_teclas_editor_vista_e_transporte(code, cmd_chord, over_timeline),
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
