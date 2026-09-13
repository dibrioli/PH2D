//! **O dreno do barramento — a timeline, os pedidos da moldura e a barra do topo.** Braços do `match` da [`fase_bus_drain`](super), movidos pela
//! ordem de sempre; a única troca no corpo deles é `pd.<pedido>` onde escreviam `<pedido>`, e o `gfx` de cada
//! sub-dreno é re-derivado (o dreno só corre com ele). Ver o cabeçalho de lá.

use super::*;
use ph2d_editor_core::action_bus::EditorAction;

impl crate::App {
    /// O `PanelEvent` da timeline docada (docs/Timeline W2.E2): o `+Track`, o Motion Path, o cartão do onion e o
    /// transporte.
    pub(super) fn fase_bus_timeline_panel(&mut self, action: EditorAction) -> Option<EditorAction> {
        let gfx = self.gfx.as_mut()?;
        let FrameGfx { hero_screen, .. } = FrameGfx::of(gfx);
        let hero = hero_screen.as_mut()?;
        match action {
            // docs/Timeline W2.E2: the docked timeline panel is not a
            // tool — translate its transport PanelEvents into
            // `TimelineIntent`s (id → intent; the timeline semantics live
            // here, editor-core stays timeline-agnostic) and queue them
            // for `timeline_bridge::run` to apply this frame.
            EditorAction::TimelinePanelEvent(ev) => {
                // "+Track <prop>" binds the selected sprite's property
                // (the panel doesn't know the selection; the shell does).
                if let ph2d_editor_core::tool::PanelEvent::Click(id) = &ev
                    && let Some(prop) = timeline_bridge::prop_for_addprop_id(*id)
                {
                    if let Some(entity) = hero.gizmo.iter_selected().next() {
                        self.timeline_intents
                            .push(ph2d_timeline::TimelineIntent::Bind { entity, prop });
                    }
                } else if let ph2d_editor_core::tool::PanelEvent::Toggle(id, on) = &ev
                    && *id == ph2d_panel_timeline::ids::TIMELINE_MOTION_PATH
                {
                    // The Motion Path toggle is PER OBJECT (like +Track, the
                    // panel doesn't know the selection): convert THIS object's
                    // position to a trajectory (`on`) or separate X/Y — Convert
                    // to Motion Path / to Separate Axes (ADR-0141).
                    if let Some(entity) = hero.gizmo.iter_selected().next() {
                        self.timeline_intents.push(
                            ph2d_timeline::TimelineIntent::ConvertPositionMode {
                                entity,
                                to_path: *on,
                            },
                        );
                    }
                } else if let ph2d_editor_core::tool::PanelEvent::Click(id) = &ev
                    && *id == ph2d_panel_timeline::ids::TIMELINE_ONION_SETTINGS
                {
                    // Open the onion settings card (hero chrome), seeded from the current
                    // onion. Shell-side because the card lives in `hero.store`, out of the
                    // panel's reach (mirror of the Motion Path case above). `OnionSettings`
                    // is `Copy`, so this holds no borrow while `hero.store` is written; the
                    // count↔slider + rgb↔u8 mappings live in `crate::onion_modal`.
                    let o = self.timeline.onion;
                    let (ax, ay) = hero
                        .hit_index
                        .rect_for(*id)
                        .map_or((120.0, 120.0), |r| (r.x - 90.0, r.y - 236.0));
                    hero.store.open_onion_modal(
                        ax,
                        ay,
                        o.opacity,
                        crate::onion_modal::count_to_frac(o.frames_before),
                        crate::onion_modal::count_to_frac(o.frames_after),
                        crate::onion_modal::rgb_to_u8(o.color_before),
                        crate::onion_modal::rgb_to_u8(o.color_after),
                    );
                } else if let Some(intent) =
                    timeline_bridge::intent_for_transport(&ev, &self.timeline, &self.playhead)
                {
                    self.timeline_intents.push(intent);
                    // A jump to an absolute time may land outside the
                    // visible span; pan the dope sheet after it (the
                    // panel page-follows only while playing). Deferred
                    // to the apply — see `timeline_reveal_after_apply`.
                    self.timeline_reveal_after_apply |= timeline_bridge::jumps_the_playhead(&ev);
                }
            }
            other => return Some(other),
        }
        None
    }

