//! **As portas estruturais sem perda de dados** (missão 4K, B01): a troca da activa é UMA porta (fecha a
//! forma aberta, guarda a tela, carrega a nova, acorda o Wet dela), e os mapas por camada seguem a pilha.
//! Gate: `camadas_integridade_tests`.

use super::super::*;

impl PainterTool {
    /// Uma forma aberta (Ellipse/Line/Polygon/Curve sem Apply) assa-se na camada onde foi desenhada antes de
    /// qualquer porta de camada trocar a tela — senão o Esc seguinte colava a tela da antiga na nova.
    pub(crate) fn fecha_a_forma_aberta(&mut self) {
        if self.is_editing_shape() || self.has_parked_shapes() {
            self.commit_open_shape();
        }
    }

    /// A primeira camada PINTÁVEL do topo da pilha (um grupo resolve-se pelo primeiro descendente pintável).
    pub(crate) fn primeira_pintavel(&self) -> Option<RtLayerId> {
        self.layers
            .root()
            .iter()
            .find_map(|&id| match self.layers.get(id).map(|l| &l.kind) {
                Some(LayerKind::Raster(_) | LayerKind::Texture(_)) => Some(id),
                Some(LayerKind::Group(_)) => self.first_paintable_descendant(id),
                _ => None,
            })
    }

    /// Faz de `id` a activa com a tela DELA (transparente se não tem): tira-a de `images`.
    pub(crate) fn carrega_a_tela_da_activa(&mut self, id: RtLayerId) {
        let buf = self.tira_a_tela_de(id);
        self.replace_canvas(buf);
        self.layers.set_active(id);
    }

    /// [`Self::carrega_a_tela_da_activa`] da TROCA da activa (B18.3), sem copiar a tela velha para o diário
    /// (2,5 ms a 4K): o `before` do passo (`snapshot_model`, que segura a tela) a guarda, e o commit logo a
    /// seguir zera o diário (`set_cursor`). Até lá nenhum leitor o desenrola sobre a tela nova: só um
    /// `before` ELIDIDO o faz, e o desta porta nunca o é.
    pub(crate) fn carrega_a_tela_da_troca(&mut self, id: RtLayerId) {
        let buf = self.tira_a_tela_de(id);
        self.instala_a_tela_do_restauro(buf);
        self.layers.set_active(id);
    }

    fn tira_a_tela_de(&mut self, id: RtLayerId) -> Arc<Vec<u8>> {
        let (w, h) = self.source_size;
        self.images
            .remove(&id)
            .map(own_image)
            .unwrap_or_else(|| Arc::new(vec![0u8; (w as usize) * (h as usize) * 4]))
    }

    /// Os mapas por camada perdem as chaves das camadas que já não existem.
    pub(crate) fn poda_os_mapas_por_camada(&mut self) {
        let vivas: std::collections::BTreeSet<RtLayerId> = self.layers.all_ids().collect();
        self.images.retain(|l, _| vivas.contains(l));
        self.vidros.retain(|l, _| vivas.contains(l));
        self.vidro_congelado.larga_as_sessoes(Some(&vivas));
        self.heights.retain(|l, _| vivas.contains(l));
        self.covers.retain(|l, _| vivas.contains(l));
        self.mats.retain(|l, _| vivas.contains(l));
        self.tintas_molhadas.retain(|l| vivas.contains(l));
        self.wet_poda_as_adormecidas();
    }

    /// Remove `id` (and its subtree + mask) and clean up the tool buffers. The base sprite (root bottom) is
    /// NOT removable. `false` if `id` is unknown or the base. When the active goes with the subtree, the new
    /// active is the top PAINTABLE layer (never a group) and the canvas carries ITS pixels.
    pub fn delete_layer(&mut self, id: RtLayerId) -> bool {
        if self.layers.get(id).is_none() || self.layers.root().last() == Some(&id) {
            return false;
        }
        self.fecha_a_forma_aberta();
        let alcance = self.mede_a_estrutura(id);
        let undo_before = self.abre_edicao_estrutural();
        let activa = self.layers.active();
        self.layers.remove(id); // drops subtree + mask
        let trocou = self.layers.active() != activa;
        if trocou {
            // A tela da activa apagada vai com ela; a nova activa traz a dela.
            if let Some(nova) = self.primeira_pintavel() {
                self.carrega_a_tela_da_activa(nova);
            }
        }
        self.poda_os_mapas_por_camada();
        self.prune_selection();
        self.commit_structural_edit(undo_before);
        if trocou {
            self.wet_acorda_da_camada();
        }
        self.a_estrutura_muda_so_em(alcance);
        true
    }

