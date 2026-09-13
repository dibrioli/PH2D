//! **Os atalhos do editor, a segunda metade do `match code`** — ramos do `handle_editor_key` ([`super`]). Um
//! `match` parte-se pelo braço `_`: os braços seguintes, PELA MESMA ORDEM, formam o `match code` do ramo, e o
//! primeiro que casar é o mesmo de antes (o `match` do Rust é ordenado, guardas incluídas). Os corpos
//! MUDARAM-SE verbatim (`line/input-dispatch`, 2026-09-13), à mesma indentação.
//!
//! ⚠️ O ficheiro vive no território do despacho (`input_dispatch/`) e é FILHO do `input_handlers` (por `#[path]`):
//! assim ele vê os métodos privados de lá sem mudar a visibilidade de nenhum.

use super::*;

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
                    gfx.toasts.push(Toast::info("Camera · reset"));
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
                // No playback mode on the Containers LIST (Enio, 2026-07-22). The
                // bridge would pause it right back next frame anyway — refusing
                // here keeps the toast from announcing a play that never happens.
                if self.timeline.containers_list {
                    gfx.toasts
                        .push(Toast::info("Timeline · no playback in the Containers list"));
                } else {
                    let playing = self.playhead.toggle_play();
                    gfx.toasts.push(Toast::info(if playing {
                        "Timeline · play"
                    } else {
                        "Timeline · pause"
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
}
