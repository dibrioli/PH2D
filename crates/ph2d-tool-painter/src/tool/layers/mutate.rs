//! Per-property + structural MUTATORS — layer metadata setters (visibility /
//! opacity / blend / clipping / alpha-lock / reference), structural edits
//! (add / delete / duplicate / group / mask create + apply / reparent /
//! reorder), the active-target swap, and the multi-selection ops. Adjustment-
//! layer PARAM mutators live in the sibling `adjustments` module.
//! `impl PainterTool` (one of several blocks in this crate). Split out of the
//! former `tool/layers.rs` god-file (pure move).

use super::super::*;
use super::passo::{arrasto, prop};
use crate::layers::ReliefComposite;

impl PainterTool {
    /// Set a layer's visibility (layers panel edit). No-op if `id` unknown.
    ///
    /// Unlike the STRUCTURAL edits (`add_raster_layer`/`select_layer`/
    /// `move_layer_*`), the three metadata setters below are intentionally NOT
    /// guarded by `!stroke_active`: they only touch `Layer` flags (no buffer
    /// swap, no `canvas_rgba`/`images`/undo reshuffle), so they cannot corrupt
    /// an in-flight stroke. `invalidate_composite` just drops the cache, which a
    /// mid-stroke edit recovers from on the next drain.
    pub fn set_layer_visible(&mut self, id: RtLayerId, visible: bool) {
        self.passo_de_camada(None, |t| {
            t.layers.set_visible(id, visible);
            t.invalidate_composite();
        })
    }

    /// **Renomeia uma camada.** No-op se `id` for desconhecido.
    ///
    /// ⚠️ **Faltava, e a ausência lia-se como decisão.** Dá para trocar a visibilidade, a opacidade,
    /// o modo de mistura e a profundidade do impasto — mas não o nome, que é o único destes que o
    /// artista lê a toda a hora. A fusão-em-camadas (plano `docs/Sprite_projeto/18` W10) precisou
    /// dele: uma pilha de «Layer 1 / Layer 2 / Layer 3» não diz qual sprite é qual, que é
    /// precisamente o que aquele verbo existe para mostrar.
    ///
    /// Mesma família dos três acima e pela mesma razão não é guardado por `!stroke_active`: mexe
    /// num campo do [`Layer`], nunca em buffers.
    pub fn set_layer_name(&mut self, id: RtLayerId, name: impl Into<String>) {
        self.macro_recusa("set_layer_name");
        self.passo_de_camada(arrasto(id, prop::NOME, 0), |t| {
            if let Some(layer) = t.layers.get_mut(id) {
                layer.name = name.into();
            }
            t.bump_layers_revision();
        })
    }

    /// Set a layer's opacity (0..=1). No-op if `id` unknown.
    pub fn set_layer_opacity(&mut self, id: RtLayerId, opacity: f32) {
        self.passo_de_camada(arrasto(id, prop::OPACIDADE, 0), |t| {
            t.layers.set_opacity(id, opacity);
            t.invalidate_composite();
        })
    }

    /// Set a layer's blend mode. No-op if `id` unknown.
    pub fn set_layer_blend_mode(&mut self, id: RtLayerId, mode: BlendMode) {
        self.passo_de_camada(None, |t| {
            t.layers.set_blend_mode(id, mode);
            t.invalidate_composite();
        })
    }

    /// Set a layer's **Impasto depth** (`-1..1`) — the live, permanent handle on everything ever
    /// sculpted on it (`Layer::impasto_depth`). No-op if `id` unknown.
    ///
    /// It composites, it does not re-sculpt: the height plane is untouched and `0` is a mute, not an
    /// erase. That is what lets it be the ONE knob that still reaches a stroke laid an hour ago — the
    /// brush's own Depth is baked into each stroke as it lands and afterwards only the last one is
    /// still live.
    pub fn set_layer_impasto_depth(&mut self, id: RtLayerId, depth: f32) {
        self.passo_de_camada(arrasto(id, prop::IMPASTO, 0), |t| {
            t.layers.set_impasto_depth(id, depth);
            t.invalidate_composite();
        })
    }

