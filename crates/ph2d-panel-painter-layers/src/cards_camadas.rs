//! O cartão Layers em linhas de cartão (fatia H9, `docs/CartoesUI/13_PAINTER_PLANO.md`): as duas barras de verbos e, por
//! camada, 👁 nome ↑ ↓, a mistura com a opacidade, o relevo e a máscara. Mesmos ids, mesmos eventos. A linha que voa é a
//! da lista do Kanban; a largada continua a ser a do despacho (30/40/30 sobre o rect da linha). Capturados, como peças:
//! os parâmetros de ajuste (curva, degradê, Levels) e o editor da camada Texture.

use crate::cards_captura::{Captura, chave_da_peca, x_de, y_de};
use crate::cards_linhas::{aberta, registra_lista};
use crate::paint_rows::{PCT_SCALE, register_bare_slider};
use crate::{paint, state};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::{HierarchyDragState, InteractiveState, PanelRowFamily};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::linhas_extra::{CardBar, ListRow};
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::ids::{PainterLayerWidget, painter_layer_widget_id};
use ph2d_tool_painter::{BlendMode, Layer, LayerId, LayerKind, LayerStack, ReliefComposite};
use std::collections::{BTreeMap, BTreeSet};

const SECAO: NodeId = pids::PAINTER_CARD_CAMADAS;
/// A faixa de cada banda da largada: a de `find_painter_layer_drop`.
const BANDA_DE_CIMA: f32 = 0.3; // LITERAL-PX-OK: fração da banda, não é medida de UI
const BANDA_DE_BAIXO: f32 = 0.7; // LITERAL-PX-OK: fração da banda, não é medida de UI
/// O relevo mora num número com sinal: `-1..1`.
const FUNDO_DO_RELEVO: f32 = -1.0; // LITERAL-PX-OK: profundidade com sinal, não é medida de UI
const AMPLITUDE_DO_RELEVO: f32 = 2.0; // LITERAL-PX-OK: profundidade com sinal, não é medida de UI

/// O que ainda pede o `PaintCtx`, por camada.
pub(crate) type Pecas = BTreeMap<u64, Captura>;

/// A altura de cada camada no último quadro: o rect da linha, que é o que o despacho lê, tem de ser o da linha inteira.
type Alturas = BTreeMap<u64, f32>;

thread_local! {
    static ALTURAS: std::cell::RefCell<Alturas> = const { std::cell::RefCell::new(BTreeMap::new()) };
}

fn recuo(depth: usize) -> f32 {
    depth as f32 * ph2d_tokens::list_indent_px()
}

/// Corre `f` em cada camada à vista, com a profundidade.
fn percorre(
    stack: &LayerStack,
    ids: &[LayerId],
    depth: usize,
    f: &mut dyn FnMut(LayerId, &Layer, usize, usize),
) {
    for (i, id) in ids.iter().enumerate() {
        let Some(l) = stack.get(*id) else { continue };
        f(*id, l, depth, i);
        if let LayerKind::Group(g) = &l.kind
            && !g.collapsed
        {
            percorre(stack, &g.children, depth + 1, f);
        }
    }
}

/// Corre os pintores que pedem o `PaintCtx`, antes da lista.
pub(crate) fn pecas(ctx: &mut PaintCtx, ink: CardInk, (x, w): (f32, f32)) -> Pecas {
    let Some(stack) = state::current_layers().filter(|s| !s.is_empty()) else {
        return BTreeMap::new();
    };
    let mut pedidos: Vec<(u64, LayerId, usize)> = Vec::new();
    percorre(&stack, stack.root(), 0, &mut |id, l, depth, _| {
        let tem = match &l.kind {
            LayerKind::Adjustment(_) => true,
            LayerKind::Texture(_) => stack.active() == Some(id),
            _ => false,
        };
        if tem {
            pedidos.push((id.0, id, depth));
        }
    });
    let topo = ctx.slot.y;
    let mut fora = BTreeMap::new();
    for (n, id, depth) in pedidos {
        let Some(l) = stack.get(id) else { continue };
        let chave = chave_da_peca(SECAO, n);
        let (px, tema) = (x_de(chave, x + recuo(depth + 1)), ink.theme);
        let pw = (w - recuo(depth + 1)).max(0.0);
        let captura = Captura::da_peca(
            ctx,
            chave,
            SECAO,
            ink,
            (px, y_de(chave, topo)),
            |ctx, y| match &l.kind {
                LayerKind::Adjustment(adj) => crate::paint_adjust::paint_adjustment_params(
                    ctx,
                    tema,
                    id.0,
                    &adj.params,
                    px,
                    pw,
                    y,
                ),
                LayerKind::Texture(tex) => {
                    crate::paint_texture::paint_texture_layer_editor(ctx, tema, tex, px, pw, y)
                }
                _ => y,
            },
        );
        fora.insert(n, captura);
    }
    fora
}

