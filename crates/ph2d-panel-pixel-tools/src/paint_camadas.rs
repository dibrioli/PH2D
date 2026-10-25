//! ⭐ **A secção CAMADAS do painel PIXEL** (P3, o menu Layer e a lista do Aseprite): uma linha por
//! camada, de cima para baixo — o triângulo do grupo, o olho, o cadeado, o nome (clique = activa,
//! duplo clique = renomear) e o Solo —; as propriedades da activa (modo e opacidade); e as acções.

use super::*;
use crate::ids::LayerWidget;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon};
use ph2d_editor_core::pixel_panel::{PixelBlend, PixelLayerRow};
use ph2d_editor_core::widget::{
    Dropdown, DropdownOption, TextInput, TextInputState, paint_dropdown_chip,
    paint_text_input_with_buffer,
};
use ph2d_tokens::Radius;

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
            y = self.linha_camada(row, info.active_layer == Some(row.id), renaming == Some(row.id), y);
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
        y = self.botoes_camada(&fila(0, 1), y);
        y = self.botoes_camada(&fila(2, 3), y);
        y = self.botoes_camada(&fila(4, 5), y);
        y = self.botoes_camada(&fila(6, 7), y);
        y = self.botoes_camada(&[(a[8].0, a[8].2)], y);
        y = self.botoes_camada(&fila(9, 10), y);
        y = self.botoes_camada(&[(a[11].0, a[11].2)], y);
        if info.has_clip {
            y = self.botoes_camada(&fila(12, 13), y);
        }
        y
    }

    fn icone(&mut self, id: ph2d_a11y::NodeId, icon: IconId, r: Rect, on: bool) {
        let cor = if on { ColorToken::Text1 } else { ColorToken::Text3 };
        paint_icon(
            self.scene,
            icon,
            r,
            resolve(cor, self.theme),
            StrokeToken::Default.px(),
        );
        self.hit_index.register(id, r);
    }

    fn linha_camada(&mut self, row: &PixelLayerRow, active: bool, renaming: bool, y: f32) -> f32 {
        let h = ROW_H_PX;
        let gap = Spacing::Xs.px();
        if active {
            let r = Rect::new(self.x, y, self.w, h);
            let radius = ph2d_editor_core::paint::frame_radius(self.theme, Radius::Sm.px());
            fill_rounded_rect(self.scene, r, radius, resolve(ColorToken::AccentSoft, self.theme));
        }
        #[expect(clippy::cast_precision_loss, reason = "a profundidade de um grupo, ≤ 255")]
        let recuo = f32::from(row.depth) * Spacing::Md.px();
        let mut x = self.x + recuo;
        let quadrado = |x: f32| Rect::new(x, y, h, h);
        if row.group {
            let icon = if row.collapsed {
                IconId::ChevronRight
            } else {
                IconId::ChevronDown
            };
            self.icone(ids::layer_widget_id(row.id, LayerWidget::Collapse), icon, quadrado(x), true);
            x += h + gap;
        }
        let olho = if row.visible { IconId::Eye } else { IconId::EyeClosed };
        self.icone(ids::layer_widget_id(row.id, LayerWidget::Eye), olho, quadrado(x), row.visible);
        x += h + gap;
        let cadeado = if row.locked { IconId::Lock } else { IconId::Unlock };
        self.icone(ids::layer_widget_id(row.id, LayerWidget::Lock), cadeado, quadrado(x), row.locked);
        x += h + gap;
        let solo_x = self.x + self.w - h;
        self.icone(
            ids::layer_widget_id(row.id, LayerWidget::Solo),
            IconId::Circle,
            quadrado(solo_x),
            false,
        );
        let nome_r = Rect::new(x, y, (solo_x - gap - x).max(1.0), h);
        if renaming {
            let (st, text, caret, anchor) = match self.store.get(ids::LAYER_RENAME) {
                Some(InteractiveState::TextInput {
                    state,
                    text,
                    caret,
                    selection_anchor,
                }) => (*state, text.clone(), *caret, *selection_anchor),
                _ => (TextInputState::Focused, String::new(), 0, None),
            };
            let input = TextInput::new(ids::LAYER_RENAME, "")
                .visual((st, self.store.hover_live(ids::LAYER_RENAME)));
            paint_text_input_with_buffer(
                &input,
                Some(text.as_str()),
                Some(caret),
                anchor,
                nome_r,
                self.scene,
                self.text_system,
                self.theme,
            );
            self.hit_index.register(ids::LAYER_RENAME, nome_r);
        } else {
            let nome = if row.reference {
                tr_with("panel.pixel.layer.reference", &[("name", &row.name)])
            } else {
                row.name.clone()
            };
            let cor = if row.visible { ColorToken::Text1 } else { ColorToken::Text3 };
            paint_text(
                self.text_system,
                self.scene,
                &nome,
                nome_r.x,
                y + (h - TypeToken::Sm.px()) * 0.5,
                TypeToken::Sm.px(),
                nome_r.w,
                resolve(cor, self.theme),
            );
            self.hit_index
                .register(ids::layer_widget_id(row.id, LayerWidget::Row), nome_r);
        }
        y + h + Spacing::Xxs.px()
    }

    /// O modo e a opacidade da activa; o grupo não tem modo próprio (passa os filhos como estão).
    fn propriedades(&mut self, row: &PixelLayerRow, y: f32) -> f32 {
        let nomes = [tr("panel.pixel.layer.mode"), tr("panel.pixel.layer.opacity")];
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

    fn botoes_camada(&mut self, fila: &[(ph2d_a11y::NodeId, &str)], y: f32) -> f32 {
        let gap = Spacing::Xs.px();
        #[expect(clippy::cast_precision_loss, reason = "≤ 2 botões")]
        let n = fila.len() as f32;
        let bw = ((self.w - gap * (n - 1.0)) / n).max(1.0);
        for (i, (id, key)) in fila.iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "≤ 2 botões")]
            let r = Rect::new(self.x + i as f32 * (bw + gap), y, bw, ROW_H_PX);
            let b = Button::new(*id, tr(key)).visual(self.store.button_visual(*id));
            paint_button(&b, r, self.scene, self.text_system, self.theme);
            self.hit_index.register(*id, r);
        }
        y + ROW_H_PX + ph2d_tokens::control_gap_px()
    }
}