    /// Os pedidos que esperam outro ponto do quadro: cancelar a ferramenta modal (ADR-0040 TG-C), o pill SCULPT, o
    /// Undo da imagem e o Undo/Redo da barra.
    pub(super) fn fase_bus_tool_requests(
        &mut self,
        action: EditorAction,
        pd: &mut DrainOut,
    ) -> Option<EditorAction> {
        let gfx = self.gfx.as_mut()?;
        let FrameGfx { tools, .. } = FrameGfx::of(gfx);
        match action {
            // ADR-0040 TG-B/TG-C: generic "cancel the active modal
            // tool". Switch back to the default tool and tear down
            // any image-tool shell-side preview caches. Bg Removal +
            // Padding panels both raise this; the bgremoval cleanup
            // is a no-op when padding (or any non-bgremoval tool)
            // was active. Padding's shell-side state is purely
            // tool-internal (no shell-cached preview), so no
            // padding-specific cleanup is needed here.
            EditorAction::CancelActiveTool => {
                // ADR-0108: end any in-progress Vector draw cleanly when
                // the tool is toggled off. The Pen lives on the shell, so
                // the partial path PERSISTS in `vec_scene` (open) — no
                // discard, no warning; `finish` just leaves drawing mode
                // (a cheap no-op for any other tool being cancelled).
                self.vec.pen.finish();
                if let Some(default_id) = tools.default_tool_id()
                    && tools.set_active(&default_id)
                {
                    self.last_bgremoval_pushed_entity = None;
                    self.bgremoval_preview = None;
                    self.title_dirty = true;
                }
            }
            // O pill SCULPT (ADR-0150). ⚠️ **Um pedido, drenado no topo do frame
            // SEGUINTE** — a mesma rota do `Shift+B` e do padrão do sprite, e pelo mesmo
            // motivo, que aqui é mais forte: entrar pode ter de CRIAR a cena, e o `device`
            // está emprestado neste ponto do laço.
            EditorAction::ToggleSculpt3d => self.sculpt3d_req.toggle_request = true,
            EditorAction::UndoImageEdit => pd.undo_image_edit = true,
            // Os botões Undo/Redo da barra: MESMO caminho do Ctrl+Z. O despacho
            // espera o fim do frame (`post_frame_undo`) porque `undo_or_redo`
            // precisa de `&mut self` e o `gfx` está emprestado aqui.
            EditorAction::UndoStep { redo } => self.undo_button = Some(redo),
            other => return Some(other),
        }
        None
    }

    /// A barra do topo — o filtro de imagem, o modo de apresentação e o transporte — e o `_` que cala a variante que
    /// nenhum braço toma (`EditorAction` é `#[non_exhaustive]`).
    pub(super) fn fase_bus_topbar(&mut self, action: EditorAction) -> Option<()> {
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            surface, renderer, ..
        } = FrameGfx::of(gfx);
        match action {
            EditorAction::SetImageFilter { mode } => {
                // Single global image-filter toggle. Rebuilds the
                // atlas + individual samplers and their bind groups
                // so EVERY sprite samples with the new mode; no
                // texture re-upload. The Vello BG-Removal preview
                // reads `hero.project.image_filter` directly (set by
                // the editor before this action), so both stay in
                // sync.
                renderer.set_filter_mode(mode);
            }
            EditorAction::SetPresentMode { vsync } => {
                // Config → Display toggle. VSync (Fifo) = smooth
                // hardware-paced motion; Immediate = non-blocking
                // (no mouse-stutter). Reconfigures the swap chain
                // in place. Both modes are available on this
                // backend (boot log confirms); Fifo is the
                // universal fallback.
                surface.set_present_mode(if vsync {
                    wgpu::PresentMode::Fifo
                } else {
                    wgpu::PresentMode::Immediate
                });
            }
            EditorAction::Transport(cmd) => {
                // TopBar Play/Pause/Reset drive the ONE clock
                // (`Playhead`, W4.T7). Physics, Motion, Timeline and
                // Flip all ride it, so one click moves every
                // time-based subsystem at once. The single door
                // `transport::apply` is unit-tested headless. NOTE:
                // physics scrub-back — the ball flying back up — is
                // W1.5; here Reset only returns the clock to 0.
                ph2d_transport::apply(cmd, &mut self.playhead);
            }
            // (Bgremoval bake leftover handled inside the
            // `OneShotImageOp` arm above — defers to the
            // image_edit drain site so `bgremoval_active` is
            // observed AFTER any same-frame ActivateTool fires.)
            // EditorAction is `#[non_exhaustive]`. A future
            // variant landing in `ph2d-editor` shouldn't break
            // the shell — drop it silently here until a
            // dispatch site is wired up.
            _ => {}
        }
        Some(())
    }
}
