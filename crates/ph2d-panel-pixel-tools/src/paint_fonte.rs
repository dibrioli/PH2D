//! P8b: **o dropdown de fontes do Text** — o chip na fila «Font» (acima de B, I, Antialias e
//! Hinting) e a lista com rolagem pela porta da fundação (`panel::paint_dropdown_list`, a mesma do
//! Vector). Cada linha desenha o nome NA PRÓPRIA FONTE, em células, com o rasterizador do Pixel Art
//! (`pixel_panel::font_preview`: o painel pede as visíveis, a família publica-as); até chegar, o
//! nome sai na fonte da UI. A ordem é a de `ph2d_pixel_text::carimbo::font_list`.

use super::*;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::pixel_panel::PixelRequest;
use ph2d_editor_core::pixel_panel::font_preview::{self, PixelFontPreview};
use ph2d_editor_core::widget::{Dropdown, DropdownOption, PopoverRow, paint_dropdown_chip};
use ph2d_vector::Color;

fn opcoes(info: &PixelPanelInfo) -> Vec<DropdownOption<usize>> {
    info.text_fonts
        .iter()
        .enumerate()
        .map(|(i, f)| DropdownOption::new(ids::text_font_option_id(i), i, f.clone()))
        .collect()
}

fn dropdown(info: &PixelPanelInfo, open: bool) -> Dropdown<usize> {
    let dd = Dropdown::new(ids::TEXT_FONT_DD, "", opcoes(info)).open(open);
    dd.selected(usize::from(info.options.text_font))
}

impl Corpo<'_> {
    /// «Font» à esquerda e o chip com o nome da fonte na coluna do valor.
    pub(super) fn chip_da_fonte(&mut self, info: &PixelPanelInfo, sec: Seccao, y: f32) -> f32 {
        let r = property_row::paint_label_row(
            self.scene,
            self.text_system,
            self.theme,
            self.x,
            self.w,
            y,
            ROW_H_PX,
            tr("panel.pixel.text.font"),
            sec,
        )
        .control;
        let open = matches!(
            self.store.get(ids::TEXT_FONT_DD),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
        let mut dd = dropdown(info, open).visual(self.store.dropdown_visual(ids::TEXT_FONT_DD));
        if info.text_fonts.is_empty() {
            // Sem a lista (instantâneo mínimo): o chip mostra o nome publicado.
            dd = Dropdown::new(
                ids::TEXT_FONT_DD,
                "",
                vec![DropdownOption::new(
                    ids::text_font_option_id(0),
                    0,
                    info.text_font_name.clone(),
                )],
            )
            .selected(0);
        }
        paint_dropdown_chip(&dd, r, self.scene, self.text_system, self.theme);
        self.hit_index.register(ids::TEXT_FONT_DD, r);
        if open {
            self.fonte_aberta = Some(r);
        }
        r.y + r.h + ph2d_tokens::control_gap_px()
    }
}

/// A lista aberta, por cima do painel.
pub(crate) fn popover(ctx: &mut PaintCtx, chip: Rect, theme: Theme) {
    let (dd, nomes) = with_current(|info| (dropdown(info, true), info.text_fonts.clone()));
    let mut linha = |scene: &mut VectorScene, text: &mut TextSystem, row: PopoverRow| {
        let Some(nome) = nomes.get(row.index) else {
            return;
        };
        let cor = resolve(row.ink(), theme);
        let feito = font_preview::with_preview(nome, |p| {
            p.filter(|p| p.w > 0 && p.h > 0)
                .map(|p| celulas(scene, p, row.rect, cor))
                .is_some()
        });
        if !feito {
            font_preview::want(nome);
            let pad = Spacing::Md.px();
            paint_text(
                text,
                scene,
                nome,
                row.rect.x + pad,
                row.rect.y + (row.rect.h - TypeToken::Base.px()) * 0.5,
                TypeToken::Base.px(),
                (row.rect.w - pad * 2.0).max(1.0),
                cor,
            );
        }
    };
    let largura = largura_pintada(ctx.text_system, &nomes, chip.h);
    let viewport = ctx.viewport;
    ph2d_editor_core::panel::paint_dropdown_list_measured(
        ctx,
        &dd,
        chip,
        viewport,
        theme,
        Some(&mut linha),
        largura,
    );
}

