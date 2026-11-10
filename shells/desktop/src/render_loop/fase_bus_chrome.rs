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
                    // A jump may land off-span: pan the dope sheet after the apply (`timeline_reveal_after_apply`).
                    self.timeline_reveal_after_apply |= timeline_bridge::jumps_the_playhead(&ev);
                }
            }
            other => return Some(other),
        }
        None
    }

    /// Os pedidos que esperam outro ponto do quadro: cancelar a ferramenta modal (ADR-0040 TG-C), o Undo da imagem e
    /// o Undo/Redo da barra.
    pub(super) fn fase_bus_tool_requests(
        &mut self,
        action: EditorAction,
        pd: &mut DrainOut,
    ) -> Option<EditorAction> {
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            tools, zen, toasts, ..
        } = FrameGfx::of(gfx);
        match action {
            // ADR-0040 TG-B/TG-C: back to the default tool, dropping the image-tool preview caches; the Pen's
            // partial path stays in `vec_scene` (ADR-0108: `finish` only leaves drawing mode).
            EditorAction::CancelActiveTool => {
                self.vec.pen.finish();
                if let Some(default_id) = tools.default_tool_id()
                    && tools.set_active(&default_id)
                {
                    self.bgremoval.last_pushed_entity = None;
                    self.bgremoval.preview = None;
                    self.title_dirty = true;
                }
            }
            EditorAction::UndoImageEdit => pd.undo_image_edit = true,
            // Os botões Undo/Redo da barra: MESMO caminho do Ctrl+Z. O despacho
            // espera o fim do frame (`post_frame_undo`) porque `undo_or_redo`
            // precisa de `&mut self` e o `gfx` está emprestado aqui.
            EditorAction::UndoStep { redo } => self.undo_button = Some(redo),
            // *Ver ▸ Modo foco* (F5): a mesma porta do `Ctrl+Space` e do toque de 4 dedos.
            EditorAction::FocusMode => {
                self.title_dirty |= zen.toggle_announced(toasts);
            }
            other => return Some(other),
        }
        None
    }

    /// A barra do topo — o filtro de imagem, o modo de apresentação e o transporte — e o `_` que cala a variante que
    /// nenhum braço toma (`EditorAction` é `#[non_exhaustive]`).
    pub(super) fn fase_bus_topbar(&mut self, action: EditorAction) -> Option<()> {
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            surface,
            renderer,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        match action {
            // D17 — o duplo-clique num Kanban ou num Quadro dos Assets abre a aba dele.
            EditorAction::OpenAsset(doc) => {
                hero_screen.as_mut()?.documents.open_asset(doc);
            }
            EditorAction::SetImageFilter { mode } => {
                // One global toggle: rebuilds the samplers (no re-upload); the BG-Removal
                // preview reads `hero.project.image_filter`, so both stay in sync.
                renderer.set_filter_mode(mode);
            }
            EditorAction::SetPresentMode { vsync } => {
                // VSync (Fifo, the universal fallback) = paced; off = the best mode the backend HAS.
                surface.set_present_mode(if vsync {
                    wgpu::PresentMode::Fifo
                } else {
                    surface.best_nonblocking_mode()
                });
            }
            EditorAction::PlayerControlsToggle => {
                if let Some(hero) = hero_screen.as_ref() {
                    let s = &mut self.components.jogador;
                    let r = ph2d_player_controls::host::toggle(
                        s,
                        &hero.input_map,
                        self.playhead.is_playing(),
                    );
                    Self::relogio_dos_controles(&mut self.playhead, r);
                }
            }
            EditorAction::Transport(cmd) => {
                ph2d_transport::apply(cmd, &mut self.playhead);
                // ⭐ O Play de CIMA põe a cena a CORRER (a Animação de cada objecto, em qualquer aba —
                // o Autoplay do Godot); o Reset de cima sai. O Pause só pausa (onda 4, item 1).
                if let Some(i) = ph2d_transport::runtime_intent(cmd) {
                    self.timeline_intents.push(i);
                }
            }
            // `EditorAction` is `#[non_exhaustive]`: a variant with no dispatch site yet drops here.
            _ => {}
        }
        Some(())
    }
}
