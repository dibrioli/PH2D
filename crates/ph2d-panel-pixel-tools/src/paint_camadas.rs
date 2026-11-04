//! ⭐ **A secção CAMADAS do painel PIXEL** (P3, o menu Layer e a lista do Aseprite): uma linha por
//! camada, de cima para baixo — o triângulo do grupo, o olho, o cadeado, o nome (clique = activa,
//! duplo clique = renomear) e o Solo —; as propriedades da activa (modo e opacidade); e as acções.

use super::*;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::pixel_layer_row::LayerRowPainter;
use ph2d_editor_core::pixel_panel::{PixelBlend, PixelLayerRow};
use ph2d_editor_core::widget::{Dropdown, DropdownOption, paint_dropdown_chip};

/// As opções do menu do modo, na ordem dos ids do `.ase`.
pub(crate) fn blend_options() -> Vec<DropdownOption<PixelBlend>> {
    PixelBlend::ALL
        .iter()
        .enumerate()
        .map(|(i, &b)| DropdownOption::new(ids::blend_option_id(i), b, tr(b.label_key())))
        .collect()
}

impl Corpo<'_> {
    pub(super) fn camadas(&mut self, info: &PixelPanelInfo, renaming: Option<u32>, y: f32) -> f32 {
        let mut y = self.titulo(tr("panel.pixel.layers"), y);
        for row in &info.layers {
            y = self.linha_camada(
                row,
                info.active_layer == Some(row.id),
                renaming == Some(row.id),
                y,
            );
        }
        y += ph2d_tokens::control_gap_px();
        if let Some(row) = info
            .active_layer
            .and_then(|a| info.layers.iter().find(|r| r.id == a))
        {
            y = self.propriedades(row, y);
        }
        let a = ids::LAYER_ACTIONS;
        let fila = |i: usize, j: usize| [(a[i].0, a[i].2), (a[j].0, a[j].2)];
        y = self.botoes(&fila(0, 1), y);
        y = self.botoes(&fila(2, 3), y);
        y = self.botoes(&fila(4, 5), y);
        y = self.botoes(&fila(6, 7), y);
        y = self.botoes(&[(a[8].0, a[8].2)], y);
        // *via Copy/Cut* pedem selecção (o Aseprite desliga-os sem ela): sem selecção os botões não
        // se pintam (nenhum controlo morto). O Copy e o Select vivem nas secções Edit e Select.
        if info.has_selection {
            y = self.botoes(&fila(9, 10), y);
        }
        if info.has_clip {
            y = self.botoes(&fila(11, 12), y);
            y = self.botoes(&[(a[13].0, a[13].2)], y);
        }
        y
    }

    fn linha_camada(&mut self, row: &PixelLayerRow, active: bool, renaming: bool, y: f32) -> f32 {
        let mut p = LayerRowPainter {
            scene: self.scene,
            text_system: self.text_system,
            store: self.store,
            hit_index: self.hit_index,
            theme: self.theme,
            continuous: false,
        };
        let rename = renaming.then_some(ids::LAYER_RENAME);
        p.paint(row, (self.x, y, self.w), active, rename, &|w| {
            ids::layer_widget_id(row.id, w)
        })
    }

    /// O modo e a opacidade da activa; o grupo não tem modo próprio (passa os filhos como estão).
    fn propriedades(&mut self, row: &PixelLayerRow, y: f32) -> f32 {
        let nomes = [
            tr("panel.pixel.layer.mode"),
            tr("panel.pixel.layer.opacity"),
        ];
        let sec = Seccao::medida(self.text_system, 1, &nomes);
        let mut y = self.linha(
            &format!("{} · {}", tr("panel.pixel.layer.properties"), row.name),
            ColorToken::Text3,
            y,
        );
        if row.group {
            return self.bloco(tr("panel.pixel.layer.group_mode"), ColorToken::Text3, y);
        }
        let num_cartao = self.cartao.is_some();
        let r = if num_cartao {
            // Mode + Opacity dividem a linha: o chip é o de sempre, à largura de uma coluna.
            let meia = ((self.w - m::FIELD_GAP) * 0.5).max(0.0);
            let topo = self.legenda_no_cartao(tr("panel.pixel.layer.mode"), ColorToken::Text3, y);
            Rect::new(self.x, topo + m::LABEL_BOTTOM, meia, m::FIELD_H)
        } else {
            property_row::paint_label_row(
                self.scene,
                self.text_system,
                self.theme,
                self.x,
                self.w,
                y,
                ROW_H_PX,
                tr("panel.pixel.layer.mode"),
                sec,
            )
            .control
        };
        let open = matches!(
            self.store.get(ids::LAYER_BLEND),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
        let dd = Dropdown::new(ids::LAYER_BLEND, "", blend_options())
            .selected(row.blend)
            .open(open)
            .visual(self.store.dropdown_visual(ids::LAYER_BLEND));
        paint_dropdown_chip(&dd, r, self.scene, self.text_system, self.theme);
        self.hit_index.register(ids::LAYER_BLEND, r);
        if open {
            self.blend_aberto = Some((r, row.blend));
        }
        if num_cartao {
            let (x, w) = (self.x, self.w);
            self.x = x + r.w + m::FIELD_GAP;
            self.w = r.w;
            let fim =
                self.campos_no_cartao(&[(tr("panel.pixel.layer.opacity"), ids::LAYER_OPACITY)], y);
            self.x = x;
            self.w = w;
            return fim.max(r.y + r.h);
        }
        y = r.y + r.h + ph2d_tokens::control_gap_px();
        self.campo(tr("panel.pixel.layer.opacity"), ids::LAYER_OPACITY, sec, y)
    }
}
