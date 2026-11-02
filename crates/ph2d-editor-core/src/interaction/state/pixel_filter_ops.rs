//! **Os diálogos de filtro do Pixel P8** (Edit ▸ Invert, Edit ▸ FX ▸ Despeckle/Outline) no mesmo
//! `ContextMenuKind::PixelEditDialog` da P6: as escolhas (Preview, canais, Tiled, Outline) guardam-se
//! na sessão; [`WidgetStore::pixel_filter_peek`] dá o pedido ACTUAL sem fechar (a família
//! pré-visualiza-o), o OK do `pixel_edit_ops` arma-o.

use super::super::WidgetStore;
use crate::ids;
use crate::pixel_panel::{
    PIXEL_CURVE_MAX, PixelCurve, PixelDialogChoice, PixelDialogKind, PixelHueMode, PixelOutlineBg,
    PixelOutlineMatrix, PixelOutlinePlace,
};
use crate::widget;
use crate::zones::Rect;
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
            PixelDialogKind::BrightnessContrast => self.adjust_fields(0..2),
            PixelDialogKind::HueSaturation => self.adjust_fields(2..6),
            _ => Vec::new(),
        }
    }

    /// Os campos de Adjustments `range` com o último valor da sessão.
    fn adjust_fields(&self, range: std::ops::Range<usize>) -> Vec<(NodeId, f64, f64, f64)> {
        let last = self.pixel.edit.filter.adjust;
        ids::PIXEL_FILTER_ADJUST_FIELDS[range]
            .iter()
            .map(|&(k, id, lo, hi)| (id, f64::from(last[k]), f64::from(lo), f64::from(hi)))
            .collect()
    }

    /// O valor inteiro do campo de Adjustments `k`, preso à faixa dele.
    fn adjust_value(&self, k: usize) -> i32 {
        let (_, id, lo, hi) = ids::PIXEL_FILTER_ADJUST_FIELDS[k];
        let v = self.number_value(id).unwrap_or(0.0);
        let v = if v.is_finite() { v.round() } else { 0.0 };
        #[expect(
            clippy::cast_possible_truncation,
            reason = "preso à faixa i32 do campo"
        )]
        let r = v.clamp(f64::from(lo), f64::from(hi)) as i32;
        r
    }

    /// Um campo do filtro mudou: prende-o à faixa (inteiro). `true` se é um campo do filtro.
    pub(super) fn pixel_filter_value_changed(&mut self, id: NodeId) -> bool {
        if let Some(&(k, ..)) = ids::PIXEL_FILTER_ADJUST_FIELDS.iter().find(|f| f.1 == id) {
            let v = self.adjust_value(k);
            self.set_number_value(id, f64::from(v));
            return true;
        }
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
        let (channels, all) = (f.channels, f.all);
        Some(match d.kind {
            PixelDialogKind::Invert => PixelDialogChoice::Invert { channels, all },
            PixelDialogKind::Despeckle => {
                let side = d.size.0.max(d.size.1).max(1);
                PixelDialogChoice::Despeckle {
                    width: self.median_side(ids::PIXEL_FILTER_MEDIAN_W, side),
                    height: self.median_side(ids::PIXEL_FILTER_MEDIAN_H, side),
                    tiled: f.tiled,
                    channels,
                    all,
                }
            }
            PixelDialogKind::Outline => PixelDialogChoice::Outline {
                place: f.place,
                matrix: f.matrix,
                background: f.background,
                tiled: f.tiled,
                channels,
                all,
            },
            PixelDialogKind::BrightnessContrast => PixelDialogChoice::BrightnessContrast {
                brightness: self.adjust_value(0),
                contrast: self.adjust_value(1),
                channels,
                all,
            },
            PixelDialogKind::HueSaturation => PixelDialogChoice::HueSaturation {
                mode: f.hue_mode,
                hue: self.adjust_value(2),
                saturation: self.adjust_value(3),
                lightness: self.adjust_value(4),
                alpha: self.adjust_value(5),
                channels,
                all,
            },
            PixelDialogKind::ColorCurve => PixelDialogChoice::ColorCurve {
                curve: f.curve,
                channels,
                all,
            },
            PixelDialogKind::ConvolutionMatrix => PixelDialogChoice::Convolution {
                kernel: self.pixel_filter_kernel(),
                tiled: f.tiled,
                channels,
                all,
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
        let f = &mut self.pixel.edit.filter;
        match c {
            PixelDialogChoice::Despeckle { width, height, .. } => f.median = (width, height),
            PixelDialogChoice::BrightnessContrast {
                brightness,
                contrast,
                ..
            } => f.adjust[..2].copy_from_slice(&[brightness, contrast]),
            PixelDialogChoice::HueSaturation {
                hue,
                saturation,
                lightness,
                alpha,
                ..
            } => f.adjust[2..].copy_from_slice(&[hue, saturation, lightness, alpha]),
            _ => {}
        }
    }

    pub fn toggle_pixel_filter_preview(&mut self) {
        let f = &mut self.pixel.edit.filter;
        f.preview = !f.preview;
    }

    /// O botão do alvo: «Selected» ↔ «All».
    pub fn toggle_pixel_filter_target(&mut self) {
        let f = &mut self.pixel.edit.filter;
        f.all = !f.all;
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

    pub fn set_pixel_filter_hue_mode(&mut self, m: PixelHueMode) {
        self.pixel.edit.filter.hue_mode = m;
    }

    /// A família publica os nomes dos kernels da Convolution Matrix (na ordem do núcleo).
    pub fn set_pixel_filter_kernels(&mut self, names: &'static [&'static str]) {
        self.pixel.edit.filter.kernels = names;
    }

    /// A família publica os canais de cada kernel (paralelo aos nomes).
    pub fn set_pixel_filter_kernel_channels(
        &mut self,
        channels: &'static [crate::pixel_panel::PixelChannels],
    ) {
        self.pixel.edit.filter.kernel_channels = channels;
    }

    /// Escolhe o kernel `i` (fora da lista, nada muda); fica lembrado na sessão. Como no Trial, os
    /// botões R G B A passam aos do kernel.
    pub fn set_pixel_filter_kernel(&mut self, i: usize) {
        let f = &mut self.pixel.edit.filter;
        if let (true, Ok(k)) = (i < f.kernels.len(), u8::try_from(i)) {
            f.kernel = k;
            if let Some(&c) = f.kernel_channels.get(i) {
                f.channels = c;
            }
        }
    }

    /// A lista do dropdown dos kernels está aberta?
    #[must_use]
    pub fn pixel_filter_kernel_list_open(&self) -> bool {
        matches!(
            self.get(crate::ids::PIXEL_FILTER_KERNEL),
            Some(crate::interaction::InteractiveState::Dropdown { open: true, .. })
        )
    }

    /// Fecha a lista do dropdown dos kernels; `true` se estava aberta.
    pub fn close_pixel_filter_kernel_list(&mut self) -> bool {
        let kernel = usize::from(self.pixel_filter_kernel());
        if let Some(crate::interaction::InteractiveState::Dropdown {
            open,
            selected_index,
            ..
        }) = self.get_mut(crate::ids::PIXEL_FILTER_KERNEL)
        {
            let was = *open;
            *open = false;
            *selected_index = Some(kernel);
            return was;
        }
        false
    }

    /// O kernel escolhido, preso à lista publicada (ao abrir, os botões R G B A passam aos dele).
    pub(super) fn pixel_filter_kernel(&self) -> u8 {
        let f = self.pixel.edit.filter;
        let last = u8::try_from(f.kernels.len().saturating_sub(1)).unwrap_or(u8::MAX);
        f.kernel.min(last)
    }

    /// **O editor de curva, a cada quadro**: com a caixa `rect` premida (activa), o premir agarra
    /// o ponto sob o ponteiro ou acrescenta um, e o arrasto move-o; largada, solta-o.
    pub fn drive_pixel_curve(&mut self, rect: Option<Rect>) {
        let activo = self.active_id() == Some(ids::PIXEL_FILTER_CURVE);
        let (Some(b), Some(p), true) = (rect, self.last_pointer(), activo) else {
            self.pixel.edit.filter.curve_grab = None;
            return;
        };
        let f = &mut self.pixel.edit.filter;
        let mut pts = f.curve.points().to_vec();
        let max = PIXEL_CURVE_MAX.min(widget::curve_capacity(b));
        let grab = match f.curve_grab {
            Some(i) => Some(i),
            None => widget::curve_press(b, &mut pts, p, max),
        };
        if let Some(i) = grab {
            widget::curve_drag(b, &mut pts, i, p);
        }
        f.curve_grab = grab;
        f.curve = PixelCurve::from_points(&pts);
    }

    /// O duplo-clique na caixa da curva: tira o ponto sob o ponteiro. `true` se tirou.
    pub fn remove_pixel_curve_point(&mut self, rect: Option<Rect>) -> bool {
        let (Some(b), Some(p)) = (rect, self.last_pointer()) else {
            return false;
        };
        let f = &mut self.pixel.edit.filter;
        let mut pts = f.curve.points().to_vec();
        let tirou = widget::curve_remove_at(b, &mut pts, p);
        f.curve = PixelCurve::from_points(&pts);
        f.curve_grab = None;
        tirou
    }
}

impl WidgetStore {
    /// A família publica onde o sprite está no ecrã (px), a cada quadro.
    pub fn set_pixel_edit_avoid(&mut self, r: Option<crate::pixel_panel::PixelOnScreen>) {
        self.pixel.edit.avoid = r;
    }
}
