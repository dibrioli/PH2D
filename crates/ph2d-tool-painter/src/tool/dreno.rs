//! **O dreno da tela** (B10c): o estado que o dreno muda e a composição depois do portão — o MESMO
//! caminho do editor (`take_preview_arc`), da cena de uma macro e da porta genérica `current_preview`.

use super::*;

impl PainterTool {
    /// **O sujo em várias caixas** (B17.5): o mesmo [`Self::mark_dirty`] da união, e as caixas ficam ao
    /// lado só quando nada estava sujo antes (senão o sujo de antes não teria caixa que o cubra).
    pub(crate) fn mark_dirty_caixas(&mut self, uniao: Region, caixas: Vec<Region>) {
        let limpo = self.dirty_rect.is_none();
        self.mark_dirty(uniao);
        if limpo && caixas.len() > 1 {
            self.dirty_caixas = caixas;
        }
    }

    /// Toma o sujo e as caixas dele; as caixas só valem com o retângulo (vazia = ele sozinho).
    pub(crate) fn toma_o_sujo(&mut self) -> (Option<Region>, Vec<Region>) {
        let caixas = std::mem::take(&mut self.dirty_caixas);
        let r = self.dirty_rect.take();
        let caixas = if r.is_some() { caixas } else { Vec::new() };
        debug_assert!(
            caixas.is_empty() || caixas.iter().copied().reduce(paint::region::union_region) == r
        );
        (r, caixas)
    }

    /// **O envio que um dreno publica** (B20.7): enquanto ninguém o tomou
    /// ([`Self::take_preview_upload_bbox`]), o novo SOMA-SE por união ao anterior — um dreno cujo sujo
    /// ninguém consumiu não apaga o que a textura ainda não recebeu. `None` (a tela inteira) absorve.
    pub(crate) fn publica_o_envio(&mut self, bbox: Option<Region>, caixas: Vec<Region>) {
        if !std::mem::replace(&mut self.preview_upload_por_tomar, true) {
            self.preview_upload_bbox = bbox;
            self.preview_upload_caixas = caixas;
            return;
        }
        let (Some(a), Some(b)) = (self.preview_upload_bbox, bbox) else {
            self.preview_upload_bbox = None;
            self.preview_upload_caixas.clear();
            return;
        };
        let mut todas = std::mem::take(&mut self.preview_upload_caixas);
        if todas.is_empty() {
            todas.push(a);
        }
        if caixas.is_empty() {
            todas.push(b);
        } else {
            todas.extend(caixas);
        }
        self.preview_upload_bbox = Some(paint::region::union_region(a, b));
        self.preview_upload_caixas = crate::undo_delta::confine::funde_as_caixas(todas);
    }

    /// **A tela composta para a CENA** (o palco de uma macro, ADR-0186): a MESMA composição que o editor
    /// mostra, UMA vez, sobre tudo o que os passos `Preview` drenaram desde a última leitura
    /// ([`Self::drena_sem_compor`] deixou o retângulo sujo a acumular: a composição é parcial quando a
    /// anterior está em cache). Não muda o estado da pintura.
    pub(crate) fn composto_da_cena(&mut self) -> Option<(Arc<Vec<u8>>, u32, u32)> {
        if self.canvas_rgba.is_empty() {
            return None;
        }
        Some(self.compoe_a_saida(false))
    }

    /// **O passo `Preview` de uma macro tocada** — o dreno do [`Self::take_preview_arc`] sem a
    /// composição: o que o dreno muda no estado (o papel, a composição adiada do Composite Brush, o
    /// vidro) corre igual e na mesma ordem; o retângulo sujo fica a acumular para a cena compor uma vez
    /// ([`Self::composto_da_cena`]). `true` = a tela mudou.
    pub(crate) fn drena_sem_compor(&mut self) -> bool {
        if !self.drena_o_estado() {
            return false;
        }
        let compoe = self.mask_grayscale_view_source().is_none()
            && !(self.is_trivial_stack() && !self.mask_scratch_active() && !self.impasto_visible());
        if compoe {
            self.segue_o_vidro_no_quadro();
        }
        true
    }

    /// **O estado que o dreno muda**, o MESMO no editor e na macro tocada; `true` = a tela mudou (o
    /// portão do `preview_dirty`, que se drena sempre).
    fn drena_o_estado(&mut self) -> bool {
        // ⚠️ **ANTES do portão do `preview_dirty`, e a ordem é a metade que o artista vê.** O dente do
        // papel é canvas-inteiro, e nenhum dos nove knobs que o produzem levanta o `preview_dirty`
        // sozinho — reconciliar depois do portão deixaria a tela parada até a pincelada seguinte, que é
        // literalmente o report *"o papel não é atualizado em tempo real"*.
        self.reconcile_substrate();
        // ⭐ A composição do Composite Brush adiada neste quadro corre AQUI, antes de a tela ser
        // lida (`paint::composite_por_quadro`) — a mesma forma do acerto do papel acima.
        self.compoe_o_pendente();
        std::mem::take(&mut self.preview_dirty) && !self.canvas_rgba.is_empty()
    }

