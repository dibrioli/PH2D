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
    images: BTreeMap<RtLayerId, Arc<LayerImage>>,
    /// **Impasto**: the document's relief and paint coverage, per layer. They travel with the document
    /// for the same reason `images` does — and they MUST, for a sharper one: they are keyed by
    /// [`RtLayerId`], and `LayerStack::new()` restarts `next_id` at 1, so two documents' ids collide by
    /// construction. Left behind, the outgoing sprite's relief would light the incoming one's paint
    /// (`restore_doc` never re-sourced, so nothing cleared them) — the exact shape of Bug #13.c, which
    /// this line exists to keep dead.
    heights: BTreeMap<RtLayerId, Arc<Vec<f32>>>,
    covers: BTreeMap<RtLayerId, Arc<Vec<u8>>>,
    /// O MATERIAL por camada. Viaja com o documento pela MESMA razão que o relevo (Bug #13.c: os ids
    /// de camada colidem entre documentos, então o que fica pra trás sombreia a tinta do próximo).
    mats: BTreeMap<RtLayerId, Arc<Vec<[u8; 4]>>>,
    layer_pixel_versions: BTreeMap<RtLayerId, u64>,
    source_size: (u32, u32),
    undo: crate::undo::UndoController,
    selection: BTreeSet<RtLayerId>,
}

impl StashedDoc {
    /// The canvas size of a stashed document (the shell needs it before restoring).
    pub(crate) fn size(&self) -> (u32, u32) {
        self.source_size
    }

    /// Freeze this document for the disk — the structure, the pixels and the relief, and nothing else.
    /// The undo history and the layer-row selection stay behind on purpose: a session's history is not
    /// a property of the artwork, and restoring one from a file would be restoring the *ghost* of a
    /// session that no longer exists.
    pub(crate) fn to_painted(&self, id: u32) -> crate::tool::persist::PaintedDocument {
        crate::tool::persist::PaintedDocument {
            id,
            layers: self.layers.clone(),
            canvas_rgba: self.canvas_rgba.as_ref().clone(),
            images: self
                .images
                .iter()
                .map(|(k, v)| (*k, v.as_ref().clone()))
                .collect(),
            // The disk holds plain vectors — the `Arc` is a RUNTIME sharing device (it makes an undo
            // snapshot a refcount bump instead of an 80 MB copy at 4096²), not a wire format. Same
            // boundary the `canvas_rgba` above already crosses.
            heights: unshare(&self.heights),
            covers: unshare(&self.covers),
            mats: unshare(&self.mats),
            size: self.source_size,
        }
    }

