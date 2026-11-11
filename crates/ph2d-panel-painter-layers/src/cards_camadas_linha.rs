//! Uma camada do cartão Layers (`crate::cards_camadas`): 👁 nome ↑ ↓, a mistura com a opacidade, o relevo e a sub-linha da
//! máscara. Cortado por responsabilidade: lá mora o cartão e o arrasto, aqui a linha.

use crate::paint_rows::PCT_SCALE;
use crate::state;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon};
use ph2d_editor_core::panel::cards::linhas_extra::{CardBar, ListRow};
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_tool_painter::ids::{PainterLayerWidget, painter_layer_widget_id};
use ph2d_tool_painter::{BlendMode, Layer, LayerId, LayerKind, LayerStack, ReliefComposite};

/// O relevo mora num número com sinal: `-1..1`.
const FUNDO_DO_RELEVO: f32 = -1.0; // LITERAL-PX-OK: profundidade com sinal, não é medida de UI
const AMPLITUDE_DO_RELEVO: f32 = 2.0; // LITERAL-PX-OK: profundidade com sinal, não é medida de UI

/// O olho e as duas setas, na linha da camada.
const CELULAS_DE_ICONE: f32 = 3.0; // LITERAL-PX-OK: contagem de células, não é medida de UI

pub(crate) fn recuo(depth: usize) -> f32 {
    depth as f32 * ph2d_tokens::list_indent_px()
}

