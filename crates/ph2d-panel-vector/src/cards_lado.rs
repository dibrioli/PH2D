//! ⭐ **Um painel à ESQUERDA da lista, em cartão** (F44 + fatia G2): o lugar, a altura, a rolagem e o
//! `Dropdown` que o mantém aberto são os de [`crate::fx_side::side_panel`]; a moldura e a tinta são as
//! de um cartão solto, e o corpo pinta-se num [`CardCtx`].

use crate::fx_side::side_rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, rect_to_vello};
use ph2d_editor_core::panel::cards::{CardCtx, censo, dicas};
use ph2d_editor_core::panel::{PaintCtx, camada_das_listas};
use ph2d_editor_core::widget::PaintedPopover;
use ph2d_editor_core::widget::card_ink::{self, CardInk};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::{Affine, VectorScene};

/// O cabeçalho: o nome, e da direita ✕ · ↓ · ↑ (as setas só onde mexem).
pub(crate) struct Cabeca<'a> {
    /// O glifo do tipo (F43: o efeito ou filtro aberto), à esquerda do nome.
    pub glifo: Option<IconId>,
    pub titulo: &'a str,
    pub fecha: NodeId,
    pub desce: Option<NodeId>,
    pub sobe: Option<NodeId>,
}

fn sombra(scene: &mut VectorScene, r: Rect, cor: ph2d_vector::Color, (dy, blur): (f64, f64)) {
    let rr = ph2d_vector::Rect::new(
        f64::from(r.x),
        f64::from(r.y) + dy,
        f64::from(r.x + r.w),
        f64::from(r.y + r.h) + dy,
    );
    scene.blurred_rounded_rect(
        Affine::IDENTITY,
        rr,
        cor,
        f64::from(m::CARD_RADIUS),
        blur / 2.0,
    );
}

/// A moldura de um cartão que flutua: as duas sombras e o fundo do solto.
fn moldura(scene: &mut VectorScene, ink: CardInk, r: Rect) {
    sombra(
        scene,
        r,
        ink.color(ColorToken::KanbanShadowFlight),
        m::LOOSE_SHADOW_FAR,
    );
    sombra(
        scene,
        r,
        ink.color(ColorToken::KanbanFlightNear),
        m::LOOSE_SHADOW_NEAR,
    );
    fill_rounded_rect(scene, r, m::CARD_RADIUS, ink.color(ColorToken::KanbanCard));
}

/// Um ícone do cabeçalho: a mesma caixa, o mesmo hover e o mesmo glifo do cabeçalho de um cartão.
fn icone(c: &mut CardCtx<'_>, id: NodeId, icon: IconId, b: Rect) {
    c.hit_index.register(id, b);
    dicas::icone(id);
    if c.store.hover_live(id) > 0.0 {
        fill_rounded_rect(
            c.scene,
            b,
            m::HEAD_ICON_RADIUS,
            c.ink.color(ColorToken::CardFieldHover),
        );
    }
    let g = m::HEAD_ICON_GLYPH;
    paint_icon(
        c.scene,
        icon,
        Rect::new(b.x + (b.w - g) * 0.5, b.y + (b.h - g) * 0.5, g, g),
        c.ink.color(ColorToken::Text2),
        StrokeToken::Default.px(),
    );
}

/// Pinta o cabeçalho em `y` (o topo do cartão) e devolve onde o corpo começa.
fn cabecalho(c: &mut CardCtx<'_>, h: &Cabeca<'_>, y: f32) -> f32 {
    let hy = y + m::CARD_PAD_TOP;
    let by = hy + (m::CARD_HEAD_H - m::HEAD_ICON_BOX) * 0.5;
    let mut ax = c.x + c.w;
    let caixa = |ax: &mut f32| {
        *ax -= m::HEAD_ICON_BOX;
        Rect::new(*ax, by, m::HEAD_ICON_BOX, m::HEAD_ICON_BOX)
    };
    let fecho = caixa(&mut ax);
    icone(c, h.fecha, IconId::Close, fecho);
    if let Some(id) = h.desce {
        let b = caixa(&mut ax);
        icone(c, id, IconId::ChevronDown, b);
    }
    if let Some(id) = h.sobe {
        let b = caixa(&mut ax);
        icone(c, id, IconId::ChevronUp, b);
    }
    let mut tx = c.x;
    if let Some(glifo) = h.glifo {
        let g = m::ICON_GLYPH;
        paint_icon(
            c.scene,
            glifo,
            Rect::new(tx, hy + (m::CARD_HEAD_H - g) * 0.5, g, g),
            c.ink.color(ColorToken::Text1),
            StrokeToken::Default.px(),
        );
        tx += g + m::SEG_ICON_GAP;
    }
    ph2d_editor_core::paint::paint_text_title_elided_tracked(
        c.text_system,
        c.scene,
        h.titulo,
        [
            tx,
            hy + ph2d_editor_core::paint::folga_do_texto(m::CARD_HEAD_H, m::CARD_TITLE_PX),
        ],
        m::CARD_TITLE_PX,
        (ax - tx).max(0.0),
        c.ink.color(ColorToken::Text1),
        0.0,
    );
    // O corpo já não abre o cartão: a 1.ª linha sem rótulo depois de uma escolha leva o vão.
    c.legacy(hy + m::CARD_HEAD_H + m::CARD_HEAD_BOTTOM, |_, y| y)
}