/// Com o store MUTÁVEL, antes de pintar: as linhas, as listas e as barras têm estado, e os ícones têm dica.
pub(crate) fn prepare(ctx: &mut PaintCtx) {
    let store = ctx.host.store_mut();
    for (id, chave) in VERBOS {
        paint::register_button(store, id);
        store.set_tooltip(id, tr(chave));
    }
    store.set_tooltip(
        pids::PAINTER_LAYERS_ADD_ADJUSTMENT,
        tr("panel.painter_layers.layers.add_adjustment"),
    );
    let Some(stack) = state::current_layers() else {
        return;
    };
    for id in stack.all_ids() {
        let Some(l) = stack.get(id) else { continue };
        let w = |k| painter_layer_widget_id(id.0, k);
        for k in [
            PainterLayerWidget::Row,
            PainterLayerWidget::Visibility,
            PainterLayerWidget::MoveUp,
            PainterLayerWidget::MoveDown,
            PainterLayerWidget::MaskView,
            PainterLayerWidget::MaskInvert,
            PainterLayerWidget::MaskApply,
            PainterLayerWidget::ImpastoLevel,
        ] {
            paint::register_button(store, w(k));
        }
        registra_lista(
            store,
            w(PainterLayerWidget::Blend),
            usize::from(l.blend_mode.to_u8()),
        );
        register_bare_slider(store, w(PainterLayerWidget::Opacity), l.opacity);
        if l.has_relief {
            register_bare_slider(
                store,
                w(PainterLayerWidget::ImpastoDepth),
                fracao_do_relevo(l),
            );
        }
    }
}

fn fracao_do_relevo(l: &Layer) -> f32 {
    (l.impasto_depth - FUNDO_DO_RELEVO) / AMPLITUDE_DO_RELEVO
}

/// Depois da lista: uma lista aberta de cada vez, e a do `+ Adj` abre com a largura do cartão.
pub(crate) fn listas_abertas(ctx: &mut PaintCtx, (x, w): (f32, f32)) {
    let adj = pids::PAINTER_LAYERS_ADD_ADJUSTMENT;
    if aberta(ctx.host.store(), adj)
        && let Some(r) = ctx.host.hit_index_mut().rect_for(adj)
    {
        state::set_pending_adj_menu(Some(Rect::new(x, r.y, w, r.h)));
    }
    let Some(stack) = state::current_layers() else {
        return;
    };
    let mut achou = false;
    for id in stack.all_ids() {
        let Some(l) = stack.get(id) else { continue };
        let caixa = painter_layer_widget_id(id.0, PainterLayerWidget::Blend);
        if !aberta(ctx.host.store(), caixa) {
            continue;
        }
        if achou {
            if let Some(InteractiveState::Dropdown { open, .. }) =
                ctx.host.store_mut().get_mut(caixa)
            {
                *open = false;
            }
            continue;
        }
        if let Some(r) = ctx.host.hit_index_mut().rect_for(caixa) {
            state::set_pending_blend_dd(Some((id.0, r, l.blend_mode.to_u8())));
            achou = true;
        }
    }
}

/// As linhas que se arrastam: as máscaras selecionam-se, mas não voam.
pub(crate) fn linhas_arrastaveis() -> BTreeSet<NodeId> {
    state::current_layers().map_or_else(BTreeSet::new, |s| {
        s.all_ids()
            .filter(|l| !s.is_mask(*l))
            .map(|l| painter_layer_widget_id(l.0, PainterLayerWidget::Row))
            .collect()
    })
}

const VERBOS: [(NodeId, &str); 5] = [
    (
        pids::PAINTER_LAYERS_ADD,
        "panel.painter_layers.layers.new_layer",
    ),
    (
        pids::PAINTER_LAYERS_GROUP,
        "panel.painter_layers.layers.new_group",
    ),
    (
        pids::PAINTER_LAYERS_DUPLICATE,
        "panel.painter_layers.layers.duplicate_layer",
    ),
    (
        pids::PAINTER_LAYERS_DELETE,
        "panel.painter_layers.layers.delete_layer",
    ),
    (
        pids::PAINTER_LAYERS_ADD_TEXTURE,
        "panel.painter_layers.layers.add_texture_layer",
    ),
];

