//! Per-document persistence — the painter holds ONE working document (a [`LayerStack`] + pixels), but it
//! CACHES the layer stack of each sprite it has edited, keyed by sprite id, so switching sprites does not
//! flatten the previous one. Without this, binding a new sprite (`set_source`) replaces the layer stack
//! with a fresh single raster, permanently losing the previous sprite's multi-layer structure (Enio
//! 2026-06-26: "criei um sprite multi-camada para usar como Shape; ao trocar de imagem ele é achatado").
//!
//! The stash/restore is driven from `bind_document` using the painter's OWN `bound_doc` tracking — so it
//! never depends on the shell's frame ordering (the bake-on-leave + source re-push). Only a NON-trivial
//! (multi-layer) document is cached; a trivial single-raster doc is fully reconstructable from the sprite
//! pixels, so it is just re-`set_source`d.

use super::*;

/// A stashed working document: the layer model + its pixels + the per-document history & selection, held
/// in [`PainterTool::doc_cache`] by sprite id across sprite switches. The transient composite caches are
/// NOT stashed — they are rebuilt on restore.
pub(crate) struct StashedDoc {
    canvas_rgba: Arc<Vec<u8>>,
    layers: LayerStack,
    images: BTreeMap<RtLayerId, LayerImage>,
    layer_pixel_versions: BTreeMap<RtLayerId, u64>,
    source_size: (u32, u32),
    undo: crate::undo::UndoController,
    selection: BTreeSet<RtLayerId>,
}

impl PainterTool {
    /// Bind the working canvas to sprite `entity` (its `rgba`/`width`/`height` pixels) — the painter's
    /// source-of-truth switch; the bridge calls THIS instead of the generic `set_source`. It stashes the
    /// outgoing document's layer stack (when multi-layer) by its sprite id, then RESTORES the target's
    /// stashed stack if cached, else `set_source`s a fresh single-raster doc. So a multi-layer sprite
    /// keeps its layers across switches instead of being flattened.
    pub fn bind_document(
        &mut self,
        entity: u64,
        rgba: Vec<u8>, // COLOR-RAW-OK: raw sprite source bytes forwarded verbatim to the `set_source` trait contract (also `Vec<u8>`)
        width: u32,
        height: u32,
    ) {
        if self.bound_doc == Some(entity) {
            // Same sprite re-pushed: only re-seed when there are no layers to lose (a trivial stack), so
            // an external image-tool edit still updates the canvas without flattening our own layers.
            if self.is_trivial_stack() {
                self.set_source(rgba, width, height);
            }
            return;
        }
        // Stash the OUTGOING document (keep its multi-layer work) before we replace it.
        if let Some(old) = self.bound_doc
            && !self.is_trivial_stack()
        {
            let stashed = self.stash_current_doc();
            self.doc_cache.insert(old, stashed);
        }
        self.bound_doc = Some(entity);
        match self.doc_cache.remove(&entity) {
            Some(doc) => self.restore_doc(doc), // bring back its cached multi-layer stack
            None => self.set_source(rgba, width, height), // fresh (or the sprite's flat texture)
        }
    }

    /// Move the current working document out into a [`StashedDoc`] (leaving the painter's live fields
    /// empty — the caller restores or re-sources immediately after).
    fn stash_current_doc(&mut self) -> StashedDoc {
        StashedDoc {
            canvas_rgba: std::mem::take(&mut self.canvas_rgba),
            layers: std::mem::take(&mut self.layers),
            images: std::mem::take(&mut self.images),
            layer_pixel_versions: std::mem::take(&mut self.layer_pixel_versions),
            source_size: self.source_size,
            undo: std::mem::take(&mut self.undo),
            selection: std::mem::take(&mut self.selection),
        }
    }

    /// Restore a stashed document into the live fields + reset the transient composite caches/flags so the
    /// next preview recomposes the restored stack.
    fn restore_doc(&mut self, doc: StashedDoc) {
        self.discard_open_shape();
        self.canvas_rgba = doc.canvas_rgba;
        self.layers = doc.layers;
        self.images = doc.images;
        self.layer_pixel_versions = doc.layer_pixel_versions;
        self.source_size = doc.source_size;
        self.undo = doc.undo;
        self.selection = doc.selection;
        self.composited = None;
        self.compositor_cache = CompositorCache::new();
        self.adjustment_cache_pending = false;
        self.preview_dirty = true;
        self.dirty_rect = None;
        self.preview_upload_bbox = None;
        self.edited_since_bind = false;
        self.layers_revision = self.layers_revision.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Switching sprites preserves each sprite's multi-layer stack (it is stashed by id + restored on
    /// re-bind), instead of flattening the previous document — the reported data-loss bug.
    #[test]
    fn switching_documents_preserves_multi_layer_stacks() {
        let mut t = PainterTool::default();
        // Bind sprite 1 and make it MULTI-layer (a 2nd raster → non-trivial).
        t.bind_document(1, vec![255u8; 64 * 64 * 4], 64, 64);
        t.layers.add_raster("Layer 2", 64, 64);
        assert!(!t.is_trivial_stack(), "sprite 1 is multi-layer");
        let layers_1 = t.layers.root().len();
        assert!(layers_1 >= 2);

        // Switch to a fresh sprite 2 (single layer).
        t.bind_document(2, vec![0u8; 32 * 32 * 4], 32, 32);
        assert!(t.is_trivial_stack(), "sprite 2 is a fresh single-layer doc");
        assert_eq!(t.source_size, (32, 32));

        // Switch BACK to sprite 1 — its multi-layer stack must be restored (the rgba is ignored on a hit).
        t.bind_document(1, vec![0u8; 4], 1, 1);
        assert_eq!(
            t.layers.root().len(),
            layers_1,
            "sprite 1's layers are restored, not flattened"
        );
        assert_eq!(
            t.source_size,
            (64, 64),
            "sprite 1's canvas size is restored"
        );
    }
}
