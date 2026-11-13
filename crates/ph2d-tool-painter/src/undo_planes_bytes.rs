//! O que uma [`PlaneDeltas`](super::PlaneDeltas) retém e o que clonar aloca — filho de `undo_planes`, cortado
//! pelo tecto de 700 linhas.

impl super::PlaneDeltas {
    /// O que esta entrada retém — o número que o cap em BYTES conta.
    pub(crate) fn heap_bytes(&self) -> usize {
        self.canvas_rgba.heap_bytes()
            + self.images.heap_bytes()
            + self.heights.heap_bytes()
            + self.covers.heap_bytes()
            + self.mats.heap_bytes()
            + self.vidros.heap_bytes()
            + self.mask_scratch.heap_bytes()
            + self.selection_mask.heap_bytes()
            + self.selection_crisp.heap_bytes()
            + self.deform_disp.heap_bytes()
            + self.deform_pre.heap_bytes()
            + self.deform_pre_h.heap_bytes()
            + self.deform_pre_cover.heap_bytes()
            + self.deform_pre_mats.heap_bytes()
            + self.sculpt_pre.heap_bytes()
            + self.sculpt_amount.heap_bytes()
            + self.sculpt_plane_sum.heap_bytes()
            + self.sculpt_pre_cover.heap_bytes()
            + self.sculpt_pre_mats.heap_bytes()
            + self.sculpt_pre_rgba.heap_bytes()
    }

    /// O que CLONAR esta entrada aloca (as janelas; os planos inteiros são `Arc` partilhados) — o que um
    /// instantâneo do tocador declara ([`Self::heap_bytes`] é o teto do cap, que conta o pior caso).
    pub(crate) fn copiado_bytes(&self) -> usize {
        self.canvas_rgba.copiado_bytes()
            + self.images.copiado_bytes()
            + self.heights.copiado_bytes()
            + self.covers.copiado_bytes()
            + self.mats.copiado_bytes()
            + self.vidros.copiado_bytes()
            + self.mask_scratch.copiado_bytes()
            + self.selection_mask.copiado_bytes()
            + self.selection_crisp.copiado_bytes()
            + self.deform_disp.copiado_bytes()
            + self.deform_pre.copiado_bytes()
            + self.deform_pre_h.copiado_bytes()
            + self.deform_pre_cover.copiado_bytes()
            + self.deform_pre_mats.copiado_bytes()
            + self.sculpt_pre.copiado_bytes()
            + self.sculpt_amount.copiado_bytes()
            + self.sculpt_plane_sum.copiado_bytes()
            + self.sculpt_pre_cover.copiado_bytes()
            + self.sculpt_pre_mats.copiado_bytes()
            + self.sculpt_pre_rgba.copiado_bytes()
    }
}