    /// The inverse: a document off the disk becomes a stashed one, as if the artist had just switched
    /// away from it — so `bind_document` restores it through the ONE path that already exists.
    pub(crate) fn from_painted(doc: crate::tool::persist::PaintedDocument) -> Self {
        Self {
            canvas_rgba: crate::tool::persist::arc_pixels(doc.canvas_rgba),
            layers: doc.layers,
            images: doc
                .images
                .into_iter()
                .map(|(k, v)| (k, Arc::new(v)))
                .collect(),
            heights: reshare(doc.heights),
            covers: reshare(doc.covers),
            mats: reshare(doc.mats),
            // Rebuilt on demand: a version cache and a fresh history. (`bump_layer_pixels` re-stamps
            // the versions the first time the compositor asks.)
            layer_pixel_versions: BTreeMap::new(),
            source_size: doc.size,
            undo: crate::undo::UndoController::default(),
            selection: BTreeSet::new(),
        }
    }
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
            // Same sprite re-pushed: only re-seed when there is no work to lose, so an external
            // image-tool edit still updates the canvas without flattening our own layers.
            if self.doc_is_disposable() {
                self.set_source(rgba, width, height);
            }
            return;
        }
        // Stash the OUTGOING document (keep its work) before we replace it.
        if let Some(old) = self.bound_doc
            && !self.doc_is_disposable()
        {
            let stashed = self.stash_current_doc();
            self.doc_cache.insert(old, stashed);
            // If the outgoing doc IS the Shape source, it now needs a live preview rendered (it's no
            // longer the active sprite, so the bridge must drive its composite into a preview slot).
            if Some(old) == self.shape_source_doc {
                self.shape_source_preview_dirty = true;
            }
        }
        self.bound_doc = Some(entity);
        match self.doc_cache.remove(&entity) {
            Some(doc) => self.restore_doc(doc), // bring back its cached multi-layer stack
            None => self.set_source(rgba, width, height), // fresh (or the sprite's flat texture)
        }
    }

    /// Whether the working document can simply be THROWN AWAY on a rebind, because the sprite's own
    /// pixels reconstruct it: a single plain raster layer AND no impasto relief.
    ///
    /// The relief is the reason this is not just [`Self::is_trivial_stack`]. A one-layer document that
    /// has been sculpted is *not* reconstructable from the sprite: the height field is a channel of its
    /// own, and the sprite is only RGBA. Treating it as disposable silently threw the sculpting away on
    /// the next sprite switch — the pixels came back (baked, with the light in them) and the relief did
    /// not, so the artist could no longer edit the thickness of paint they were looking at.
    fn doc_is_disposable(&self) -> bool {
        self.is_trivial_stack() && self.heights.is_empty()
    }

    /// Move the current working document out into a [`StashedDoc`] (leaving the painter's live fields
    /// empty — the caller restores or re-sources immediately after).
    fn stash_current_doc(&mut self) -> StashedDoc {
        self.drop_live_relief(); // the open stroke's ground belongs to the document being put away
        StashedDoc {
            canvas_rgba: std::mem::take(&mut self.canvas_rgba),
            layers: std::mem::take(&mut self.layers),
            images: std::mem::take(&mut self.images),
            heights: std::mem::take(&mut self.heights),
            covers: std::mem::take(&mut self.covers),
            mats: std::mem::take(&mut self.mats),
            layer_pixel_versions: std::mem::take(&mut self.layer_pixel_versions),
            source_size: self.source_size,
            undo: std::mem::take(&mut self.undo),
            selection: std::mem::take(&mut self.selection),
        }
    }

    /// Restore a stashed document into the live fields + reset the transient composite caches/flags so the
    /// next preview recomposes the restored stack.
    fn restore_doc(&mut self, doc: StashedDoc) {
        // Restoring a stashed sprite is a rebind too — abandon any in-progress edit tied to the outgoing
        // canvas (open shape, pending Fill, Mask scratch, …) before swapping. See `paint::lifecycle`.
        self.reset_transient_edit_state();
        self.canvas_rgba = doc.canvas_rgba;
        self.layers = doc.layers;
        self.images = doc.images;
        // The relief comes back WITH its document. Note this is a REPLACE, not a merge: together with
        // the `mem::take` in `stash_current_doc` it is defence in depth against the outgoing sprite's
        // maps lingering (their layer ids collide with the incoming document's by construction — Bug
        // #13.c's shape). Either one alone already blocks it; the gate goes red only when BOTH fall,
        // which is exactly what the mutation run showed, so neither line is decoration.
        self.heights = doc.heights;
        self.mats = doc.mats;
        self.covers = doc.covers;
        self.drop_live_relief();
        // The stack and the relief travelled together, so the `has_relief` flags arrive already true —
        // EXCEPT for a document that came off disk from a build that never sculpted it. Re-deriving is
        // O(layers) and makes the flag self-healing at every door into a document: it can never be the
        // thing that lies to the panel.
        self.sync_relief_flags();
        self.layer_pixel_versions = doc.layer_pixel_versions;
        self.source_size = doc.source_size;
        self.resolve_symmetry_geometry(); // re-pin the auto-centre symmetry pivot to the restored canvas
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

    /// Set layer `id`'s **opacity** in the SHAPE SOURCE document `src` — the brush's per-layer opacity box
    /// is a remote control of that layer. If `src` is the bound (live) doc, edit it directly + re-composite
    /// the visible sprite; else (the source sprite is stashed because the artist switched away to paint a
    /// DIFFERENT sprite with this shape) edit its STASHED stack, so the change persists and shows when the
    /// artist re-selects that sprite. Never touches the painted document's layers (the bug). No-op if `src`
    /// isn't loaded.
    pub(crate) fn set_shape_source_layer_opacity(
        &mut self,
        src: Option<u64>,
        id: RtLayerId,
        v: f32,
    ) {
        if src == self.bound_doc {
            self.set_layer_opacity(id, v);
        } else if let Some(d) = src
            && let Some(stashed) = self.doc_cache.get_mut(&d)
        {
            stashed.layers.set_opacity(id, v);
            self.shape_source_preview_dirty = true; // re-render its live preview (it's not selected)
        }
    }

    /// Set layer `id`'s **blend mode** in the SHAPE SOURCE document `src` — the blend dropdown's remote
    /// control. Same live-vs-stashed routing as [`Self::set_shape_source_layer_opacity`].
    pub(crate) fn set_shape_source_layer_blend(
        &mut self,
        src: Option<u64>,
        id: RtLayerId,
        mode: BlendMode,
    ) {
        if src == self.bound_doc {
            self.set_layer_blend_mode(id, mode);
        } else if let Some(d) = src
            && let Some(stashed) = self.doc_cache.get_mut(&d)
        {
            stashed.layers.set_blend_mode(id, mode);
            self.shape_source_preview_dirty = true; // re-render its live preview (it's not selected)
        }
    }

    /// The STASHED Shape source sprite id whose live preview the bridge should drive — `Some` only when a
    /// multi-layer Shape is captured from a sprite that is NOT the one currently being painted (so it sits
    /// in `doc_cache`). `None` when the source IS the active doc (the normal active preview covers it) or
    /// there is no captured Shape. The bridge suppresses this sprite + samples its composited preview.
    pub fn shape_source_preview_sprite(&self) -> Option<u64> {
        let src = self.shape_source_doc?;
        if Some(src) == self.bound_doc || self.brush_shape_layers().0 == 0 {
            return None;
        }
        self.doc_cache.contains_key(&src).then_some(src)
    }

    /// Composite the STASHED Shape source document into premultiplied-ish RGBA `(bytes, w, h)` — but ONLY
    /// when its preview is dirty (it was just stashed, or a brush opacity/blend remote-control edited it).
    /// Clears the dirty flag. `None` when not dirty / no stashed source / it has zero size. The bridge
    /// uploads the result into a per-sprite preview slot so the non-selected sprite reflects the edit live.
    pub fn take_shape_source_preview(&mut self) -> Option<(Vec<u8>, u32, u32)> {
        if !self.shape_source_preview_dirty {
            return None;
        }
        let src = self.shape_source_preview_sprite()?;
        let stashed = self.doc_cache.get(&src)?;
        let (w, h) = stashed.source_size;
        if w == 0 || h == 0 {
            return None;
        }
        self.shape_source_preview_dirty = false;
        let active = stashed.layers.active().unwrap_or(RtLayerId(0));
        let src_px = ToolPixelSource {
            active_id: active,
            active_rgba: stashed.canvas_rgba.as_slice(),
            images: &stashed.images,
        };
        Some((
            crate::compositor::composite(&stashed.layers, &src_px, w, h),
            w,
            h,
        ))
    }
}