    pub(super) fn take_preview_arc_corpo(&mut self) -> Option<(Arc<Vec<u8>>, u32, u32)> {
        if !self.drena_o_estado() {
            self.last_drain_branch = crate::tool::DrainBranch::Idle;
            return None;
        }
        Some(self.compoe_a_saida(true))
    }

    /// Uma caixa do braço parcial: compõe, acende, tinge e cola no cache (o `composited` está quente).
    fn compoe_a_caixa(&mut self, active: RtLayerId, bbox: Region) {
        let (w, h) = self.source_size;
        let mut region = {
            let src = ToolPixelSource {
                active_id: active,
                active_rgba: &self.canvas_rgba,
                images: &self.images,
            };
            self.composto_pelo_vidro(&src, bbox).unwrap_or_else(|| {
                let mut c = composite_region(&self.layers, &src, w, h, bbox);
                self.compoe_sobre_o_papel(&mut c, bbox);
                c
            })
        };
        // Impasto: light the FRESHLY-composited region (never the cache — lighting is not idempotent).
        // The normal reads across the region's edge into the full height field.
        self.apply_impasto_light(&mut region, bbox);
        // Mask overlay: re-tint ONLY this region (the partial twin of `apply_mask_overlay`).
        self.apply_mask_overlay_region(&mut region, bbox);
        let cache = Arc::make_mut(self.composited.as_mut().expect("checked is_some"));
        blit_region(cache, w, &region, bbox);
    }

