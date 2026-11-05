//! ⭐ **A P6 no painel PIXEL**: as opções das ferramentas de selecção (o modo de juntar, a varinha,
//! a amostragem), os campos da moldura solta (só com células levantadas, como no Aseprite) e os
//! menus Select, Edit e Sprite. O que pede selecção (ou uma cópia, ou o Reselect) não se pinta sem
//! ela — nenhum controlo morto.

use super::*;
use ph2d_card_stack::metrics as m;
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
                if self.cartao.is_some() {
                    y = self.par_no_cartao(par, y);
                    continue;
                }
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
            y = self.campo_e_escolha(
                (tr("panel.pixel.wand.tolerance"), ids::WAND_TOLERANCE),
                (tr("panel.pixel.connectivity"), &conn),
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
            let amostra = [
                (
                    tr("panel.pixel.sample.layer"),
                    !o.wand_sample_visible,
                    ids::WAND_SAMPLE_LAYER,
                ),
                (
                    tr("panel.pixel.sample.visible"),
                    o.wand_sample_visible,
                    ids::WAND_SAMPLE_VISIBLE,
                ),
            ];
            y = self.escolha(tr("panel.pixel.sample"), &amostra, sec, y);
        }
        if t == PixelTool::Transform {
            let cx = [(
                ids::AUTO_SELECT_LAYER,
                tr("panel.pixel.auto_select_layer"),
                o.auto_select_layer,
            )];
            y = self.caixas(&cx, sec, y);
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
        let mut y = self.titulo(tr("panel.pixel.transform"), y);
        for par in ids::XFORM_FIELDS.chunks(2) {
            y = match par {
                [(a, ia, _), (b, ib, _)] => {
                    self.dois_campos((tr(a.label_key()), *ia), (tr(b.label_key()), *ib), sec, y)
                }
                [(a, ia, _)] => self.campo(tr(a.label_key()), *ia, sec, y),
                _ => y,
            };
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
        let y = self.menu_select(info, y);
        self.menu_edit(info, y)
    }

    pub(super) fn menu_select(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let s = &ids::SELECT_ACTIONS;
        let y = self.titulo(tr("panel.pixel.select"), y);
        let sel = info.has_selection;
        let mut filas = vec![if sel {
            vec![par(s, 0), par(s, 1)]
        } else {
            vec![par(s, 0)]
        }];
        filas.push(if info.can_reselect {
            vec![par(s, 2), par(s, 3)]
        } else {
            vec![par(s, 3)]
        });
        filas.push(vec![par(s, 4)]);
        if sel {
            filas.push(vec![par(s, 5), par(s, 6)]);
            filas.push(vec![par(s, 7)]);
            filas.push(vec![par(s, 8), par(s, 9)]);
        } else {
            filas.push(vec![par(s, 8)]);
        }
        self.filas_de_botoes(&filas, y)
    }

    /// Os verbos de um menu: num cartão, aos pares pela ordem, salvo o que não cabe na metade (lei 1 de 05 §4);
    /// no painel de antes, as filas como vêm.
    fn filas_de_botoes(&mut self, filas: &[Vec<(NodeId, &str)>], y: f32) -> f32 {
        if self.cartao.is_none() {
            return filas.iter().fold(y, |y, f| self.botoes(f, y));
        }
        let todos: Vec<(NodeId, &str)> = filas.iter().flatten().copied().collect();
        let meia = (self.w - m::FIELD_GAP) * 0.5;
        let (mut y, mut i) = (y, 0);
        while i < todos.len() {
            let ts = &mut *self.text_system;
            let mut cabe =
                |k: &str| ts.prefix_width(tr(k), m::VALUE_PX) + 2.0 * m::FIELD_PAD_R <= meia;
            let n = if i + 1 < todos.len() && cabe(todos[i].1) && cabe(todos[i + 1].1) {
                2
            } else {
                1
            };
            y = self.botoes(&todos[i..i + n], y);
            i += n;
        }
        y
    }

    pub(super) fn menu_edit(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let sel = info.has_selection;
        let e = &ids::EDIT_ACTIONS;
        let y = self.titulo(tr("panel.pixel.edit"), y);
        let mut filas = Vec::new();
        if sel {
            filas.push(vec![par(e, 0), par(e, 1)]);
            filas.push(vec![par(e, 2)]);
            // P8: Fill e Stroke pedem selecção (o Aseprite desliga-os sem ela).
            filas.push(vec![par(e, 18), par(e, 19)]);
        }
        if info.has_clip {
            filas.push(vec![par(e, 3)]);
        }
        filas.push(vec![par(e, 17)]);
        filas.push(vec![par(e, 4), par(e, 5)]);
        filas.push(vec![par(e, 6), par(e, 7)]);
        filas.push(vec![par(e, 8), par(e, 9)]);
        filas.push(vec![par(e, 10)]);
        if sel {
            filas.push(vec![par(e, 11), par(e, 12)]);
            filas.push(vec![par(e, 13), par(e, 14)]);
        }
        // P8: Invert… e o grupo FX ▸ (sem widget de submenu: o grupo é uma secção).
        filas.push(vec![par(e, 20)]);
        filas.push(vec![par(e, 27)]);
        let y = self.filas_de_botoes(&filas, y);
        // P8: Adjustments ▸ (secção, como o FX).
        let y = self.linha(tr("panel.pixel.edit.adjustments"), ColorToken::Text3, y);
        let y = self.filas_de_botoes(&[vec![par(e, 23)], vec![par(e, 24), par(e, 25)]], y);
        let y = self.linha(tr("panel.pixel.edit.fx"), ColorToken::Text3, y);
        self.filas_de_botoes(&[vec![par(e, 21), par(e, 22)], vec![par(e, 26)]], y)
    }

    /// Os botões do menu Sprite (o Crop pede selecção).
    pub(super) fn menu_sprite(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let s = &ids::SPRITE_ACTIONS;
        let mut filas = vec![
            vec![par(s, 0), par(s, 1)],
            vec![par(s, 2), par(s, 3)],
            vec![par(s, 4)],
            vec![par(s, 5), par(s, 6)],
        ];
        filas.push(if info.has_selection {
            vec![par(s, 7), par(s, 8)]
        } else {
            vec![par(s, 8)]
        });
        self.filas_de_botoes(&filas, y)
    }
}