    /// [`Self::set_layer_impasto_depth`] from a **bare slider's** `0..1` track — the panel's per-row
    /// sliders (opacity, and now this) store normalised and the tool maps to the domain, exactly like
    /// `set_brush_size_norm`. `0.5` is the zero: the two halves of the track mean opposite things.
    pub fn set_layer_impasto_depth_norm(&mut self, id: RtLayerId, norm: f32) {
        self.passo_de_camada(arrasto(id, prop::IMPASTO, 0), |t| {
            t.layers.set_impasto_depth_norm(id, norm);
            t.invalidate_composite();
        })
    }

    /// Flip a layer's relief mode (`Add` ⇄ `Level`) — [`LayerStack::toggle_impasto_composite`].
    pub fn toggle_layer_impasto_composite(&mut self, id: RtLayerId) {
        self.passo_de_camada(None, |t| {
            t.layers.toggle_impasto_composite(id);
            t.invalidate_composite();
        })
    }

    /// Set how a layer's relief meets the relief below it (`Add` / `Level`). No-op if `id` unknown.
    pub fn set_layer_impasto_composite(&mut self, id: RtLayerId, mode: ReliefComposite) {
        self.passo_de_camada(None, |t| {
            t.layers.set_impasto_composite(id, mode);
            t.invalidate_composite();
        })
    }

    /// Toggle a layer's clipping-mask modifier (§2.8) — non-destructive (the
    /// painting is preserved; only the composite changes). No-op if `id` unknown.
    pub fn set_layer_clipping(&mut self, id: RtLayerId, clipping: bool) {
        self.passo_de_camada(None, |t| {
            t.layers.set_clipping(id, clipping);
            t.invalidate_composite();
        })
    }

    /// Toggle a layer's alpha-lock modifier (§2.10). Future paint into this
    /// layer is then clamped to its existing alpha. No-op if `id` unknown.
    pub fn set_layer_alpha_locked(&mut self, id: RtLayerId, locked: bool) {
        self.passo_de_camada(None, |t| {
            t.layers.set_alpha_locked(id, locked);
            t.invalidate_composite();
        })
    }

    /// Toggle a layer's reference modifier (§2.9) — exclusive (setting one
    /// clears the others). No-op if `id` unknown. Full ColorDrop behavior is W7;
    /// this is the non-destructive flag + UI badge.
    pub fn set_layer_reference(&mut self, id: RtLayerId, is_reference: bool) {
        self.passo_de_camada(None, |t| {
            t.layers.set_reference(id, is_reference);
            t.invalidate_composite();
        })
    }

    /// Add an empty group at the top of the stack (§2.1). No-op (`None`)
    /// mid-stroke or at the hard cap.
    pub fn add_group(&mut self) -> Option<RtLayerId> {
        self.passo_de_camada(None, |t| {
            let id = t.layers.add_group("Group")?;
            t.invalidate_composite();
            Some(id)
        })
    }

    /// Wrap the ACTIVE layer in a new group and return the group id. Interim
    /// behavior until multi-select lands (group-the-selection): for now a single
    /// active layer is nested so the "Group" button does something visible. The
    /// active layer stays the edit target (its `canvas_rgba` is untouched). No-op
    /// (`None`) mid-stroke, with no active layer, on the base sprite, or at cap.
    pub fn group_active(&mut self) -> Option<RtLayerId> {
        self.passo_de_camada(None, |t| {
            let active = t.layers.active()?;
            // The base sprite is pinned at root bottom — don't nest it.
            if t.layers.root().last() == Some(&active) {
                return None;
            }
            let g = t.layers.add_group("Group")?;
            if !t.layers.move_into_group(active, g) {
                t.layers.remove(g); // couldn't nest (depth/cycle) — drop the empty group
                return None;
            }
            t.layers.set_active(active); // keep painting the layer, not the group
            t.selection.clear();
            t.selection.insert(g);
            t.selection.insert(active);
            t.invalidate_composite();
            Some(g)
        })
    }

