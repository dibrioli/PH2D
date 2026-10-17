//! **O instantâneo de um tool** — a cópia de que o tocador de uma macro recomeça uma etapa editada
//! (`docs/Macros/02_plano_W5.md` §3, W5c). Os pixels das camadas são `Arc` e copiam-se por partilha (a
//! primeira escrita de quem continua paga UMA cópia da camada, `Arc::make_mut`); o resto é dado e
//! copia-se inteiro — a água viva também. O padrão sem `..` é a lei: um campo novo do [`PainterTool`] não
//! compila aqui até alguém decidir se ele viaja.

use super::PainterTool;

/// Porque um tool não se copia.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemInstantaneo {
    /// A gravar uma macro: a gravação é deste tool, não de uma cópia.
    AGravar,
    /// A água corre num worker vivo (o editor): o motor não se alcança sem o trazer para casa.
    MotorNoWorker,
    /// Com a ponte de um dispositivo: a cópia carimbaria pela CPU, e o original não.
    PonteDoDispositivo,
}

impl PainterTool {
    /// **Uma cópia que não partilha nada vivo com este tool** (a fita da macro incluída).
    ///
    /// # Errors
    /// [`SemInstantaneo`].
    pub fn instantaneo(&self) -> Result<PainterTool, SemInstantaneo> {
        let PainterTool {
            params,
            canvas_rgba,
            wash,
            mancha_na_aguada,
            device_stamp,
            lut_cache,
            layers,
            images,
            heights,
            covers,
            mats,
            vidros,
            vidro_congelado,
            #[cfg(test)]
            sem_vidro,
            papel,
            gate,
            mask_scratch_gen,
            composited,
            preview_upload_bbox,
            last_drain_branch,
            layers_revision,
            layer_pixel_versions,
            pixel_clock,
            source_size,
            preview_dirty,
            preview_version,
            pending_commit,
            edited_since_bind,
            deferred_bake,
            undo,
            dock_shows_layers,
            undo_writes,
            dirty_rect,
            #[cfg(test)]
            marks,
            preview_dirty_region,
            gpu_lane_stale,
            lit_substrate,
            selection,
            compositor_cache,
            adjustment_cache_pending,
            paint,
            bound_doc,
            doc_cache,
            pending_layer,
            shape_source_doc,
            shape_source_revision,
            shape_source_preview_dirty,
            mask_view_grayscale,
            macro_gravacao,
            desde_o_ultimo_up,
        } = self;
        if macro_gravacao.is_some() {
            return Err(SemInstantaneo::AGravar);
        }
        if device_stamp.is_some() {
            return Err(SemInstantaneo::PonteDoDispositivo);
        }
        if !self.wet_copiavel() {
            return Err(SemInstantaneo::MotorNoWorker);
        }
        let mut copia = PainterTool {
            params: params.clone(),
            canvas_rgba: canvas_rgba.clone(),
            wash: *wash,
            mancha_na_aguada: mancha_na_aguada.clone(),
            device_stamp: None,
            lut_cache: lut_cache.clone(),
            layers: layers.clone(),
            images: images.clone(),
            heights: heights.clone(),
            covers: covers.clone(),
            mats: mats.clone(),
            vidros: vidros.clone(),
            vidro_congelado: vidro_congelado.clone(),
            #[cfg(test)]
            sem_vidro: *sem_vidro,
            papel: *papel,
            gate: gate.clone(),
            mask_scratch_gen: *mask_scratch_gen,
            composited: composited.clone(),
            preview_upload_bbox: *preview_upload_bbox,
            last_drain_branch: *last_drain_branch,
            layers_revision: *layers_revision,
            layer_pixel_versions: layer_pixel_versions.clone(),
            pixel_clock: *pixel_clock,
            source_size: *source_size,
            preview_dirty: *preview_dirty,
            preview_version: *preview_version,
            pending_commit: *pending_commit,
            edited_since_bind: *edited_since_bind,
            deferred_bake: *deferred_bake,
            undo: undo.clone(),
            dock_shows_layers: *dock_shows_layers,
            undo_writes: *undo_writes,
            dirty_rect: *dirty_rect,
            #[cfg(test)]
            marks: marks.clone(),
            preview_dirty_region: *preview_dirty_region,
            gpu_lane_stale: *gpu_lane_stale,
            lit_substrate: *lit_substrate,
            selection: selection.clone(),
            compositor_cache: compositor_cache.clone(),
            adjustment_cache_pending: *adjustment_cache_pending,
            paint: paint.clone(),
            bound_doc: *bound_doc,
            doc_cache: doc_cache.clone(),
            pending_layer: *pending_layer,
            shape_source_doc: *shape_source_doc,
            shape_source_revision: *shape_source_revision,
            shape_source_preview_dirty: *shape_source_preview_dirty,
            mask_view_grayscale: *mask_view_grayscale,
            macro_gravacao: None,
            desde_o_ultimo_up: *desde_o_ultimo_up,
        };
        copia.wet_fita_propria();
        Ok(copia)
    }

    /// **O que um instantâneo deste tool RETÉM** enquanto o original continua: a água copiada e a tela da
    /// camada ativa e a composição (partilhadas na cópia, desdobradas na primeira escrita do original). As
    /// outras camadas ninguém as escreve a tocar uma macro. O gate `medida_instantaneo_memoria` compara
    /// este número com o que o alocador vê.
    #[must_use]
    pub fn bytes_do_instantaneo(&self) -> usize {
        self.canvas_rgba.len()
            + self.composited.as_ref().map_or(0, |c| c.len())
            + self.wet_bytes_retidos()
    }
}