/// ⭐ **O painel**, no topo alinhado à `line` que o abriu, com a tinta do cartão `seccao`. `id` é o
/// `Dropdown` que o mantém aberto; `corpo` pinta a partir do `y` e devolve o fundo.
#[allow(clippy::too_many_arguments)]
pub(crate) fn lado(
    ctx: &mut PaintCtx,
    id: NodeId,
    seccao: NodeId,
    line: Rect,
    panel: Rect,
    theme: Theme,
    cabeca: &Cabeca<'_>,
    corpo: &mut dyn FnMut(&mut CardCtx<'_>, f32) -> f32,
) {
    let region = ctx.layout.popover_region();
    flutua(
        ctx,
        id,
        seccao,
        panel.w,
        theme,
        &|conteudo| side_rect(panel, line, conteudo, region),
        Some(cabeca),
        corpo,
    );
}

/// ⭐ **Um cartão que flutua**, de largura `largura`, no rect que `onde` dá para a altura do conteúdo:
/// encolhe e rola. Sem `cabeca` o corpo abre logo sob a folga do cartão (o painel do traço da barra).
#[allow(clippy::too_many_arguments)]
pub(crate) fn flutua(
    ctx: &mut PaintCtx,
    id: NodeId,
    seccao: NodeId,
    largura: f32,
    theme: Theme,
    onde: &dyn Fn(f32) -> Rect,
    cabeca: Option<&Cabeca<'_>>,
    corpo: &mut dyn FnMut(&mut CardCtx<'_>, f32) -> f32,
) {
    let ink = CardInk::of_card(ctx.host.store().section_theme(seccao), theme);
    let mut inner_w = (largura - m::CARD_PAD_X * 2.0).max(0.0);
    let abre = |c: &mut CardCtx<'_>, y: f32| match cabeca {
        Some(h) => cabecalho(c, h, y),
        None => y + m::CARD_PAD_TOP,
    };
    // A altura medida numa passagem a seco: o fundo pinta-se ANTES.
    let mut mede = |ctx: &mut PaintCtx, util: f32| {
        let mut scene = VectorScene::new();
        let mut hits = HitIndex::new();
        let mut c = CardCtx::flutuante(
            &mut scene,
            ctx.text_system,
            ctx.host.store(),
            &mut hits,
            ink,
            m::CARD_PAD_X,
            util,
        );
        let medida = card_ink::com(ink, || {
            let y = abre(&mut c, 0.0);
            corpo(&mut c, y)
        });
        medida + m::CARD_PAD_BOTTOM
    };
    let mut conteudo = mede(ctx, inner_w);
    let mut rect = onde(conteudo);
    // A barra de rolagem tem faixa própria: quando ela aparece, as linhas encolhem para não colar nela.
    if conteudo > rect.h {
        let faixa = ph2d_editor_core::widget::scrollbar_track_rect(rect).x;
        inner_w = (faixa - ph2d_tokens::Spacing::Xs.px() - rect.x - m::CARD_PAD_X).max(0.0);
        conteudo = mede(ctx, inner_w);
        rect = onde(conteudo);
    }
    let max = (conteudo - rect.h).max(0.0);
    let rolagem = ctx.host.store().panel_scroll(id).clamp(0.0, max); // CLAMP-OK: 0.0 literal; max é uma extensão não negativa
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let marca = hit_index.len();
    let text_system = &mut *ctx.text_system;
    // Na camada das listas: depois dele vêm os outros painéis, os cartões soltos, a barra e o HUD.
    camada_das_listas::por_cima(ctx.scene, |scene| {
        moldura(scene, ink, rect);
        // O fundo ENGOLE o clique: sem ele um clique entre dois campos caía na arte por baixo.
        hit_index.register(ph2d_editor_core::ids::RAIL_BACKDROP, rect);
        scene.push_clip(&rect_to_vello(rect));
        hit_index.push_clip(rect);
        {
            let mut c = CardCtx::flutuante(
                &mut *scene,
                &mut *text_system,
                store,
                &mut *hit_index,
                ink,
                rect.x + m::CARD_PAD_X,
                inner_w,
            );
            card_ink::com(ink, || {
                let y = abre(&mut c, rect.y - rolagem);
                censo::na_secao_de(id, || corpo(&mut c, y));
            });
        }
        hit_index.pop_clip();
        scene.pop_layer();
    });
    camada_das_listas::alvos_desde(hit_index, marca);
    let pintada = PaintedPopover {
        id,
        panel: rect,
        content_h: conteudo,
        visible_h: rect.h,
        scroll: rolagem,
    };
    ph2d_editor_core::panel::paint_popover_scrollbar(&pintada, store, ctx.scene, theme, hit_index);
    ctx.host.store_mut().publish_dropdown_popover(pintada);
}
