//! **P13: as rampas sob a grelha da paleta** — uma faixa com borda por rampa (a da amostra clicada
//! acende), *Make Ramp* / *New Ramp…*, o *Cycle* da rampa e o diálogo New Ramp em linhas do cartão.

use super::*;
use crate::seam_rampas as r;

impl Corpo<'_> {
    pub(super) fn rampas(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let gap = ph2d_tokens::control_gap_px();
        let cell = Spacing::Lg.px();
        let atual = r::atual(info).map(|a| a.id);
        let mut y = y;
        for rampa in &info.ramps {
            #[expect(clippy::cast_precision_loss, reason = "uma rampa, ≤ 256")]
            let n = rampa.indices.len().max(1) as f32;
            let faixa = Rect::new(self.x, y, (cell * n).min(self.w), cell);
            let w = faixa.w / n;
            for (k, &i) in rampa.indices.iter().enumerate() {
                let c = info
                    .palette
                    .get(usize::from(i))
                    .copied()
                    .unwrap_or_default();
                #[expect(clippy::cast_precision_loss, reason = "uma rampa, ≤ 256")]
                let rr = Rect::new(faixa.x + k as f32 * w, y, w, cell);
                paint_color_swatch(
                    &ColorSwatch::new(r::STRIP, "", c),
                    rr,
                    self.scene,
                    self.theme,
                );
            }
            let (cor, traco) = if atual == Some(rampa.id) {
                (ColorToken::Accent, StrokeToken::Thick)
            } else {
                (ColorToken::BorderStrong, StrokeToken::Thin)
            };
            stroke_rect(self.scene, faixa, traco.px(), tinta(cor, self.theme));
            y += cell + gap;
        }
        let livre = !info.selected.is_empty()
            && !info.selected.iter().any(|&i| {
                info.ramps
                    .iter()
                    .any(|a| a.indices.iter().any(|&j| usize::from(j) == i))
            });
        let novo = (r::NEW, "pixel.palette.new_ramp");
        y = if livre {
            self.botoes(&[(r::MAKE, "pixel.ramp.make"), novo], y)
        } else {
            self.botoes(&[novo], y)
        };
        let nomes: Vec<&str> = r::FIELDS.iter().map(|(_, k, ..)| tr(k)).collect();
        let sec = Seccao::medida(self.text_system, 1, &nomes);
        if self.paleta_vista.nova_rampa {
            y = self.linha(tr("pixel.ramp.base"), ColorToken::Text3, y);
            for ((id, ..), nome) in r::FIELDS.iter().zip(&nomes) {
                y = self.campo(nome, *id, sec, y);
            }
            y = self.botoes(&[(r::CREATE, "pixel.ramp.create")], y);
        }
        let Some(rampa) = r::atual(info) else {
            return y;
        };
        y = self.botoes(&[(r::REMOVE, "pixel.ramp.remove")], y);
        let caixa = [(r::CYCLE, tr("pixel.cycle"), rampa.cycle.is_some())];
        y = self.caixas(&caixa, sec, y);
        let Some(c) = rampa.cycle else {
            return y;
        };
        y = self.campo(tr("pixel.cycle.period"), r::PERIOD, sec, y);
        let segs: Vec<(&str, bool, NodeId)> = r::DIRECTIONS
            .iter()
            .map(|&(d, id, k)| (tr(k), d == c.direction, id))
            .collect();
        self.escolha(tr("pixel.cycle.direction"), &segs, sec, y)
    }
}
