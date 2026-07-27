//! **A LISTA dos dezenove planos canvas-shaped de um `ModelSnapshot`** — o irmão de
//! [`crate::undo_delta`], que é o motor.
//!
//! O corte entre os dois arquivos é por responsabilidade: *como um plano é guardado* (a janela, o diff,
//! o fallback) mora lá; *quais são os planos, e com que stride* mora aqui, ao lado da única coisa que
//! precisa mudar quando um vigésimo nascer.

use crate::undo_delta::{StoredImages, StoredMap, StoredPlane, Strides};
use ph2d_painter_brush::material::MaterialBytes;

/// **Os DEZENOVE planos canvas-shaped de um `ModelSnapshot`, guardados como delta.**
///
/// ⚠️ A lista é uma só, e os dois lados dela ([`Self::split`] e [`Self::side`]) andam juntos por
/// construção. Os dois modos de falha de esquecer um plano são bem diferentes, e é por isso que cada um
/// tem o seu gate:
///
/// - **esquecer no `split`** = o plano fica guardado INTEIRO nos dois endpoints. Falha *segura* — o undo
///   continua correto — e **invisível**, porque o único sintoma é memória. Quem pega é
///   `measure_undo_memory`, que mede o retido depois de 24 traços.
/// - **esquecer no `side`** = o plano volta como o do CURSOR, isto é, o undo não desfaz aquele plano.
///   Quem pega é `every_plane_of_a_snapshot_survives_the_round_trip`, que faz os dezenove diferirem.
#[derive(Clone, Debug, Default)]
pub(crate) struct PlaneDeltas {
    canvas_rgba: StoredPlane<u8>,
    images: StoredImages,
    heights: StoredMap<f32>,
    covers: StoredMap<u8>,
    mats: StoredMap<MaterialBytes>,
    mask_scratch: StoredPlane<u8>,
    selection_mask: StoredPlane<u8>,
    selection_crisp: StoredPlane<u8>,
    deform_disp: StoredPlane<[f32; 2]>,
    deform_pre: StoredPlane<u8>,
    deform_pre_h: StoredPlane<f32>,
    deform_pre_cover: StoredPlane<u8>,
    deform_pre_mats: StoredPlane<MaterialBytes>,
    sculpt_pre: StoredPlane<f32>,
    sculpt_amount: StoredPlane<f32>,
    sculpt_plane_sum: StoredPlane<f32>,
    sculpt_pre_cover: StoredPlane<u8>,
    sculpt_pre_mats: StoredPlane<MaterialBytes>,
    sculpt_pre_rgba: StoredPlane<u8>,
}

impl PlaneDeltas {
    /// Extrai os deltas dos dois endpoints e **esvazia os dois**: depois disto os `ModelSnapshot`
    /// guardados carregam só metadados, e os pixels vivem aqui.
    pub(crate) fn split(
        before: &mut crate::undo::ModelSnapshot,
        after: &mut crate::undo::ModelSnapshot,
        hint: Option<crate::compositor::Region>,
    ) -> Self {
        // O stride vem do endpoint que está sendo guardado. Um passo que muda o TAMANHO do canvas cai
        // sozinho no `Whole` (os comprimentos não batem), então basta um dos dois.
        let s = Strides::of(before.canvas_size.0);
        // A janela chega em PIXELS do canvas e cada plano a mede em ELEMENTOS (o RGBA carrega quatro por
        // pixel, os escalares um). `Strides` é quem sabe a conversão, então ela é feita aqui e uma vez.
        let cw = before.canvas_size.0;
        let win_rgba =
            hint.and_then(|r| crate::undo_delta::PlaneWindow::from_region(r, cw, s.rgba));
        let win_scalar =
            hint.and_then(|r| crate::undo_delta::PlaneWindow::from_region(r, cw, s.scalar));
        Self {
            canvas_rgba: StoredPlane::split(
                &mut before.canvas_rgba,
                &mut after.canvas_rgba,
                s.rgba,
                win_rgba,
            ),
            images: StoredImages::split(&mut before.images, &mut after.images),
            heights: StoredMap::split(
                &mut before.heights,
                &mut after.heights,
                s.scalar,
                win_scalar,
            ),
            covers: StoredMap::split(&mut before.covers, &mut after.covers, s.scalar, win_scalar),
            mats: StoredMap::split(&mut before.mats, &mut after.mats, s.scalar, win_scalar),
            mask_scratch: StoredPlane::split(
                &mut before.mask_scratch,
                &mut after.mask_scratch,
                s.rgba,
                win_rgba,
            ),
            selection_mask: StoredPlane::split(
                &mut before.selection_mask,
                &mut after.selection_mask,
                s.scalar,
                win_scalar,
            ),
            selection_crisp: StoredPlane::split(
                &mut before.selection_crisp,
                &mut after.selection_crisp,
                s.scalar,
                win_scalar,
            ),
            deform_disp: StoredPlane::split(
                &mut before.deform.disp,
                &mut after.deform.disp,
                s.scalar,
                win_scalar,
            ),
            deform_pre: StoredPlane::split(
                &mut before.deform.pre,
                &mut after.deform.pre,
                s.rgba,
                win_rgba,
            ),
            deform_pre_h: StoredPlane::split(
                &mut before.deform.pre_h,
                &mut after.deform.pre_h,
                s.scalar,
                win_scalar,
            ),
            deform_pre_cover: StoredPlane::split(
                &mut before.deform.pre_cover,
                &mut after.deform.pre_cover,
                s.scalar,
                win_scalar,
            ),
            deform_pre_mats: StoredPlane::split(
                &mut before.deform.pre_mats,
                &mut after.deform.pre_mats,
                s.scalar,
                win_scalar,
            ),
            sculpt_pre: StoredPlane::split(
                &mut before.sculpt.pre,
                &mut after.sculpt.pre,
                s.scalar,
                win_scalar,
            ),
            sculpt_amount: StoredPlane::split(
                &mut before.sculpt.amount,
                &mut after.sculpt.amount,
                s.scalar,
                win_scalar,
            ),
            sculpt_plane_sum: StoredPlane::split(
                &mut before.sculpt.plane_sum,
                &mut after.sculpt.plane_sum,
                s.scalar,
                win_scalar,
            ),
            sculpt_pre_cover: StoredPlane::split(
                &mut before.sculpt.pre_cover,
                &mut after.sculpt.pre_cover,
                s.scalar,
                win_scalar,
            ),
            sculpt_pre_mats: StoredPlane::split(
                &mut before.sculpt.pre_mats,
                &mut after.sculpt.pre_mats,
                s.scalar,
                win_scalar,
            ),
            sculpt_pre_rgba: StoredPlane::split(
                &mut before.sculpt.pre_rgba,
                &mut after.sculpt.pre_rgba,
                s.rgba,
                win_rgba,
            ),
        }
    }