    /// Apply a drag-drop reparent/reorder emitted by the dispatch
    /// (`WidgetEvent::PanelRowReparent` da família `PainterLayer`). Reverses the dragged + target
    /// `NodeId`s to `LayerId`s via the per-row widget id, then dispatches to
    /// `move_into_group` (drop INTO a group) / `move_to_sibling_of` (before /
    /// after) / `move_to_root_bottom_above_base` (drop at the end). The
    /// LayerStack guards (base pinned, `MAX_GROUP_DEPTH`, cycle) reject unsafe
    /// drops. No-op mid-stroke or if `dragged` doesn't resolve to a layer.
    pub fn handle_layer_reparent(
        &mut self,
        dragged: ph2d_a11y::NodeId,
        drop: ph2d_editor_core::interaction::PanelRowDrop,
    ) {
        self.macro_recusa("handle_layer_reparent");
        self.passo_de_camada(None, |t| {
            if reparent_in(&mut t.layers, dragged, drop) {
                t.invalidate_composite();
            }
        })
    }

    /// Snapshot the ACTIVE layer's working buffer (`canvas_rgba`) into
    /// `images` before a layer switch — so the layer we're leaving keeps its
    /// pixels. (The active layer is otherwise NOT stored in `images`.)
    pub(crate) fn flush_active_to_images(&mut self) {
        // A tinta viva fica com a camada dela: o `Arc` novo da tela acabaria a sessão no guarda.
        self.wet_adormece();
        if let Some(active) = self.layers.active() {
            let (w, h) = self.source_size;
            self.images.insert(
                active,
                Arc::new(LayerImage {
                    width: w,
                    height: h,
                    // B18.1: a camada que sai partilha o plano; quem escrever nele depois faz o fork.
                    rgba8: Arc::clone(&self.canvas_rgba),
                }),
            );
        }
    }

    /// Add a new transparent raster layer at the top of the stack and make
    /// it active. The previous active layer's pixels are flushed to `images`;
    /// the new active starts as a fresh transparent `canvas_rgba`. Returns the
    /// new id, or `None` mid-stroke / before a source is set / at the cap.
    pub fn add_raster_layer(&mut self, name: impl Into<String>) -> Option<RtLayerId> {
        self.add_raster_layer_com(name, None)
    }