    /// Duplicate `id` (a raster/texture) above itself — the pixels AND the body (relief, coverage, material,
    /// glass, wet paper) go with the copy, which becomes the active edit target. `None` for another kind or
    /// at the cap.
    pub fn duplicate_layer(&mut self, id: RtLayerId) -> Option<RtLayerId> {
        if !matches!(
            self.layers.get(id)?.kind,
            LayerKind::Raster(_) | LayerKind::Texture(_)
        ) {
            return None;
        }
        self.fecha_a_forma_aberta();
        let alcance = self.mede_a_estrutura(id);
        let undo_before = self.abre_edicao_estrutural();
        self.flush_active_to_images();
        let (w, h) = self.source_size;
        let src_pixels = self
            .images
            .get(&id)
            .map(|img| Arc::clone(&img.rgba8))
            .unwrap_or_else(|| Arc::new(vec![0u8; (w as usize) * (h as usize) * 4]));
        let Some(new_id) = self.layers.duplicate(id) else {
            // Cap: a activa volta à tela (o invariante «a activa nunca está em images»).
            if let Some(a) = self.layers.active() {
                self.carrega_a_tela_da_activa(a);
            }
            return None;
        };
        self.images.remove(&new_id);
        if let Some(v) = self.vidros.get(&id).cloned() {
            self.vidros.insert(new_id, v);
        }
        if let Some(v) = self.heights.get(&id).cloned() {
            self.heights.insert(new_id, v);
        }
        if let Some(v) = self.covers.get(&id).cloned() {
            self.covers.insert(new_id, v);
        }
        if let Some(v) = self.mats.get(&id).cloned() {
            self.mats.insert(new_id, v);
        }
        if self.tintas_molhadas.contains(&id) {
            self.tintas_molhadas.insert(new_id);
        }
        // Sem a captura do diário (2,5 ms a 4K): o `undo_before` forte segura a tela e o commit logo a
        // seguir zera o diário — o mesmo contrato da troca (`carrega_a_tela_da_troca`).
        self.instala_a_tela_do_restauro(src_pixels);
        self.commit_structural_edit(undo_before);
        self.wet_acorda_da_camada();
        self.reset_selection_to(new_id);
        self.a_estrutura_muda_so_em(alcance);
        Some(new_id)
    }

    /// O tamanho do buffer de `id` onde quer que ele viva (a tela, se é a activa).
    fn tamanho_do_buffer(&self, id: RtLayerId) -> Option<usize> {
        if self.layers.active() == Some(id) {
            Some(self.canvas_rgba.len())
        } else {
            self.images.get(&id).map(|i| i.rgba8.len())
        }
    }

    /// **Apply mask (§2.7) — destructive bake.** Multiply each parent texel's straight alpha by the mask
    /// coverage (Rec.601 luma, `1 - v` when inverted — EXACTLY the live compositor's `mask_value`), then
    /// remove the mask. The parent becomes the active edit target iff the mask (or nothing) was active.
    /// `false` if `mask_id` is not a mask, its parent is gone, or a buffer is missing/short — decided
    /// BEFORE anything moves.
    pub fn apply_mask(&mut self, mask_id: RtLayerId) -> bool {
        let inverted = match self.layers.get(mask_id).map(|l| &l.kind) {
            Some(LayerKind::Mask(m)) => m.inverted,
            _ => return false,
        };
        let Some(parent) = self
            .layers
            .all_ids()
            .find(|&p| self.layers.get(p).and_then(|l| l.mask) == Some(mask_id))
        else {
            return false;
        };
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let curto = |t: &Self, id| t.tamanho_do_buffer(id).is_none_or(|len| len < n * 4);
        if curto(self, mask_id) || curto(self, parent) {
            return false;
        }
        self.fecha_a_forma_aberta();
        let undo_before = self.abre_edicao_estrutural();
        self.flush_active_to_images();
        let Some(mask_px) = self.images.get(&mask_id).map(|img| img.rgba8.clone()) else {
            return false;
        };
        let Some(parent_img) = self.images.get_mut(&parent).map(Arc::make_mut) else {
            return false;
        };
        let parent_px = Arc::make_mut(&mut parent_img.rgba8);
        mascara_no_alfa(
            &mask_px[..n * 4],
            &mut parent_px[..n * 4],
            w as usize,
            inverted,
        );
        let was_active = self.layers.active();
        self.layers.remove(mask_id);
        self.images.remove(&mask_id);
        self.bump_layer_pixels(Some(parent));
        let new_active = match was_active {
            Some(a) if a != mask_id => a,
            _ => parent,
        };
        self.carrega_a_tela_da_activa(new_active);
        self.commit_structural_edit(undo_before);
        self.wet_acorda_da_camada();
        self.reset_selection_to(new_active);
        self.invalidate_composite();
        true
    }
}

/// O alfa de `parent_px` multiplicado pela cobertura de `mask_px` (linhas de `w` texels), em faixas de
/// linhas: cada texel só lê o seu, então o bit é o mesmo a qualquer nº de fios. Gate: `aplica_mascara_tests`.
pub(crate) fn mascara_no_alfa(mask_px: &[u8], parent_px: &mut [u8], w: usize, inverted: bool) {
    let row_bytes = w * 4;
    let rows = parent_px.len() / row_bytes.max(1);
    let fios = crate::compositor::parallel_threads(parent_px.len() / 4, rows);
    crate::compositor::em_faixas_de_linhas(parent_px, row_bytes, fios, |faixa, y0| {
        let base = y0 * w;
        for i in 0..faixa.len() / 4 {
            let v = crate::compositor::mask_value(mask_px, base + i);
            let cov = if inverted { 1.0 - v } else { v };
            let a = faixa[i * 4 + 3] as f32;
            faixa[i * 4 + 3] = (a * cov).round().clamp(0.0, 255.0) as u8;
        }
    });
}
