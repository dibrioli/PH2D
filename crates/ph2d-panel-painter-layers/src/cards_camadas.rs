//! O cartão Layers em linhas de cartão (fatia H9, `docs/CartoesUI/13_PAINTER_PLANO.md`): as duas barras de verbos e, por
//! camada, 👁 nome ↑ ↓, a mistura com a opacidade, o relevo e a máscara. Mesmos ids, mesmos eventos. A linha que voa é a
//! da lista do Kanban; a largada continua a ser a do despacho (30/40/30 sobre o rect da linha). Debaixo da linha: os
//! parâmetros de ajuste (`cards_ajustes`) e o editor da camada Texture ativa (`cards_grao`).

use crate::cards_camadas_linha::{Lugar, camada, fracao_do_relevo, mascara, recuo};
use crate::cards_linhas::{aberta, registra_lista};
use crate::paint_rows::register_bare_slider;
use crate::paint_texture::{Previa, brush_view_from_texture_layer};
use crate::{cards_ajustes, cards_grao, paint, state};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::{HierarchyDragState, InteractiveState, PanelRowFamily};
use ph2d_editor_core::paint::{fill_rounded_rect, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::linhas_extra::ListRow;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::ids::{PainterLayerWidget, painter_layer_widget_id};
use ph2d_tool_painter::{BrushSettings, Layer, LayerId, LayerKind, LayerStack};
use std::collections::BTreeSet;

/// A faixa de cada banda da largada: a de `find_painter_layer_drop`.
const BANDA_DE_CIMA: f32 = 0.3; // LITERAL-PX-OK: fração da banda, não é medida de UI
const BANDA_DE_BAIXO: f32 = 0.7; // LITERAL-PX-OK: fração da banda, não é medida de UI
/// O que se desenha uma vez por quadro: a prévia da camada Texture ativa, a única com editor à vista.
pub(crate) type Pecas = Option<Previa>;

/// A camada Texture ativa vestida de pincel, e a lista Alpha dela. Sem pincel publicado, não há editor.
fn textura_ativa(stack: &LayerStack) -> Option<(BrushSettings, u8)> {
    match &stack.get(stack.active()?)?.kind {
        LayerKind::Texture(tex) => Some((
            brush_view_from_texture_layer(state::current_brush()?, tex),
            tex.ramp_alpha_mode,
        )),
        _ => None,
    }
}

/// A prévia, à largura da coluna.
pub(crate) fn pecas(w: f32) -> Pecas {
    let stack = state::current_layers()?;
    let (view, alpha) = textura_ativa(&stack)?;
    cards_grao::previa_da_camada(w, &view, alpha)
}

/// Com o store MUTÁVEL, antes de pintar: as linhas, as listas e as barras têm estado, e os ícones têm dica.
pub(crate) fn prepare(ctx: &mut PaintCtx) {
    if let Some(stack) = state::current_layers() {
        if let Some((view, alpha)) = textura_ativa(&stack) {
            cards_grao::prepara_da_camada(ctx, &view, alpha);
        }
        let store = ctx.host.store_mut();
        for id in stack.all_ids() {
            if let Some(LayerKind::Adjustment(adj)) = stack.get(id).map(|l| &l.kind) {
                cards_ajustes::prepare(store, id.0, &adj.params);
            }
        }
    }
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
    if let Some((view, alpha)) = textura_ativa(&stack) {
        cards_grao::listas_da_camada(ctx, &view, alpha);
    }
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
        let y = c.hint(tr("panel.painter_layers.layers.no_layers"), y);
        return aplicar(c, y);
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
    aplicar(c, y)
}

fn aplicar(c: &mut CardCtx<'_>, y: f32) -> f32 {
    c.row(
        &[Cell::new(Widget::Button {
            id: pids::PAINTER_APPLY,
            text: tr("panel.painter_layers.layers.apply"),
            primary: true,
        })],
        y,
    )
}

/// Os parâmetros que a camada tem, um nível abaixo dela.
fn debaixo_da_linha(
    c: &mut CardCtx<'_>,
    stack: &LayerStack,
    id: LayerId,
    l: &Layer,
    depth: usize,
    previa: &Pecas,
    y: f32,
) -> f32 {
    let (x, w) = (c.x, c.w);
    let dentro = recuo(depth + 1);
    (c.x, c.w) = (x + dentro, (w - dentro).max(0.0));
    let fim = match &l.kind {
        LayerKind::Adjustment(adj) => cards_ajustes::corpo(c, id.0, &adj.params, y),
        LayerKind::Texture(_) if stack.active() == Some(id) => match textura_ativa(stack) {
            Some((view, alpha)) => cards_grao::corpo_da_camada(c, &view, alpha, previa.as_ref(), y),
            None => y,
        },
        _ => y,
    };
    (c.x, c.w) = (x, w);
    fim
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
        *y = debaixo_da_linha(c, stack, id, l, depth, pecas, *y);
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
