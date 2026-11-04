//! O cartão das CAMADAS do Flip: a barra de verbos e, por camada, 👁 🔒 nome ↑ ↓, mistura e números.

use super::*;

pub(super) fn cartao_das_camadas(
    layers: FlipLayersSnapshot,
    renomeia: Option<u64>,
) -> CardDef<'static> {
    let resumo = layers.rows.len().to_string();
    cartao(
        ids::FLIP_SEC_LAYERS,
        tr("panel.flip.layers.layers"),
        Some(resumo),
        move |c, y| {
            let botao = |id, chave, ativo| {
                let text = tr(chave);
                Cell::new(if ativo {
                    Widget::Button {
                        id,
                        text,
                        primary: false,
                    }
                } else {
                    Widget::ButtonOff { text }
                })
            };
            let ha = layers.active.is_some();
            let mut y = c.row(
                &[
                    botao(t::FLIP_LAYER_ADD, "panel.flip.layers.add", true),
                    botao(ids::FLIP_LAYER_DUPLICATE, "panel.flip.layers.duplicate", ha),
                    botao(ids::FLIP_LAYER_DELETE, "panel.flip.layers.delete", ha),
                ],
                y,
            );
            // De cima para baixo; o índice 0 é o fundo, sem mistura (nada por baixo para misturar).
            let n = layers.rows.len();
            for (i, row) in layers.rows.iter().enumerate().rev() {
                y = camada(
                    c,
                    row,
                    i,
                    n,
                    layers.active == Some(row.id),
                    renomeia == Some(row.id),
                    y,
                );
            }
            y
        },
    )
}

/// A seta desligada pinta-se apagada e não regista alvo (no-op, como no corpo de antes).
fn seta(c: &mut CardCtx<'_>, id: NodeId, icon: IconId, r: Rect, ativa: bool) {
    if ativa {
        c.bare(
            r,
            &[Seg {
                id,
                icon: Some(icon),
                text: "",
                on: false,
            }],
        );
        return;
    }
    let g = m::ICON_GLYPH;
    paint_icon(
        c.scene,
        icon,
        Rect::new(r.x + (r.w - g) * 0.5, r.y + (r.h - g) * 0.5, g, g),
        c.ink.color(ColorToken::TextDisabled),
        StrokeToken::Default.px(),
    );
}

/// Uma camada: 👁 🔒 nome ↑ ↓ e a marca do olho; a mistura; opacidade e profundidade.
fn camada(
    c: &mut CardCtx<'_>,
    row: &FlipLayerRow,
    i: usize,
    n: usize,
    ativa: bool,
    renomeia: bool,
    y: f32,
) -> f32 {
    let id = |k| ids::flip_layer_widget_id(row.id, k);
    let y = y + m::FIELD_ROW_GAP + m::FIELD_ROW_GAP;
    let (x0, w0) = (c.x, c.w);
    let linha = Rect::new(x0, y, w0, m::FIELD_H);
    if ativa {
        fill_rounded_rect(
            c.scene,
            linha,
            m::SEG_RADIUS,
            c.ink.color(ColorToken::KanbanCardRaised),
        );
    }
    let cabeca = [
        Seg {
            id: id(FlipLayerWidget::Visibility),
            icon: Some(if row.visible {
                IconId::Eye
            } else {
                IconId::EyeClosed
            }),
            text: "",
            on: false,
        },
        Seg {
            id: id(FlipLayerWidget::Lock),
            icon: Some(if row.locked {
                IconId::Lock
            } else {
                IconId::Unlock
            }),
            text: "",
            on: false,
        },
    ];
    c.bare(Rect::new(x0, y, m::FIELD_H * 2.0, m::FIELD_H), &cabeca);
    let fim = x0 + w0 - m::KEY_MARK_BOX;
    seta(
        c,
        id(FlipLayerWidget::MoveUp),
        IconId::ChevronUp,
        Rect::new(fim - m::FIELD_H * 2.0, y, m::FIELD_H, m::FIELD_H),
        i + 1 < n,
    );
    seta(
        c,
        id(FlipLayerWidget::MoveDown),
        IconId::ChevronDown,
        Rect::new(fim - m::FIELD_H, y, m::FIELD_H, m::FIELD_H),
        i > 0,
    );
    let olho = [id(FlipLayerWidget::Visibility)];
    let ink = c.ink;
    paint_key_mark(
        c.scene,
        ink,
        c.hit_index,
        fim + m::KEY_MARK_BOX * 0.5,
        y + m::FIELD_H * 0.5,
        &olho,
    );
    c.x = x0 + m::FIELD_H * 2.0;
    c.w = (fim - m::FIELD_H * 2.0 - c.x).max(0.0);
    let mut nome = ListRow::new(id(FlipLayerWidget::Row), &row.name);
    nome.selected = ativa;
    nome.dim = !row.visible;
    nome.edit = renomeia.then_some(ids::FLIP_LAYER_RENAME_INPUT);
    let mut y = c.list_row(&nome, y);
    c.x = x0;
    c.w = w0;
    if i > 0 {
        y = mistura(c, row, y);
    }
    let barras = [
        [
            id(FlipLayerWidget::Opacity),
            id(FlipLayerWidget::OpacityNum),
        ],
        [id(FlipLayerWidget::Depth), id(FlipLayerWidget::DepthNum)],
    ];
    c.row(
        &[
            campo(barras[0][1], tr("card.flip.opacity")).key(&barras[0]),
            campo(barras[1][1], tr("card.flip.depth")).key(&barras[1]),
        ],
        y,
    )
}

fn mistura(c: &mut CardCtx<'_>, row: &FlipLayerRow, y: f32) -> f32 {
    let caixa = [ids::flip_layer_widget_id(row.id, FlipLayerWidget::Blend)];
    let segs: Vec<Seg<'static>> = (0..MAX_BLEND_MODES)
        .map(|modo| Seg {
            id: ids::flip_layer_blend_option_id(row.id, modo),
            icon: None,
            text: tr(BlendMode::from_u8(modo).label_key()),
            on: modo == row.blend,
        })
        .collect();
    let celula = Cell::new(Widget::Dropdown {
        id: caixa[0],
        text: tr(BlendMode::from_u8(row.blend).label_key()),
        options: &segs,
    })
    .label(tr("card.flip.blend"))
    .key(&caixa);
    c.row(&[celula], y)
}