    /// A porta das duas acima: os píxeis (ou a tela transparente) entram ANTES do commit, dentro do passo.
    /// Escrevê-los depois era uma escrita da tela inteira fora de toda janela declarada (B01.5: o passo
    /// seguinte disparava o `debug_assert` de `undo_delta_absorb`).
    fn add_raster_layer_com(
        &mut self,
        name: impl Into<String>,
        rgba: Option<Vec<u8>>,
    ) -> Option<RtLayerId> {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return None;
        }
        // Cap check BEFORE flushing (audit W3): otherwise a cap-hit add would
        // leave the just-flushed active layer stranded in `images`, breaking
        // the "active is never in images" invariant.
        if self.layers.len() >= crate::layers::HARD_CAP_LAYERS {
            return None;
        }
        self.fecha_a_forma_aberta();
        let mantem = rgba.is_none() && self.a_nova_transparente_mantem_a_figura();
        let adormece = self.layers.active().filter(|_| mantem && self.tinta_viva());
        crate::sonda_partes::parte("mais.antes");
        let undo_before = self.snapshot_model();
        crate::sonda_partes::parte("mais.snapshot_model");
        self.flush_active_to_images();
        crate::sonda_partes::parte("mais.flush");
        let id = self.layers.add_raster(name, w, h)?; // sets active = id (top)
        self.images.remove(&id); // active lives in canvas_rgba, not images
        let nasce_neutra = rgba.is_none();
        let rgba = rgba.unwrap_or_else(|| tela_transparente((w as usize) * (h as usize) * 4));
        crate::sonda_partes::parte("mais.tela_zero");
        self.replace_canvas(Arc::new(rgba));
        crate::sonda_partes::parte("mais.replace_canvas");
        self.undo.tela_nova_neutra = nasce_neutra;
        self.commit_structural_edit(undo_before);
        crate::sonda_partes::parte("mais.commit");
        self.reset_selection_to(id);
        if mantem {
            self.troca_sem_mudar_a_figura(); // B18.4: o «+» transparente não recompõe nem dobra o relevo
            let v = self.layer_pixel_versions.get(&id).copied().unwrap_or(0);
            self.camada_nova_limpa = Some((id.0, v));
            crate::sonda_partes::parte("mais.troca");
            if let Some(id) = adormece {
                self.a_tinta_que_adormeceu_muda(id);
            }
            crate::sonda_partes::parte("mais.adormeceu_muda");
        } else {
            self.invalidate_composite();
        }
        Some(id)
    }

    /// **Uma camada nova que já NASCE com pixels** — plano
    /// [`docs/Sprite_projeto/18`](../../../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md)
    /// W10, por ordem do Enio (2026-08-21): *"Merge Sprites em camadas (cria uma camada por
    /// sprite)"*.
    ///
    /// # Por que uma porta nova, e não a de cima seguida de uma escrita
    ///
    /// ⚠️ **Os pixels de uma camada não são alcançáveis de fora.** A activa vive no `canvas_rgba` e
    /// as outras num `BTreeMap` privado; o [`Self::add_raster_layer`] cria a camada **transparente**
    /// e não há API pública para lá pôr nada. Quem quisesse montar um documento de N camadas a
    /// partir de N imagens — que é exactamente o que a fusão-em-camadas faz — não tinha por onde.
    ///
    /// ⛔ **A alternativa recusada** era expor o `canvas_rgba`. Isso abriria a porta a alguém
    /// escrever nele **a meio de um traço**, e o undo desta ferramenta é por planos capturados nas
    /// fronteiras do gesto: uma escrita fora dessa janela não entra em snapshot nenhum e o Ctrl+Z
    /// devolve pixels que nunca existiram. Esta porta faz a coisa **inteira** — cria, escreve e
    /// fecha o passo estrutural — e não deixa a meio um estado que só ela saberia terminar.
    ///
    /// # Panics
    ///
    /// Nunca: um `rgba` do tamanho errado devolve `None`, como o resto desta família. Um documento
    /// de 512² com uma camada de 64² não é um erro a corrigir adiante — é a chamada errada, e vale
    /// mais recusá-la do que compor lixo.
    pub fn add_raster_layer_with_pixels(
        &mut self,
        name: impl Into<String>,
        rgba: Vec<u8>, // COLOR-RAW-OK: bytes de sprite, entregues verbatim como no `set_source`
    ) -> Option<RtLayerId> {
        self.macro_recusa("add_raster_layer_with_pixels");
        let (w, h) = self.source_size;
        // ⚠️ A conferência ANTES de mexer em coisa nenhuma: `add_raster_layer` faz um passo
        // estrutural (flush da activa, undo, selecção), e descobrir o tamanho errado depois dele
        // deixaria uma camada vazia na pilha que ninguém pediu.
        if rgba.len() != (w as usize) * (h as usize) * 4 {
            return None;
        }
        self.add_raster_layer_com(name, Some(rgba))
    }

    /// Create a grayscale mask on the ACTIVE raster layer (§2.7) and make the
    /// mask the edit target — a fresh opaque-WHITE buffer (fully visible). The
    /// brush then paints luminance into it (`effective_active_color` grays the
    /// stroke). No-op (`None`) mid-stroke, if the active layer isn't a raster,
    /// if it already has a mask, or at the hard cap.
    pub fn add_mask_to_active(&mut self) -> Option<RtLayerId> {
        self.macro_recusa("add_mask_to_active");
        let active = self.layers.active()?;
        // A raster OR a texture layer can take a mask (the compositor multiplies the layer's alpha by
        // the mask for both). A texture has no dims in its spec, so source the canvas dims here.
        if !matches!(
            self.layers.get(active)?.kind,
            LayerKind::Raster(_) | LayerKind::Texture(_)
        ) {
            return None; // not a group / mask / adjustment
        }
        self.fecha_a_forma_aberta();
        let undo_before = self.snapshot_model();
        let (w, h) = self.source_size;
        let mask = self.layers.add_mask_for(active, w, h)?;
        // The parent (still active) flushes to images; the mask becomes the active edit target with a
        // fresh opaque-WHITE buffer (full visible).
        self.flush_active_to_images();
        self.images.remove(&mask); // active lives in canvas_rgba, not images
        self.replace_canvas(Arc::new(vec![255u8; (w as usize) * (h as usize) * 4]));
        self.layers.set_active(mask);
        self.commit_structural_edit(undo_before);
        self.reset_selection_to(mask);
        self.invalidate_composite();
        Some(mask)
    }

    /// Toggle the mask's grayscale-VIEW eye: show that mask's grayscale on canvas (eye open) ↔ the masked
    /// effect (eye closed, default). View-only + transient — flips `mask_view_grayscale` (one at a time)
    /// and invalidates the composite. No-op if `mask_id` isn't a mask.
    pub fn toggle_mask_view_grayscale(&mut self, mask_id: RtLayerId) {
        if !matches!(
            self.layers.get(mask_id).map(|l| &l.kind),
            Some(LayerKind::Mask(_))
        ) {
            return;
        }
        self.mask_view_grayscale = if self.mask_view_grayscale == Some(mask_id) {
            None
        } else {
            Some(mask_id)
        };
        self.invalidate_composite();
    }

    /// Toggle a mask's `Invert` flag (§2.7). The live compositor already honors
    /// it (`1 - value`), so this just flips the flag + invalidates the
    /// composite. No-op mid-stroke or if `mask_id` is not a mask.
    pub fn toggle_mask_inverted(&mut self, mask_id: RtLayerId) {
        self.passo_de_camada(None, |t| {
            let inverted = match t.layers.get(mask_id).map(|l| &l.kind) {
                Some(LayerKind::Mask(m)) => m.inverted,
                _ => return,
            };
            t.layers.set_mask_inverted(mask_id, !inverted);
            t.invalidate_composite();
        })
    }

    /// Make `id` the active edit target: flush the current active's pixels to
    /// `images`, load `id`'s pixels into `canvas_rgba` (transparent if it has
    /// none yet), swap the mask brush color, reset undo, invalidate. No buffer
    /// work if `id` is already active. Shared by `select_layer` / the multi-
    /// select `select_single` / `select_additive` / `select_range`. Caller owns
    /// the `selection` set bookkeeping. A Group id is resolved to its first
    /// paintable descendant (a group can never be the paint target).
    pub(crate) fn set_active_layer(&mut self, id: RtLayerId) {
        // A Group has NO pixel buffer — making it the paint target would load a
        // throwaway transparent `canvas_rgba` (the composite never reads it) and
        // silently swallow strokes. Resolve a group to its first paintable
        // descendant so clicking a group "enters" it; an empty group (or unknown
        // id) leaves the active target unchanged.
        let target = match self.layers.get(id).map(|l| &l.kind) {
            Some(LayerKind::Group(_)) => match self.first_paintable_descendant(id) {
                Some(t) => t,
                None => return,
            },
            Some(_) => id,
            None => return,
        };
        if self.layers.active() == Some(target) {
            return;
        }
        // B18.3: com a figura igual antes e depois, o quadro seguinte não recompõe a pilha.
        let mantem = self.a_troca_mantem_a_figura(target);
        self.fecha_a_forma_aberta();
        let undo_before = self.snapshot_model();
        self.flush_active_to_images();
        #[cfg(test)]
        if super::cache::TROCA_RECOMPOE_SEMPRE.with(std::cell::Cell::get) {
            self.carrega_a_tela_da_activa(target); // a sonda A/B: o caminho de antes
        } else {
            self.carrega_a_tela_da_troca(target);
        }
        #[cfg(not(test))]
        self.carrega_a_tela_da_troca(target);
        self.commit_structural_edit(undo_before);
        self.wet_acorda_da_camada();
        if mantem && self.a_troca_mantem_a_figura(target) {
            self.troca_sem_mudar_a_figura();
        } else {
            self.invalidate_composite();
        }
    }

    /// Collapse the multi-selection to a single layer (selection bookkeeping
    /// only — does NOT touch the active edit target). Called by the structural
    /// ops (add / duplicate) that already set the active layer themselves, so a
    /// stale multi-selection does not linger as a phantom highlight.
    pub(crate) fn reset_selection_to(&mut self, id: RtLayerId) {
        self.selection.clear();
        self.selection.insert(id);
    }

    /// Drop any selection members that no longer exist (after a delete/remove).
    pub(crate) fn prune_selection(&mut self) {
        self.selection.retain(|id| self.layers.get(*id).is_some());
    }

    /// Make `id` the active layer and collapse the multi-selection to it (a
    /// plain row click). No-op mid-stroke or unknown id. (Kept under the
    /// historic name; callers that just want a single active layer still work.)
    pub fn select_layer(&mut self, id: RtLayerId) {
        self.select_single(id);
    }

    /// Plain row click — replace the selection with `id` and make it active.
    /// No-op mid-stroke or for an unknown id.
    pub fn select_single(&mut self, id: RtLayerId) {
        if self.layers.get(id).is_none() {
            return;
        }
        self.set_active_layer(id);
        self.selection.clear();
        self.selection.insert(id);
    }

    /// Cmd/Ctrl-click — toggle `id` in the selection. Adding it makes it the
    /// active edit target; removing the active member repoints active to
    /// another member. Never empties the selection (a toggle-off of the lone
    /// member is ignored). No-op mid-stroke or for an unknown id.
    pub fn select_additive(&mut self, id: RtLayerId) {
        if self.layers.get(id).is_none() {
            return;
        }
        // The current active is always a selected row — fold it in so the FIRST
        // Cmd-click extends the active layer rather than replacing it.
        if let Some(a) = self.layers.active() {
            self.selection.insert(a);
        }
        if self.selection.contains(&id) {
            if self.selection.len() == 1 {
                return; // keep at least one member selected
            }
            self.selection.remove(&id);
            if self.layers.active() == Some(id)
                && let Some(&next) = self.selection.iter().next()
            {
                self.set_active_layer(next);
            }
        } else {
            self.selection.insert(id);
            self.set_active_layer(id);
        }
    }

    /// Shift-click — select the contiguous run of layers between the current
    /// active (anchor) and `id` along the visible row order, and make `id`
    /// active. Falls back to `select_single` if either endpoint is not in the
    /// visible z-order run (e.g. a mask sub-row, or no active anchor). No-op
    /// mid-stroke or for an unknown id.
    pub fn select_range(&mut self, id: RtLayerId) {
        if self.layers.get(id).is_none() {
            return;
        }
        let order = self.visible_row_order();
        let bi = order.iter().position(|&x| x == id);
        let ai = self
            .layers
            .active()
            .and_then(|a| order.iter().position(|&x| x == a));
        match (ai, bi) {
            (Some(ai), Some(bi)) => {
                let (lo, hi) = if ai <= bi { (ai, bi) } else { (bi, ai) };
                self.selection.clear();
                self.selection.extend(order[lo..=hi].iter().copied());
                self.set_active_layer(id);
            }
            _ => self.select_single(id),
        }
    }

    /// Wrap ALL selected layers in a new group (multi-select Group). Creates a
    /// group, moves every selected non-base layer into it (in visible z-order,
    /// skipping the base sprite and anything the depth/cycle guards reject), and
    /// keeps the current active layer as the edit target (it is reparented, not
    /// removed, so it stays valid + paintable). The selection collapses to the
    /// new group. Falls back to `group_active` (the interim single-layer wrap)
    /// when fewer than two layers are selected. No-op (`None`) mid-stroke or if
    /// the group could not be created / nothing could be nested.
    pub fn group_selected(&mut self) -> Option<RtLayerId> {
        // Collect selected layers in stable visible order; the base sprite
        // (pinned at root bottom) is never groupable.
        let base = self.layers.root().last().copied();
        let targets: Vec<RtLayerId> = self
            .visible_row_order()
            .into_iter()
            .filter(|id| self.selection.contains(id) && Some(*id) != base)
            .collect();
        if targets.len() < 2 {
            return self.group_active();
        }
        self.passo_de_camada(None, |t| {
            let g = t.layers.add_group("Group")?;
            let mut moved_any = false;
            for &id in &targets {
                if t.layers.move_into_group(id, g) {
                    moved_any = true;
                }
            }
            if !moved_any {
                t.layers.remove(g); // could not nest anything — drop the empty group
                return None;
            }
            // Active is unchanged by grouping (its layer was reparented, not moved
            // out of focus). Collapse the selection to the new group for a clear cue.
            t.selection.clear();
            t.selection.insert(g);
            if let Some(a) = t.layers.active() {
                t.selection.insert(a);
            }
            t.invalidate_composite();
            Some(g)
        })
    }

    /// Move a layer one step toward the FRONT (top of z-order) — layers panel
    /// ↑ reorder button. No-op mid-stroke (structural-edit lifecycle, mirror of
    /// `select_layer`) or at the top. Invalidates the composite.
    pub fn move_layer_up(&mut self, id: RtLayerId) {
        self.passo_de_camada(None, |t| {
            t.layers.move_up(id);
            t.invalidate_composite();
        })
    }

    /// Move a layer one step toward the BACK (bottom of z-order) — layers panel
    /// ↓ reorder button. No-op mid-stroke or at the bottom. Invalidates the
    /// composite.
    pub fn move_layer_down(&mut self, id: RtLayerId) {
        self.passo_de_camada(None, |t| {
            t.layers.move_down(id);
            t.invalidate_composite();
        })
    }
}

