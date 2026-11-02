//! **Os diálogos de filtro do Pixel P8** (Edit ▸ Invert, Edit ▸ FX ▸ Despeckle/Outline) no mesmo
//! `ContextMenuKind::PixelEditDialog` da P6: as escolhas (Preview, canais, Tiled, Outline) guardam-se
//! na sessão; [`WidgetStore::pixel_filter_peek`] dá o pedido ACTUAL sem fechar (a família
//! pré-visualiza-o), o OK do `pixel_edit_ops` arma-o.

use super::super::WidgetStore;
use crate::ids;
use crate::pixel_panel::{
    PixelDialogChoice, PixelDialogKind, PixelOutlineBg, PixelOutlineMatrix, PixelOutlinePlace,
};
use ph2d_a11y::NodeId;

impl WidgetStore {
    /// Os campos numéricos do filtro `kind` sobre um sprite `size`: `(id, valor, mín, máx)`. A
    /// janela da mediana vai de 1 ao lado maior do sprite (maior que a imagem não lê nada novo).
    pub(super) fn pixel_filter_fields(
        &self,
        kind: PixelDialogKind,
        size: (u32, u32),
    ) -> Vec<(NodeId, f64, f64, f64)> {
        let (w, h) = self.pixel.edit.filter.median;
        let side = f64::from(size.0.max(size.1).max(1));
        match kind {
            PixelDialogKind::Despeckle => vec![
                (
                    ids::PIXEL_FILTER_MEDIAN_W,
                    f64::from(w).min(side),
                    1.0,
                    side,
                ),
                (
                    ids::PIXEL_FILTER_MEDIAN_H,
                    f64::from(h).min(side),
                    1.0,
                    side,
                ),
            ],
            _ => Vec::new(),
        }
    }

    /// Um campo do filtro mudou: prende-o à faixa (inteiro). `true` se é um campo do filtro.
    pub(super) fn pixel_filter_value_changed(&mut self, id: NodeId) -> bool {
        if id != ids::PIXEL_FILTER_MEDIAN_W && id != ids::PIXEL_FILTER_MEDIAN_H {
            return false;
        }
        let (sw, sh) = self.pixel.edit.size;
        let side = sw.max(sh).max(1);
        let v = self.median_side(id, side);
        self.set_number_value(id, f64::from(v));
        true
    }

    fn median_side(&self, id: NodeId, side: u32) -> u32 {
        let v = self.number_value(id).unwrap_or(1.0);
        let v = if v.is_finite() { v.round() } else { 1.0 };
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "preso a 1..=lado antes da conversão"
        )]
        let r = v.clamp(1.0, f64::from(side)) as u32;
        r
    }

    /// O pedido do filtro aberto com os valores de AGORA, sem fechar (`None` fora de um filtro).
    #[must_use]
    pub fn pixel_filter_peek(&self) -> Option<PixelDialogChoice> {
        let d = self.pixel.edit;
        let f = d.filter;
        let channels = f.channels;
        Some(match d.kind {
            PixelDialogKind::Invert => PixelDialogChoice::Invert { channels },
            PixelDialogKind::Despeckle => {
                let side = d.size.0.max(d.size.1).max(1);
                PixelDialogChoice::Despeckle {
                    width: self.median_side(ids::PIXEL_FILTER_MEDIAN_W, side),
                    height: self.median_side(ids::PIXEL_FILTER_MEDIAN_H, side),
                    tiled: f.tiled,
                    channels,
                }
            }
            PixelDialogKind::Outline => PixelDialogChoice::Outline {
                place: f.place,
                matrix: f.matrix,
                background: f.background,
                tiled: f.tiled,
                channels,
            },
            _ => return None,
        })
    }

    /// O filtro aberto com Preview ligado: o pedido a pré-visualizar no canvas.
    #[must_use]
    pub fn pixel_filter_preview(&self) -> Option<PixelDialogChoice> {
        let open = self
            .context_menu
            .as_ref()
            .is_some_and(|r| r.kind == crate::interaction::ContextMenuKind::PixelEditDialog);
        (open && self.pixel.edit.filter.preview)
            .then(|| self.pixel_filter_peek())
            .flatten()
    }

    /// Lembra a janela da mediana para a próxima vez (no OK).
    pub(super) fn remember_pixel_filter(&mut self, c: PixelDialogChoice) {
        if let PixelDialogChoice::Despeckle { width, height, .. } = c {
            self.pixel.edit.filter.median = (width, height);
        }
    }

    pub fn toggle_pixel_filter_preview(&mut self) {
        let f = &mut self.pixel.edit.filter;
        f.preview = !f.preview;
    }

    pub fn toggle_pixel_filter_tiled(&mut self) {
        let f = &mut self.pixel.edit.filter;
        f.tiled = !f.tiled;
    }

    /// Liga/desliga o canal `k` (`0..4` = R G B A) do *Target*.
    pub fn toggle_pixel_filter_channel(&mut self, k: usize) {
        if let Some(c) = self.pixel.edit.filter.channels.get_mut(k) {
            *c = !*c;
        }
    }

    pub fn set_pixel_filter_place(&mut self, p: PixelOutlinePlace) {
        self.pixel.edit.filter.place = p;
    }

    pub fn set_pixel_filter_matrix(&mut self, m: PixelOutlineMatrix) {
        self.pixel.edit.filter.matrix = m;
    }

    pub fn set_pixel_filter_background(&mut self, b: PixelOutlineBg) {
        self.pixel.edit.filter.background = b;
    }
}