/// Runtime `Arc` maps -> plain vectors, for the disk (see [`StashedDoc::to_painted`]).
fn unshare<T: Clone>(m: &BTreeMap<RtLayerId, Arc<Vec<T>>>) -> BTreeMap<RtLayerId, Vec<T>> {
    m.iter().map(|(k, v)| (*k, v.as_ref().clone())).collect()
}

/// The inverse of [`unshare`]: plain vectors off the disk become shareable again.
fn reshare<T>(m: BTreeMap<RtLayerId, Vec<T>>) -> BTreeMap<RtLayerId, Arc<Vec<T>>> {
    m.into_iter().map(|(k, v)| (k, Arc::new(v))).collect()
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

    /// A sculpted document keeps its RELIEF across sprite switches — and, the sharper half, does not
    /// LEND it to the sprite the artist switches to.
    ///
    /// The relief is keyed by [`RtLayerId`], and `LayerStack::new()` restarts `next_id` at 1, so two
    /// documents' ids collide by construction. Before this, `stash_current_doc` did not take the height
    /// maps and `restore_doc` did not replace them: switching to a stashed sprite left the OUTGOING
    /// sprite's relief in place, and it lit the incoming one's paint through the colliding ids. Same
    /// species as Bug #13.c (a cache that outlives the document that produced it).
    ///
    /// And a one-layer sculpted document is NOT disposable: the sprite is only RGBA, so its pixels
    /// cannot reconstruct a height channel. Treating it as trivial threw the sculpting away.
    #[test]
    fn relief_travels_with_its_document_and_is_never_lent_to_another() {
        // Sculpt through the PUBLIC setters + the real pointer path — the same seam the artist drives.
        let paint_a_ridge = |t: &mut PainterTool| {
            use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
            t.set_brush_size_px(6.0);
            t.toggle_brush_impasto();
            t.set_brush_impasto_depth(0.8);
            let cp = |p: [f32; 2], phase| CanvasPointer {
                pos: p,
                pressure: 1.0,
                tilt: [0.0, 0.0],
                phase,
            };
            t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
            t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
        };

        let mut t = PainterTool::default();
        // Sprite 1: ONE layer, sculpted. (Deliberately trivial — that is the case the old predicate
        // called "disposable".)
        t.bind_document(1, vec![255u8; 32 * 32 * 4], 32, 32);
        paint_a_ridge(&mut t);
        assert!(!t.heights.is_empty(), "sprite 1 carries relief");
        let relief_1: Vec<f32> = t
            .heights
            .values()
            .next()
            .expect("a height map")
            .as_ref()
            .clone();
        assert!(relief_1.iter().any(|&h| h.abs() > 0.1), "and it is real");

        // Switch to sprite 2, a fresh canvas the artist has never sculpted.
        t.bind_document(2, vec![255u8; 32 * 32 * 4], 32, 32);
        assert!(
            t.heights.is_empty(),
            "sprite 2 must inherit NO relief — the ids collide, so a leftover map would light its paint"
        );

        // …and back to sprite 1: its relief is exactly what it was.
        t.bind_document(1, vec![0u8; 4], 1, 1);
        let back: Vec<f32> = t
            .heights
            .values()
            .next()
            .expect("sprite 1's relief came back with it")
            .as_ref()
            .clone();
        assert_eq!(back, relief_1, "the sculpting survived the round trip");

        // The LENDING path proper: switching to a sprite that is itself CACHED goes through
        // `restore_doc`, which never re-sources — so nothing there would clear a leftover map. Give
        // sprite 2 a second layer so it gets stashed, then bounce 2 → 1 → 2 and check that sprite 1's
        // relief did not ride along. (Without the assignment in `restore_doc` this is where the ids
        // collide and the wrong sprite lights up; the round-trip assertions above sail past it.)
        t.bind_document(2, vec![255u8; 32 * 32 * 4], 32, 32);
        t.layers.add_raster("Layer 2", 32, 32); // now non-trivial ⇒ stashed on the way out
        assert!(
            t.heights.is_empty(),
            "sprite 2 still has no relief of its own"
        );
        t.bind_document(1, vec![0u8; 4], 1, 1); // 2 is stashed; 1 is restored (with its relief)
        assert!(!t.heights.is_empty(), "…and 1 still has its own");
        t.bind_document(2, vec![0u8; 4], 1, 1); // 2 comes back through `restore_doc`
        assert!(
            t.heights.is_empty(),
            "sprite 2 must come back with ITS relief (none) — not with sprite 1's, whose layer ids \
             collide with its own by construction"
        );
    }
}