/// ⭐ **A lei do arrasto de uma linha sobre uma pilha**. `true` = a pilha mudou.
pub(crate) fn reparent_in(
    stack: &mut LayerStack,
    dragged: ph2d_a11y::NodeId,
    drop: ph2d_editor_core::interaction::PanelRowDrop,
) -> bool {
    use ph2d_editor_core::interaction::PanelRowDrop;
    let find =
        |s: &LayerStack, id| super::super::layer_edit::layer_widget_in(s, id).map(|(l, _)| l);
    let Some(d) = find(stack, dragged) else {
        return false;
    };
    // Masks are owner-attached (not in the z-order) — never reparent one
    // (the row is selectable, but dragging it must not touch the stack).
    if matches!(stack.get(d).map(|l| &l.kind), Some(LayerKind::Mask(_))) {
        return false;
    }
    match drop {
        // Middle band: drop INTO a group folder. If the target isn't a
        // group (or the nest is rejected — depth cap / cycle),
        // `move_into_group` returns `false` WITHOUT mutating (its guards
        // precede the detach), so we fall back to a sibling insert ABOVE
        // the target. This kills the dead 40% middle band on normal layer
        // rows: every position over a row now resolves to a meaningful
        // move, and the panel's drop indicator mirrors it (box for a
        // group, before-line for a leaf). Photoshop/Procreate semantics:
        // you only nest into a folder; over a leaf it's always before/after.
        PanelRowDrop::Inside(t) => find(stack, t).is_some_and(|tgt| {
            stack.move_into_group(d, tgt) || stack.move_to_sibling_of(d, tgt, false)
        }),
        PanelRowDrop::Before(t) => {
            find(stack, t).is_some_and(|tgt| stack.move_to_sibling_of(d, tgt, false))
        }
        PanelRowDrop::After(t) => {
            find(stack, t).is_some_and(|tgt| stack.move_to_sibling_of(d, tgt, true))
        }
        PanelRowDrop::End => stack.move_to_root_bottom_above_base(d),
    }
}