    /// Reenche `out` (o endpoint guardado, esvaziado) a partir do **cursor**. `None` = a entrada não pode
    /// ser honrada com este cursor; quem chama descarta o histórico.
    pub(crate) fn side(
        &self,
        cursor: &crate::undo::ModelSnapshot,
        out: &mut crate::undo::ModelSnapshot,
        want_before: bool,
    ) -> Option<()> {
        let b = want_before;
        out.canvas_rgba = self.canvas_rgba.side(&cursor.canvas_rgba, b)?;
        out.images = self.images.side(&cursor.images, b)?;
        out.heights = self.heights.side(&cursor.heights, b)?;
        out.covers = self.covers.side(&cursor.covers, b)?;
        out.mats = self.mats.side(&cursor.mats, b)?;
        out.mask_scratch = self.mask_scratch.side(&cursor.mask_scratch, b)?;
        out.selection_mask = self.selection_mask.side(&cursor.selection_mask, b)?;
        out.selection_crisp = self.selection_crisp.side(&cursor.selection_crisp, b)?;
        out.deform.disp = self.deform_disp.side(&cursor.deform.disp, b)?;
        out.deform.pre = self.deform_pre.side(&cursor.deform.pre, b)?;
        out.deform.pre_h = self.deform_pre_h.side(&cursor.deform.pre_h, b)?;
        out.deform.pre_cover = self.deform_pre_cover.side(&cursor.deform.pre_cover, b)?;
        out.deform.pre_mats = self.deform_pre_mats.side(&cursor.deform.pre_mats, b)?;
        out.sculpt.pre = self.sculpt_pre.side(&cursor.sculpt.pre, b)?;
        out.sculpt.amount = self.sculpt_amount.side(&cursor.sculpt.amount, b)?;
        out.sculpt.plane_sum = self.sculpt_plane_sum.side(&cursor.sculpt.plane_sum, b)?;
        out.sculpt.pre_cover = self.sculpt_pre_cover.side(&cursor.sculpt.pre_cover, b)?;
        out.sculpt.pre_mats = self.sculpt_pre_mats.side(&cursor.sculpt.pre_mats, b)?;
        out.sculpt.pre_rgba = self.sculpt_pre_rgba.side(&cursor.sculpt.pre_rgba, b)?;
        Some(())
    }

    /// O que esta entrada retém — o número que o cap em BYTES conta.
    pub(crate) fn heap_bytes(&self) -> usize {
        self.canvas_rgba.heap_bytes()
            + self.images.heap_bytes()
            + self.heights.heap_bytes()
            + self.covers.heap_bytes()
            + self.mats.heap_bytes()
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
}
