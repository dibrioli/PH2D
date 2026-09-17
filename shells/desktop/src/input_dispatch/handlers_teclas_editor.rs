//! **Os atalhos do editor, a segunda metade do `match code`** — ramos do `handle_editor_key` ([`super`]): o `match`
//! parte-se pelo braço `_`, e os braços seguintes, PELA MESMA ORDEM e verbatim, formam o `match code` do ramo. ⚠️ É
//! FILHO do `input_handlers` (por `#[path]`), para ver os métodos privados de lá sem mudar a visibilidade de nenhum.

use super::*;
use ph2d_i18n::tr;

impl crate::App {
    /// Enquadrar (grafo, timeline ou cena), a grelha, o transporte (Space, vírgula/ponto) e o flip do animador.
    pub(super) fn ramo_teclas_editor_vista_e_transporte(
        &mut self,
        code: KeyCode,
        cmd_chord: bool,
        over_timeline: bool,
    ) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        match code {
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
            // Timeline W2.E6: over the dope sheet, F fits the TIME AXIS to the
            // keys. Same per-area focus rule as the graph, and it likewise
            // suppresses the scene frame below so only one thing fits. The view
            // transform is panel state, so this raises a request the panel's
            // `paint` consumes (it alone knows the time area's pixel width).
            KeyCode::KeyF if over_timeline => ph2d_panel_timeline::request_fit(),
            KeyCode::Home | KeyCode::KeyF => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    // Wave 2.5 PR 11.8d: bus migration (was
                    // `hero.pending_view_focus = Some(...)`).
                    hero.bus
                        .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                            kind: ph2d_editor_core::ViewFocusKind::Selected,
                        });
                } else {
                    // No hero panel — fall back to legacy "reset
                    // camera" so the non-editor demo mode still has
                    // a way to recover from a bad pan/zoom.
                    gfx.camera = Camera2d::default();
                    gfx.toasts
                        .push(Toast::info(tr("shell.handlers_teclas_editor.camera_reset")));
                }
                self.title_dirty = true;
            }
            // M14.4b: toggle grid visibility. The context-menu entry
            // promises "Show Grid · G" — this is the shortcut. Affects
            // only the hero's grid_visible flag; grid_view publishing
            // by the host continues regardless.
            // `!cmd_chord`: a CHORD must never land on a plain-letter shortcut. `Ctrl+G`
            // is Group (doc 57), and this arm was eating it for every panel in the app,
            // not just the graph — the guard fixes the class, not the symptom.
            KeyCode::KeyG if !cmd_chord => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    hero.view.grid_visible = !hero.view.grid_visible;
                    let msg = if hero.view.grid_visible {
                        tr("shell.handlers_teclas_editor.grid_on")
                    } else {
                        tr("shell.handlers_teclas_editor.grid_off")
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
                // No playback mode on the Containers LIST (Enio, 2026-07-22). The
                // bridge would pause it right back next frame anyway — refusing
                // here keeps the toast from announcing a play that never happens.
                if self.timeline.containers_list {
                    gfx.toasts.push(Toast::info(tr(
                        "shell.handlers_teclas_editor.timeline_no_playback",
                    )));
                } else {
                    let playing = self.playhead.toggle_play();
                    gfx.toasts.push(Toast::info(if playing {
                        tr("shell.handlers_teclas_editor.timeline_play")
                    } else {
                        tr("shell.handlers_teclas_editor.timeline_pause")
                    }));
                }
            }
            KeyCode::Comma | KeyCode::Period => {
                let back = code == KeyCode::Comma;
                // Com a tool Flip ativa o passo é UM QUADRO DO OBJETO (12/24 fps),
                // não um tick de simulação (60 Hz): senão "avançar um quadro" andaria
                // um quinto de desenho e o animador nunca cairia numa chave.
                let fps = self
                    .flip_state
                    .active
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
            KeyCode::ArrowUp | KeyCode::ArrowDown if self.flip_state.active => {
                self.flip_step_drawing(code == KeyCode::ArrowDown);
            }
            _ => {}
        }
    }

    /// A timeline (`L`), a chave (`K`), o clipboard e o redo do editor de áudio, o undo global, o commit do pincel
    /// e a imagem nova — e, se nenhum casar, a metade seguinte.
    pub(super) fn ramo_teclas_editor_paineis_e_desfazer(
        &mut self,
        code: KeyCode,
        cmd_chord: bool,
        over_timeline: bool,
    ) {
        #[cfg(feature = "panel-audio-editor")]
        let audio_clipboard = self.audio_editor_owns_clipboard();
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        match code {
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
                    tr("shell.handlers_teclas_editor.timeline_shown_l")
                } else {
                    tr("shell.handlers_teclas_editor.timeline_hidden_l")
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
}