fn verbos(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let icone = |id, icon, on| Seg {
        id,
        icon: Some(icon),
        text: "",
        on,
    };
    let adj = pids::PAINTER_LAYERS_ADD_ADJUSTMENT;
    let segs = [
        icone(pids::PAINTER_LAYERS_ADD, IconId::Add, false),
        icone(pids::PAINTER_LAYERS_GROUP, IconId::Group, false),
        icone(pids::PAINTER_LAYERS_DUPLICATE, IconId::Duplicate, false),
        icone(pids::PAINTER_LAYERS_DELETE, IconId::Trash, false),
        icone(adj, IconId::ColorEqualization, aberta(c.store, adj)),
        icone(pids::PAINTER_LAYERS_ADD_TEXTURE, IconId::Grid, false),
    ];
    c.row(&[Cell::new(Widget::Bare(&segs))], y)
}

/// Mask, Clip, Lock e Ref, sobre a camada ativa; o que ela não aceita fica apagado e sem alvo.
fn modificadores(c: &mut CardCtx<'_>, stack: &LayerStack, y: f32) -> f32 {
    let mods = stack.active_modifiers();
    let raster = mods.is_some_and(|m| m.is_raster);
    let textura = stack.active().is_some_and(|a| stack.is_texture(a));
    let cell = |id, chave, on: bool, textura_aceita: bool| {
        let text = tr(chave);
        Cell::new(if raster || (textura && textura_aceita) {
            Widget::Button {
                id,
                text,
                primary: on,
            }
        } else {
            Widget::ButtonOff { text }
        })
    };
    c.row(
        &[
            cell(
                pids::PAINTER_LAYERS_MASK,
                "panel.painter_layers.layers.mask",
                mods.is_some_and(|m| m.has_mask),
                true,
            ),
            cell(
                pids::PAINTER_LAYERS_CLIP,
                "panel.painter_layers.layers.clip",
                mods.is_some_and(|m| m.clipping),
                true,
            ),
            cell(
                pids::PAINTER_LAYERS_ALPHA_LOCK,
                "panel.painter_layers.layers.lock",
                mods.is_some_and(|m| m.alpha_locked),
                false,
            ),
            cell(
                pids::PAINTER_LAYERS_REFERENCE,
                "panel.painter_layers.layers.ref",
                mods.is_some_and(|m| m.is_reference),
                false,
            ),
        ],
        y,
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
struct Lugar {
    depth: usize,
    base: bool,
    sobe: bool,
    desce: bool,
    ativa: bool,
    escolhida: bool,
    voa: bool,
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
fn mascara(c: &mut CardCtx<'_>, stack: &LayerStack, id: LayerId, depth: usize, y: f32) -> f32 {
    let invertida = matches!(
        stack.get(id).map(|l| &l.kind),
        Some(LayerKind::Mask(mk)) if mk.inverted
    );
    let w = |k| painter_layer_widget_id(id.0, k);
    let (x, largura) = (c.x, c.w);
    let dentro = recuo(depth + 1);
    let topo = y + m::FIELD_ROW_GAP;
    let visao = state::current_mask_grayscale_view() == Some(id.0);
    let meia = ((largura - dentro - m::FIELD_GAP) * 0.5).max(0.0);
    c.x = x + dentro;
    c.w = meia;
    let mut nome = ListRow::new(
        w(PainterLayerWidget::Row),
        tr(if invertida {
            "panel.painter_layers.mask.mask_inverted"
        } else {
            "panel.painter_layers.mask.mask"
        }),
    );
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
fn camada(c: &mut CardCtx<'_>, l: &Layer, lugar: Lugar, y: f32) -> f32 {
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
    c.w = (largura - m::FIELD_H * 3.0).max(0.0);
    let mut nome = ListRow::new(nome_id, &l.name);
    nome.selected = lugar.ativa;
    nome.dim = lugar.voa || !l.visible;
    let mut y = c.list_row(&nome, topo);
    // A linha inteira agarra-se e larga-se: o que se regista depois dela ganha a sua célula.
    let alta = ALTURAS.with(|a| a.borrow().get(&l.id.0).copied().unwrap_or(m::FIELD_H));
    c.hit_index
        .register(nome_id, Rect::new(x, topo, largura, alta));
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
    ALTURAS.with(|a| a.borrow_mut().insert(l.id.0, y - topo));
    (c.x, c.w) = (x0, w0);
    y
}

/// A marca de onde a camada cairia, com as mesmas bandas do despacho.
fn marca_da_largada(c: &mut CardCtx<'_>, linhas: &[(NodeId, Rect, bool)], d: HierarchyDragState) {
    let barra = |c: &mut CardCtx<'_>, y: f32| {
        let esp = StrokeToken::Thick.px();
        let r = Rect::new(c.x, y - esp * 0.5, c.w, esp);
        let cor = c.ink.color(ColorToken::Accent);
        fill_rounded_rect(c.scene, r, esp * 0.5, cor);
    };
    for (id, r, grupo) in linhas {
        if *id == d.dragged || d.cursor_y < r.y || d.cursor_y >= r.y + r.h {
            continue;
        }
        let (cima, baixo) = (r.y + r.h * BANDA_DE_CIMA, r.y + r.h * BANDA_DE_BAIXO);
        if d.cursor_y >= baixo {
            barra(c, r.y + r.h);
        } else if d.cursor_y >= cima && *grupo {
            let cor = c.ink.color(ColorToken::Accent);
            // FRAME-RAW-OK: o alvo «largar DENTRO deste grupo» durante um arrasto, a mensagem
            stroke_rounded_rect(c.scene, *r, m::SEG_RADIUS, StrokeToken::Thick.px(), cor);
        } else {
            barra(c, r.y);
        }
        return;
    }
    if let Some((_, base, _)) = linhas.last() {
        barra(c, base.y);
    }
}

/// O corpo inteiro. A camada mais de baixo na raiz é a própria imagem: não tem mistura e não sai do lugar.
pub(crate) fn corpo(c: &mut CardCtx<'_>, pecas: &Pecas, y: f32) -> f32 {
    let Some(stack) = state::current_layers().filter(|s| !s.is_empty()) else {
        let y = verbos(c, y);
        return c.hint(tr("panel.painter_layers.layers.no_layers"), y);
    };
    let mut y = verbos(c, y);
    y = modificadores(c, &stack, y);
    let selecao = state::current_selection();
    let ativa = stack.active();
    let na_mao = c
        .store
        .panel_row_drag()
        .filter(|(f, d)| *f == PanelRowFamily::PainterLayer && d.active)
        .map(|(_, d)| d);
    let mut linhas: Vec<(NodeId, Rect, bool)> = Vec::new();
    subarvore(
        c,
        &stack,
        stack.root(),
        0,
        pecas,
        (ativa, &selecao, na_mao),
        &mut linhas,
        &mut y,
    );
    if let Some(d) = na_mao {
        marca_da_largada(c, &linhas, d);
        let nome = stack
            .all_ids()
            .find(|l| painter_layer_widget_id(l.0, PainterLayerWidget::Row) == d.dragged)
            .and_then(|l| stack.get(l));
        if let Some(l) = nome {
            let voa = ListRow::new(d.dragged, &l.name);
            c.list_row_flight(
                &voa,
                d.cursor_x - (d.down_x - c.x),
                d.cursor_y - m::FIELD_H * 0.5,
            );
        }
    }
    c.row(
        &[Cell::new(Widget::Button {
            id: pids::PAINTER_APPLY,
            text: tr("panel.painter_layers.layers.apply"),
            primary: true,
        })],
        y,
    )
}

type Vista<'a> = (
    Option<LayerId>,
    &'a BTreeSet<LayerId>,
    Option<HierarchyDragState>,
);

#[allow(clippy::too_many_arguments)]
fn subarvore(
    c: &mut CardCtx<'_>,
    stack: &LayerStack,
    ids: &[LayerId],
    depth: usize,
    pecas: &Pecas,
    vista: Vista<'_>,
    linhas: &mut Vec<(NodeId, Rect, bool)>,
    y: &mut f32,
) {
    let (ativa, selecao, na_mao) = vista;
    let ultimo = ids.len().saturating_sub(1);
    for (i, &id) in ids.iter().enumerate() {
        let Some(l) = stack.get(id) else { continue };
        let base = depth == 0 && i == ultimo;
        let lugar = Lugar {
            depth,
            base,
            sobe: !base && i > 0,
            desce: if depth == 0 {
                !base && i + 1 < ultimo
            } else {
                i < ultimo
            },
            ativa: ativa == Some(id),
            escolhida: selecao.contains(&id),
            voa: na_mao.is_some_and(|d| {
                d.dragged == painter_layer_widget_id(id.0, PainterLayerWidget::Row)
            }),
        };
        let topo = *y + m::FIELD_ROW_GAP + m::FIELD_ROW_GAP;
        *y = camada(c, l, lugar, *y);
        if na_mao.is_some() {
            let dentro = recuo(depth);
            linhas.push((
                painter_layer_widget_id(id.0, PainterLayerWidget::Row),
                Rect::new(c.x + dentro, topo, (c.w - dentro).max(0.0), *y - topo),
                l.is_group(),
            ));
        }
        if let Some(p) = pecas.get(&id.0) {
            *y = p.repete(c, *y + m::FIELD_ROW_GAP);
        }
        if let Some(mk) = l.mask {
            *y = mascara(c, stack, mk, depth, *y);
        }
        if let LayerKind::Group(g) = &l.kind
            && !g.collapsed
        {
            subarvore(c, stack, &g.children, depth + 1, pecas, vista, linhas, y);
        }
    }
}