/// **A largura da linha mais larga COMO SE PINTA** (P8c), para a lei da largura da porta: a prévia
/// na escala inteira dela ([`PixelFontPreview::scale_in`] na linha de `row_h`), ou o nome na fonte da
/// interface quando a prévia está vazia (face sem as letras do nome) ou ainda não chegou. Medir só o
/// rótulo na fonte da UI (a porta comum) dava a largura errada a cada linha desenhada noutra fonte.
#[must_use]
pub fn largura_pintada(text: &mut TextSystem, nomes: &[String], row_h: f32) -> f32 {
    nomes.iter().fold(0.0_f32, |w, nome| {
        let previa = font_preview::with_preview(nome, |p| {
            #[expect(clippy::cast_precision_loss, reason = "largura de uma máscara de nome")]
            p.filter(|p| p.w > 0 && p.h > 0)
                .map(|p| p.w as f32 * p.scale_in(row_h))
        });
        w.max(previa.unwrap_or_else(|| {
            ph2d_editor_core::paint::label_column_width(text, TypeToken::Base.px(), [nome.as_str()])
        }))
    })
}

/// A máscara `p` na linha `r`: cada célula um quadrado inteiro de ecrã na maior escala inteira que
/// cabe na linha ([`PixelFontPreview::scale_in`]; a fração 0,7 de antes deixava o unscii-8 a 1×),
/// corridas da mesma cobertura num só retângulo.
fn celulas(scene: &mut VectorScene, p: &PixelFontPreview, r: Rect, cor: Color) {
    let s = p.scale_in(r.h);
    let pad = Spacing::Md.px();
    #[expect(clippy::cast_precision_loss, reason = "altura de uma máscara de nome")]
    let y0 = r.y + ((r.h - p.h as f32 * s) * 0.5).floor();
    let x0 = r.x + pad;
    let x_max = r.x + r.w - pad;
    for my in 0..p.h {
        let fila = &p.cover[(my * p.w) as usize..((my + 1) * p.w) as usize];
        let mut mx = 0;
        while mx < fila.len() {
            let c = fila[mx];
            let ini = mx;
            while mx < fila.len() && fila[mx] == c {
                mx += 1;
            }
            #[expect(clippy::cast_precision_loss, reason = "colunas de uma máscara de nome")]
            let (xa, xb) = (x0 + ini as f32 * s, (x0 + mx as f32 * s).min(x_max));
            if c == 0 || xa >= x_max {
                continue;
            }
            #[expect(clippy::cast_precision_loss, reason = "linhas de uma máscara de nome")]
            let ya = y0 + my as f32 * s;
            let alfa = f32::from(c) / f32::from(u8::MAX);
            scene.fill_rect(
                ph2d_vector::Rect::new(
                    f64::from(xa),
                    f64::from(ya),
                    f64::from(xb),
                    f64::from(ya + s),
                ),
                cor.multiply_alpha(alfa),
            );
        }
    }
}

/// O índice da fonte cuja linha é `id`.
pub(crate) fn linha(id: NodeId) -> Option<usize> {
    with_current(|i| (0..i.text_fonts.len()).find(|&k| ids::text_font_option_id(k) == id))
}

/// Escolher a linha `id`: fecha a lista e pede as opções com essa fonte (o app salta o tamanho).
pub(crate) fn escolhe(
    host: &mut dyn ph2d_editor_core::panel::PanelHostInternal,
    id: NodeId,
) -> Option<PixelRequest> {
    let k = linha(id)?;
    if let Some(InteractiveState::Dropdown {
        open,
        selected_index,
        ..
    }) = host.store_mut().get_mut(ids::TEXT_FONT_DD)
    {
        *open = false;
        *selected_index = Some(k);
    }
    with_current(|info| {
        let mut o = info.options;
        o.text_font = u16::try_from(k).ok()?;
        info.editing.then_some(PixelRequest::Options(Box::new(o)))
    })
}