thread_local! {
    /// **A chave do A×B** (n.º 7 item 3, a tela do «+»): `true` = [`tela_transparente`] pede ao alocador acima do
    /// tecto do limiar dinâmico do mmap; `false` = o `vec![0u8; n]` de antes. Os bytes são os mesmos.
    pub(crate) static TELA_NOVA_PELO_MMAP: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// Acima disto o `calloc` do glibc vai SEMPRE ao `mmap` (o limiar dinâmico pára em `DEFAULT_MMAP_THRESHOLD_MAX` =
/// 32 MiB em 64 bits); páginas de `mmap` nascem zeradas e só pesam quando escritas.
const PISO_DO_MMAP: usize = 32 * 1024 * 1024 + 4096;

/// Abaixo disto o `memset` do heap custa menos que um `mmap`/`munmap`.
const TELA_MINIMA_PELO_MMAP: usize = 1024 * 1024;

/// **A tela transparente do «+»**: `n` zeros sem tocar uma página. Um plano de 4K tem 31,6 MiB, abaixo do tecto de
/// 32 MiB do limiar dinâmico do glibc: depois de libertar um plano destes, o limiar sobe acima dele e o
/// `vec![0u8; n]` seguinte sai do heap com `memset` (medido: 18–29 ms e 33,2 MB tocados no fio do «+»). Pedir a
/// capacidade acima do tecto leva-o sempre ao `mmap`; a folga é só endereço, nunca escrita. Os clones do plano
/// (`Arc::make_mut`) copiam o `len`, não a capacidade.
pub(crate) fn tela_transparente(n: usize) -> Vec<u8> {
    if !TELA_NOVA_PELO_MMAP.get() || !(TELA_MINIMA_PELO_MMAP..PISO_DO_MMAP).contains(&n) {
        return vec![0u8; n];
    }
    // `vec![0; _]` vai pelo `alloc_zeroed` (o `calloc`); o `truncate` guarda a capacidade, então o bloco é libertado
    // com o mesmo tamanho com que nasceu.
    let mut v = vec![0u8; PISO_DO_MMAP];
    v.truncate(n);
    v
}

#[cfg(test)]
mod tela_transparente_tests {
    use super::{PISO_DO_MMAP, TELA_NOVA_PELO_MMAP, tela_transparente};

    /// As faltas de página menores deste fio × 4 KiB.
    fn faltas_do_fio() -> u64 {
        let s = std::fs::read_to_string("/proc/thread-self/stat").unwrap_or_default();
        let resto = s.rsplit_once(") ").map_or("", |(_, r)| r);
        resto
            .split(' ')
            .nth(7)
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
            * 4096
    }

    /// **A tela do «+» a 4K não toca páginas e é só zeros** (n.º 7 item 3). O alocador é aquecido como no app: três
    /// planos de 4K alocados e libertados sobem o limiar dinâmico acima do plano. Controlo, no mesmo fio: a
    /// chave desligada dá os mesmos bytes; o tamanho abaixo do mínimo fica no `vec!` (capacidade exacta).
    #[test]
    fn a_tela_nova_de_4k_nasce_sem_tocar_paginas() {
        let n = 3840 * 2160 * 4;
        for _ in 0..3 {
            let v = std::hint::black_box(vec![7u8; n]);
            drop(v);
        }
        let f0 = faltas_do_fio();
        let t = tela_transparente(n);
        let tocado = faltas_do_fio() - f0;
        assert!(t.capacity() >= PISO_DO_MMAP, "capacidade {}", t.capacity());
        assert!(tocado < 1 << 20, "a tela nova tocou {tocado} bytes");
        assert_eq!(t.len(), n);
        TELA_NOVA_PELO_MMAP.set(false);
        let antes = tela_transparente(n);
        TELA_NOVA_PELO_MMAP.set(true);
        assert_eq!(antes.capacity(), n);
        assert!(t == antes, "os bytes da tela nova são os de antes");
        assert!(t.iter().all(|&b| b == 0));
        assert_eq!(tela_transparente(64 * 64 * 4).capacity(), 64 * 64 * 4);
    }
}