    /// A composição do dreno, depois do portão. `segue_o_vidro` = o dreno do editor (o vidro segue o
    /// traço aberto); a cena de uma macro já o fez em cada `Preview` ([`Self::drena_sem_compor`]).
    pub(super) fn compoe_a_saida(&mut self, segue_o_vidro: bool) -> (Arc<Vec<u8>>, u32, u32) {
        // Past the guard we are returning a fresh preview, so the CONTENT changed since the last drain:
        // bump the version the shell keys its upload on. This is the ONE place a Some is produced, so a
        // single bump here covers the trivial, composite and mask-view returns below. (See the field doc
        // on `preview_version`: the version replaces the shell's old `Arc::as_ptr` compare, which forced a
        // whole-canvas copy per move.)
        self.preview_version += 1;
        // Esta pista vai CONSUMIR agora o `dirty_rect` que a pista GPU também usaria para saber
        // o que mudou. A GPU não vê este frame, então tudo o que sai daqui é uma edição que o cache
        // dela (fatias por-camada no compositor + as texturas de plano do impasto) não conhece:
        // marque-o, para a próxima drenagem dela recusar confinamento e refazer inteiro. Espelho do
        // `composited = None` que a pista GPU faz na direção oposta (`take_preview_dirty`).
        self.gpu_lane_stale = true;
        // A mask row's grayscale-VIEW eye is open → show that mask's grayscale ([`super::vista_cinza`]).
        if let Some(cinza) = self.drena_a_vista_cinza() {
            return cinza;
        }
        self.composited_cinza = None;
        let (w, h) = self.source_size;
        // The trivial stack (single visible opaque Normal raster) stays the
        // zero-copy fast path. Any non-trivial stack (≥2 layers, or a layer with
        // opacity<1 / non-Normal blend / hidden / an adjustment) MUST be
        // composited here so per-layer opacity / blend / adjustments are visible.
        // Impasto forces the composite path: this lane hands back the RAW `canvas_rgba` Arc, which the
        // light pass must never write into (those are the artist's PIXELS, not a preview buffer) — and a
        // single-layer document is the common case, so without this guard the most ordinary way to use
        // Impasto would show no relief at all.
        if self.is_trivial_stack() && !self.mask_scratch_active() && !self.impasto_visible() {
            // Carry the accumulated dirty bbox into the bridge so a stroke uploads
            // only the touched sub-rect (B.1 partial lane), NOT the whole canvas
            // each frame. `None` here forced a full clone + premul + full texture
            // upload per painted frame, O(W×H) regardless of the 10px dab — the
            // 300→150 fps stroke regression. The first drain (source-push, no paint
            // yet) has no `dirty_rect` → stays `None` → the bridge seeds the GPU
            // texture with one full upload; paint frames then patch just the dab
            // footprint. If the texture isn't seeded yet the bridge's guard falls
            // back to a full upload anyway, so an early bbox can never desync it.
            let (bbox, caixas) = self.toma_o_sujo();
            self.publica_o_envio(bbox, caixas);
            self.last_drain_branch = crate::tool::DrainBranch::TrivialFast;
            return (Arc::clone(&self.canvas_rgba), w, h);
        }
        let active = self.layers.active().unwrap_or(RtLayerId(0));
        // Dirty-rect fast lane: when a valid full composite is cached AND only a
        // known bbox changed, recomposite ONLY that region and blit it into the
        // cache — O(N×bbox) vs O(N×W×H). Otherwise do a full recompose.
        // O VIDRO segue o traço aberto antes de o quadro o compor sobre o papel de cor (doc 48).
        if segue_o_vidro {
            self.segue_o_vidro_no_quadro();
        }
        let (dirty, caixas) = self.toma_o_sujo();
        let stroke_dirtied = dirty.is_some();
        if self.reacende_pela_luz_guardada(dirty) {
            return (
                Arc::clone(self.composited.as_ref().expect("just set")),
                w,
                h,
            );
        }
        // A live Mask scratch does NOT force the full path. Its protection-overlay tint is PER-PIXEL
        // (`apply_mask_overlay`: a texel's tint depends only on its own coverage + colour, no global
        // term), and a mask dab changes coverage only inside its `dirty_rect` (the stamp edits the
        // scratch swapped into `canvas_rgba` — `stamp_dabs_mask`). The genuinely global overlay changes
        // (colour swatch, canvas-op, first scratch) all `invalidate_composite()` → the full arm below.
        // So the partial arm re-tints only the dab region (`apply_mask_overlay_region`), byte-identical
        // to a full re-tint there. Forcing the full path here was a whole-canvas recompose + full 16 MiB
        // upload per mask-painted frame (measured: 17 ms preview + 6.9 ms upload @ 2048² — the mask FPS
        // drop Enio reported), when only a dab-sized rect changed.
        match (self.composited.is_some(), dirty) {
            (true, Some(bbox)) => {
                // B17.5: cada caixa compõe-se à parte (fora delas a figura não mudou); sem caixas, a união.
                if caixas.is_empty() {
                    self.compoe_a_caixa(active, bbox);
                }
                for &caixa in &caixas {
                    self.compoe_a_caixa(active, caixa);
                }
                self.compositor_cache.invalidate_from(active, &self.layers);
                self.adjustment_cache_pending = false;
                // The COMPOSITE stays partial (the 17 ms → ~2 ms win). But the mask overlay is a
                // TRANSLUCENT tint, and the shell's PARTIAL GPU upload of a sub-rect leaves visible
                // seams for it on the real device (Enio smoke 2026-07-24) — even though the CPU cache
                // is byte-identical to a full recompose (proven headless; a real-wgpu-only artifact,
                // mechanism not yet isolated). So force a FULL upload while a scratch is live: at 2048²
                // it is ~6 ms, well under the 16.7 ms frame budget (measured: dispatch ~8 ms, 60 fps).
                // Follow-up (doc 25 §13): isolate the partial-upload device seam so the fast path is
                // clean too (matters at 4096², where a full upload would overrun the budget).
                if self.mask_scratch_active() {
                    self.publica_o_envio(None, Vec::new());
                } else {
                    self.publica_o_envio(Some(bbox), caixas);
                }
                self.last_drain_branch = crate::tool::DrainBranch::PartialComposite;
            }
            _ => {
                // Mask brush: the active layer composites NORMALLY (fully visible) — the protection scratch
                // never hides it; only `apply_mask_overlay` below tints the frozen region.
                let src = ToolPixelSource {
                    active_id: active,
                    active_rgba: &self.canvas_rgba,
                    images: &self.images,
                };
                let full = Region { x: 0, y: 0, w, h };
                let mut composed = if let Some(v) = self.composto_pelo_vidro(&src, full) {
                    // O vidro sob papel de cor: uma passada só (o cache dos cortes não guarda a
                    // transparência por canal).
                    self.adjustment_cache_pending = false;
                    self.compositor_cache.invalidate_from(active, &self.layers);
                    self.larga_a_luz_guardada();
                    v
                } else {
                    let mut c =
                        if std::mem::take(&mut self.adjustment_cache_pending) && !stroke_dirtied {
                            // Adjustment slider-drag: restart from the cut-point cache —
                            // bit-identical to a full `composite` (gate
                            // `cache_matches_full_recompose`). B19.12: into the buffer `ajuste_mudou`
                            // recycled (no canvas-sized alloc nor page faults per frame).
                            let mut c = self.compositor_cache.toma_a_saida();
                            composite_with_cache_into(
                                &self.layers,
                                &src,
                                w,
                                h,
                                &mut self.compositor_cache,
                                &mut c,
                            );
                            c
                        } else {
                            self.compositor_cache.invalidate_from(active, &self.layers);
                            // B19.12: a saída e o acumulador reusados (nada de 166 MB novos por recomposição).
                            let mut c = self.compositor_cache.toma_a_saida();
                            let cache = &mut self.compositor_cache;
                            crate::compositor::composite_cheia_into(
                                &self.layers,
                                &src,
                                (w, h),
                                cache,
                                &mut c,
                            );
                            c
                        };
                    self.compoe_sobre_o_papel(&mut c, full);
                    self.guarda_o_nao_aceso(&c);
                    c
                };
                // Impasto: light the whole freshly-composited canvas (see the dirty-rect lane above).
                self.apply_impasto_light(&mut composed, Region { x: 0, y: 0, w, h });
                // Mask overlay: tint the composite by the active mask's coverage (no-op otherwise).
                self.apply_mask_overlay(&mut composed);
                self.composited = Some(Arc::new(composed));
                self.publica_o_envio(None, Vec::new());
                self.last_drain_branch = crate::tool::DrainBranch::FullComposite;
            }
        }
        (
            Arc::clone(self.composited.as_ref().expect("just set")),
            w,
            h,
        )
    }
}
