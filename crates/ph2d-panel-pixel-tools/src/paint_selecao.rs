//! ⭐ **A P6 no painel PIXEL**: as opções das ferramentas de selecção (o modo de juntar, a varinha,
//! a amostragem), os campos da moldura solta (só com células levantadas, como no Aseprite) e os
//! menus Select, Edit e Sprite. O que pede selecção (ou uma cópia, ou o Reselect) não se pinta sem
//! ela — nenhum controlo morto.

use super::*;
use ph2d_editor_core::pixel_panel::PixelResample;

/// Os rótulos das opções da P6 (a coluna medida das propriedades).
pub(super) const NOMES: [&str; 3] = [
    "panel.pixel.wand.tolerance",
    "panel.pixel.connectivity",
    "panel.pixel.resample",
];

/// O par `(id, rótulo)` da posição `i` de uma tabela de acções.
fn par<const N: usize>(
    t: &[(
        NodeId,
        ph2d_editor_core::pixel_panel::PixelRequest,
        &'static str,
    ); N],
    i: usize,
) -> (NodeId, &'static str) {
    (t[i].0, t[i].2)
}

impl Corpo<'_> {
    /// As opções da ferramenta de selecção na mão (ou do Move).
    pub(super) fn opcoes_selecao(&mut self, info: &PixelPanelInfo, sec: Seccao, y: f32) -> f32 {
        let (o, t) = (info.options, info.tool);
        let mut y = y;
        if t.selects() {
            y = self.linha(tr("panel.pixel.select.mode"), ColorToken::Text3, y);
            // Dois a dois e sem rótulo (a largura toda): os quatro nomes inteiros não cabem numa
            // fila no degrau estreito, e um rótulo numa só das filas desalinhava-as (foto 26/10).
            let modos: Vec<(&str, bool, NodeId)> = ids::SELECT_MODES
                .iter()
                .map(|&(m, id)| (tr(m.label_key()), m == o.select_mode, id))
                .collect();
            for par in modos.chunks(2) {
                let r = Rect::new(self.x, y, self.w, ROW_H_PX);
                let h = ph2d_editor_core::widget::panel_chrome::paint_segmented_group_adaptive(
                    r,
                    par,
                    self.scene,
                    self.text_system,
                    self.theme,
                    self.store,
                    self.hit_index,
                );
                y += h.max(ROW_H_PX) + ph2d_tokens::control_gap_px();
            }
            y = self.bloco(tr("panel.pixel.select.mods_hint"), ColorToken::Text3, y);
        }
        if t == PixelTool::MagicWand {
            y = self.campo(
                tr("panel.pixel.wand.tolerance"),
                ids::WAND_TOLERANCE,
                sec,
                y,
            );
            y = self.caixas(
                &[(
                    ids::WAND_CONTIGUOUS,
                    tr("panel.pixel.contiguous"),
                    o.wand_contiguous,
                )],
                sec,
                y,
            );
            let conn = [
                (
                    tr("panel.pixel.connect4"),
                    !o.wand_eight,
                    ids::WAND_CONNECT_4,
                ),
                (
                    tr("panel.pixel.connect8"),
                    o.wand_eight,
                    ids::WAND_CONNECT_8,
                ),
            ];
            y = self.escolha(tr("panel.pixel.connectivity"), &conn, sec, y);
        }
        // Com células levantadas a amostragem mora nos campos da moldura (nunca duas vezes).
        if t == PixelTool::Transform && info.transform.is_none() {
            y = self.amostragem(o.resample, sec, y);
        }
        y
    }

    fn amostragem(&mut self, r: PixelResample, sec: Seccao, y: f32) -> f32 {
        let seg: Vec<(&str, bool, NodeId)> = ids::RESAMPLES
            .iter()
            .map(|&(k, id)| (tr(k.label_key()), k == r, id))
            .collect();
        self.escolha(tr("panel.pixel.resample"), &seg, sec, y)
    }

    /// ⭐ **A moldura solta**: X, Y, W, H, Angle (anti-horário), Skew, a amostragem, Apply | Cancel.
    /// Com uma forma viva (P6f) a mesma barra é a caixa dela, sem amostragem (ela não reamostra).
    pub(super) fn moldura(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        if info.transform.is_none() {
            return y;
        }
        let nomes: Vec<&str> = ids::XFORM_FIELDS
            .iter()
            .map(|(f, ..)| tr(f.label_key()))
            .chain(NOMES.iter().map(|k| tr(k)))
            .collect();
        let sec = Seccao::medida(self.text_system, 1, &nomes);
        let mut y = self.linha(tr("panel.pixel.transform"), ColorToken::Text3, y);
        for (f, id, _) in ids::XFORM_FIELDS {
            y = self.campo(tr(f.label_key()), id, sec, y);
        }
        if !info.live_shape {
            y = self.amostragem(info.options.resample, sec, y);
            y = self.bloco(tr("panel.pixel.transform_hint"), ColorToken::Text3, y);
        }
        let a = &ids::EDIT_ACTIONS;
        self.botoes(&[par(a, 16), par(a, 15)], y)
    }

    /// ⭐ **Os menus Select e Edit.**
    pub(super) fn menus_selecao(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let s = &ids::SELECT_ACTIONS;
        let mut y = self.linha(tr("panel.pixel.select"), ColorToken::Text3, y);
        let sel = info.has_selection;
        y = if sel {
            self.botoes(&[par(s, 0), par(s, 1)], y)
        } else {
            self.botoes(&[par(s, 0)], y)
        };
        y = if info.can_reselect {
            self.botoes(&[par(s, 2), par(s, 3)], y)
        } else {
            self.botoes(&[par(s, 3)], y)
        };
        y = self.botoes(&[par(s, 4)], y);
        if sel {
            y = self.botoes(&[par(s, 5), par(s, 6)], y);
            y = self.botoes(&[par(s, 7)], y);
        }
        let e = &ids::EDIT_ACTIONS;
        y = self.linha(tr("panel.pixel.edit"), ColorToken::Text3, y);
        if sel {
            y = self.botoes(&[par(e, 0), par(e, 1)], y);
            y = self.botoes(&[par(e, 2)], y);
        }
        if info.has_clip {
            y = self.botoes(&[par(e, 3)], y);
        }
        y = self.botoes(&[par(e, 4), par(e, 5)], y);
        y = self.botoes(&[par(e, 6), par(e, 7)], y);
        y = self.botoes(&[par(e, 8), par(e, 9)], y);
        y = self.botoes(&[par(e, 10)], y);
        if sel {
            y = self.botoes(&[par(e, 11), par(e, 12)], y);
            y = self.botoes(&[par(e, 13), par(e, 14)], y);
        }
        y
    }

    /// Os botões do menu Sprite (o Crop pede selecção).
    pub(super) fn menu_sprite(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let s = &ids::SPRITE_ACTIONS;
        let mut y = self.botoes(&[par(s, 0), par(s, 1)], y);
        y = self.botoes(&[par(s, 2), par(s, 3)], y);
        y = self.botoes(&[par(s, 4)], y);
        y = self.botoes(&[par(s, 5), par(s, 6)], y);
        if info.has_selection {
            self.botoes(&[par(s, 7), par(s, 8)], y)
        } else {
            self.botoes(&[par(s, 8)], y)
        }
    }
}