pub(crate) fn fracao_do_relevo(l: &Layer) -> f32 {
    (l.impasto_depth - FUNDO_DO_RELEVO) / AMPLITUDE_DO_RELEVO
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

/// Uma barra nua com o nome e a percentagem em cima: o id é o de sempre, um `Slider`, e não tem número para digitar.
fn barra(c: &mut CardCtx<'_>, id: NodeId, chave: &str, value: f32, pct: f32, y: f32) -> f32 {
    let rotulo = format!("{} {pct:.0}%", tr(chave));
    let fim = c.full(Some(&rotulo), Widget::Readout { text: "" }, y);
    let r = Rect::new(c.x, fim - m::FIELD_H, c.w, m::FIELD_H);
    c.bar(
        &CardBar {
            id,
            value,
            vertical: false,
            ticks: &[],
        },
        r,
    );
    fim
}

/// O que `paint_layer_subtree` decide por camada.
#[derive(Copy, Clone)]
pub(crate) struct Lugar {
    pub(crate) depth: usize,
    pub(crate) base: bool,
    pub(crate) sobe: bool,
    pub(crate) desce: bool,
    pub(crate) ativa: bool,
    pub(crate) escolhida: bool,
    pub(crate) voa: bool,
}

/// A mistura (salvo na base, que não tem nada por baixo) e a opacidade, na mesma linha.
fn mistura_e_opacidade(c: &mut CardCtx<'_>, l: &Layer, base: bool, y: f32) -> f32 {
    let w = |k| painter_layer_widget_id(l.id.0, k);
    let op = w(PainterLayerWidget::Opacity);
    let pct = (l.opacity * PCT_SCALE).round();
    if base {
        return barra(
            c,
            op,
            "panel.painter_layers.line.opacity",
            l.opacity,
            pct,
            y,
        );
    }
    let (x, largura) = (c.x, c.w);
    let meia = ((largura - m::FIELD_GAP) * 0.5).max(0.0);
    c.w = meia;
    let nome = tr(BlendMode::from_u8(l.blend_mode.to_u8()).label_key());
    let fim = c.full(
        Some(tr("panel.painter_layers.brush.blend")),
        Widget::Dropdown {
            id: w(PainterLayerWidget::Blend),
            text: nome,
            options: &[],
        },
        y,
    );
    c.x = x + meia + m::FIELD_GAP;
    let fim2 = barra(
        c,
        op,
        "panel.painter_layers.line.opacity",
        l.opacity,
        pct,
        y,
    );
    c.x = x;
    c.w = largura;
    fim.max(fim2)
}

/// O relevo com sinal e como ele encontra o de baixo: `Add` ou `Level`.
fn relevo(c: &mut CardCtx<'_>, l: &Layer, y: f32) -> f32 {
    let w = |k| painter_layer_widget_id(l.id.0, k);
    let (x, largura) = (c.x, c.w);
    let meia = ((largura - m::FIELD_GAP) * 0.5).max(0.0);
    let nivela = matches!(l.impasto_composite, ReliefComposite::Level);
    let texto = if nivela {
        tr("panel.painter_layers.impasto.level")
    } else {
        tr("panel.painter_layers.impasto.add")
    };
    c.w = meia;
    let fim = barra(
        c,
        w(PainterLayerWidget::ImpastoDepth),
        "card.painter.layers.relief",
        fracao_do_relevo(l),
        (l.impasto_depth * PCT_SCALE).round(),
        y,
    );
    c.x = x + meia + m::FIELD_GAP;
    let fim2 = c.full(
        Some(tr("card.painter.layers.relief_mode")),
        Widget::Button {
            id: w(PainterLayerWidget::ImpastoLevel),
            text: texto,
            primary: nivela,
        },
        y,
    );
    c.x = x;
    c.w = largura;
    fim.max(fim2)
}

/// A sub-linha da máscara, um nível abaixo: nome, 👁, Inv, Apply.
pub(crate) fn mascara(
    c: &mut CardCtx<'_>,
    stack: &LayerStack,
    id: LayerId,
    depth: usize,
    y: f32,
) -> f32 {
    let invertida = matches!(
        stack.get(id).map(|l| &l.kind),
        Some(LayerKind::Mask(mk)) if mk.inverted
    );
    let w = |k| painter_layer_widget_id(id.0, k);
    let (x, largura) = (c.x, c.w);
    let dentro = recuo(depth + 1);
    let topo = y + m::FIELD_ROW_GAP;
    let visao = state::current_mask_grayscale_view() == Some(id.0);
    let texto = tr(if invertida {
        "panel.painter_layers.mask.mask_inverted"
    } else {
        "panel.painter_layers.mask.mask"
    });
    // O nome com a largura dele: o resto é de 👁, Inv e Apply, que têm de caber na mesma linha.
    let meia = c.text_system.prefix_width(texto, m::VALUE_PX) + 2.0 * m::SEG_INSET;
    c.x = x + dentro;
    c.w = meia;
    let mut nome = ListRow::new(w(PainterLayerWidget::Row), texto);
    nome.selected = stack.active() == Some(id);
    let fim = c.list_row(&nome, topo);
    let olho = [Seg {
        id: w(PainterLayerWidget::MaskView),
        icon: Some(if visao {
            IconId::Eye
        } else {
            IconId::EyeClosed
        }),
        text: "",
        on: false,
    }];
    c.x = x + dentro + meia + m::FIELD_GAP;
    c.w = (largura - dentro - meia - m::FIELD_GAP).max(0.0);
    let fim2 = c.row(
        &[
            Cell::new(Widget::Bare(&olho)),
            Cell::new(Widget::Button {
                id: w(PainterLayerWidget::MaskInvert),
                text: tr("panel.painter_layers.mask.inv"),
                primary: invertida,
            }),
            Cell::new(Widget::Button {
                id: w(PainterLayerWidget::MaskApply),
                text: tr("panel.painter_layers.mask.apply"),
                primary: false,
            }),
        ],
        topo - m::FIELD_ROW_GAP,
    );
    c.x = x;
    c.w = largura;
    fim.max(fim2)
}

/// Uma camada e o que mora debaixo dela. Devolve o fundo.
pub(crate) fn camada(c: &mut CardCtx<'_>, l: &Layer, lugar: Lugar, y: f32) -> f32 {
    let w = |k| painter_layer_widget_id(l.id.0, k);
    let (x0, w0) = (c.x, c.w);
    let recuado = recuo(lugar.depth);
    let topo = y + m::FIELD_ROW_GAP + m::FIELD_ROW_GAP;
    (c.x, c.w) = (x0 + recuado, (w0 - recuado).max(0.0));
    let (x, largura) = (c.x, c.w);
    let linha = Rect::new(x, topo, largura, m::FIELD_H);
    if lugar.ativa {
        fill_rounded_rect(
            c.scene,
            linha,
            m::SEG_RADIUS,
            c.ink.color(ColorToken::KanbanCardRaised),
        );
    } else if lugar.escolhida {
        fill_rounded_rect(
            c.scene,
            linha,
            m::SEG_RADIUS,
            c.ink.color(ColorToken::AccentSoft),
        );
    }
    let olho = [Seg {
        id: w(PainterLayerWidget::Visibility),
        icon: Some(if l.visible {
            IconId::Eye
        } else {
            IconId::EyeClosed
        }),
        text: "",
        on: false,
    }];
    let nome_id = w(PainterLayerWidget::Row);
    c.x = x + m::FIELD_H;
    c.w = (largura - m::FIELD_H * CELULAS_DE_ICONE).max(0.0);
    let mut nome = ListRow::new(nome_id, &l.name);
    nome.selected = lugar.ativa;
    nome.dim = lugar.voa || !l.visible;
    let mut y = c.list_row(&nome, topo);
    // A linha inteira agarra-se e larga-se: regista-se no fim, medida neste quadro, POR BAIXO dos filhos.
    let marca = c.hit_index.len();
    c.bare(Rect::new(x, topo, m::FIELD_H, m::FIELD_H), &olho);
    let fim = x + largura;
    seta(
        c,
        w(PainterLayerWidget::MoveUp),
        IconId::ChevronUp,
        Rect::new(fim - m::FIELD_H * 2.0, topo, m::FIELD_H, m::FIELD_H),
        lugar.sobe,
    );
    seta(
        c,
        w(PainterLayerWidget::MoveDown),
        IconId::ChevronDown,
        Rect::new(fim - m::FIELD_H, topo, m::FIELD_H, m::FIELD_H),
        lugar.desce,
    );
    (c.x, c.w) = (x, largura);
    y = mistura_e_opacidade(c, l, lugar.base, y);
    if l.has_relief {
        y = relevo(c, l, y);
    }
    c.hit_index
        .register_beneath(marca, nome_id, Rect::new(x, topo, largura, y - topo));
    (c.x, c.w) = (x0, w0);
    y
}
