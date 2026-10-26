//! ⭐ **A secção CAMADAS do painel PIXEL** (P3, o menu Layer e a lista do Aseprite): uma linha por
//! camada, de cima para baixo — o triângulo do grupo, o olho, o cadeado, o nome (clique = activa,
//! duplo clique = renomear) e o Solo —; as propriedades da activa (modo e opacidade); e as acções.

use super::*;
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
        let mut y = self.linha(tr("panel.pixel.layers"), ColorToken::Text3, y);
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
        // A cópia precisa de selecção (o Aseprite desliga Copy e via Copy/Cut sem ela): sem selecção
        // os botões não se pintam (nenhum controlo morto).
        let sel = if info.has_selection {
            "panel.pixel.layer.selection_all"
        } else {
            "panel.pixel.layer.selection_none"
        };
        y = self.linha(tr(sel), ColorToken::Text3, y);
        y = self.botoes(&fila(14, 15), y);
        if info.has_selection {
            y = self.botoes(&fila(9, 10), y);
            y = self.botoes(&[(a[11].0, a[11].2)], y);
        }
        if info.has_clip {
            y = self.botoes(&fila(12, 13), y);
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
        let r = property_row::caixa_do_controlo(self.x, self.w, y, sec);
        paint_text(
            self.text_system,
            self.scene,
            tr("panel.pixel.layer.mode"),
            self.x,
            y + (ROW_H_PX - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            (r.x - self.x).max(1.0),
            resolve(ColorToken::Text2, self.theme),
        );
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
        y += ROW_H_PX + ph2d_tokens::control_gap_px();
        self.campo(tr("panel.pixel.layer.opacity"), ids::LAYER_OPACITY, sec, y)
    }
}
