//! ⭐⭐ **O painel de propriedades à ESQUERDA da coluna e a lista do «+»** (dono 01/11, F43: *«dropdown,
//! lista de efeitos, Propriedade dos efeitos abre na lateral esquerda»*). A geometria é a do
//! `side_panel` do `ph2d-panel-vector` (à esquerda, alinhado à linha que o abriu; com a coluna colada à
//! esquerda, à direita dela); a crate do Vetor não é dependência, e o corpo aqui fala a lei do cartão.
//! Pintados DEPOIS da lista: por cima dos cartões e fora do recorte da rolagem.

use crate::cards_pintura::fx_props;
use crate::cards_reais_media::rack_props;
use crate::controlos::{dropdown, find, seg_of, segs};
use crate::ids as g;
use crate::state::{Choices, FXS, Pick, RACKS, SIDE_NONE, WidgetGalleryState};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Widget, dropdown as lista};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

/// A folga entre a coluna e o painel do lado.
const GAP: f32 = m::CARD_GAP;

/// O título (a lista do tipo + ✕) e os campos da linha aberta `i` (`0..3` Effects, `3..6` o rack).
fn body(c: &mut CardCtx<'_>, ch: &Choices, i: usize, y: f32) -> f32 {
    let (kind, options) = if i < 3 {
        (g::EFFECTS[i].kind, g::EFFECTS[i].options)
    } else {
        (g::RACK[i - 3].kind, g::RACK[i - 3].options)
    };
    let tipos = options.map(|id| seg_of(id, ch, c.store));
    let fecha = segs([g::SIDE_CLOSE], ch, c.store);
    let y = c.row(
        &[
            Cell::new(dropdown(kind, &tipos)),
            Cell::new(Widget::Bare(&fecha)),
        ],
        y,
    );
    if i < 3 {
        fx_props(c, ch, i, y)
    } else {
        rack_props(c, i - 3, y)
    }
}

/// O painel da linha aberta. Uma linha que sumiu (o «−») fecha-o.
pub(crate) fn paint(state: &mut WidgetGalleryState, ctx: &mut PaintCtx, panel: Rect, theme: Theme) {
    let aberta = state.choices.pick(Pick::SideOpen);
    if aberta == SIDE_NONE {
        return;
    }
    let i = usize::from(aberta);
    let (presente, abre, section) = if i < 3 {
        (FXS[i], g::EFFECTS[i].open, g::CARD_EFFECTS)
    } else {
        (RACKS[i - 3], g::RACK[i - 3].open, g::CARD_AUDIO)
    };
    if !state.choices.on(presente) {
        state.choices.choose(Pick::SideOpen, SIDE_NONE);
        return;
    }
    let ch = state.choices;
    let Some(line) = ctx.host.hit_index_mut().rect_for(abre) else {
        return;
    };
    let ink = CardInk::of_card(ctx.host.store().section_theme(section), theme);
    let w = panel.w - m::LIST_PAD_X * 2.0;
    let inner = w - m::CARD_PAD_X * 2.0;
    // A altura numa passagem a seco: o fundo pinta-se ANTES dos campos.
    let alto = {
        let mut cena = VectorScene::new();
        let mut hits = HitIndex::new();
        let mut c = CardCtx::flutuante(
            &mut cena,
            ctx.text_system,
            ctx.host.store(),
            &mut hits,
            ink,
            0.0,
            inner,
        );
        body(&mut c, &ch, i, 0.0)
    };
    let h = alto + m::CARD_PAD_TOP + m::CARD_PAD_BOTTOM;
    let region = ctx.layout.popover_region();
    let esquerda = panel.x - GAP - w;
    let x = if esquerda >= region.x {
        esquerda
    } else {
        panel.x + panel.w + GAP
    };
    let y = line.y.min(region.y + region.h - h).max(region.y);
    let r = Rect::new(x, y, w, h);
    fundo(ctx.scene, r, &ink);
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    // O fundo ENGOLE o clique: sem ele, um clique entre dois campos caía na arte por baixo.
    hit_index.register(ph2d_editor_core::ids::RAIL_BACKDROP, r);
    {
        let mut c = CardCtx::flutuante(
            ctx.scene,
            ctx.text_system,
            store,
            hit_index,
            ink,
            r.x + m::CARD_PAD_X,
            inner,
        );
        body(&mut c, &ch, i, r.y + m::CARD_PAD_TOP);
    }
    lista::paint_pending(ctx.scene, ctx.text_system, hit_index, store);
}

fn fundo(scene: &mut VectorScene, r: Rect, ink: &CardInk) {
    fill_rounded_rect(
        scene,
        r,
        m::CARD_RADIUS,
        ink.color(ColorToken::KanbanCardRaised),
    );
}

/// As listas que abrem de um ícone de cabeçalho: `(o «+», a secção, as opções)`.
const MENUS: &[(NodeId, NodeId, &[NodeId])] = &[
    (g::FX_ADD, g::CARD_EFFECTS, &g::FX_ADD_KINDS),
    (
        g::AU_ADD,
        g::CARD_AUDIO,
        &[g::AU_ADD_0, g::AU_ADD_1, g::AU_ADD_2, g::AU_ADD_3],
    ),
];

/// A lista do «+» aberto, com a largura da coluna do cartão, por baixo do ícone.
pub(crate) fn head_menus(ctx: &mut PaintCtx, panel: Rect, theme: Theme) {
    let Some(aberta) = lista::open() else {
        return;
    };
    let Some((add, section, opcoes)) = MENUS.iter().find(|(a, ..)| *a == aberta) else {
        return;
    };
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let Some(r) = hit_index.rect_for(*add) else {
        return;
    };
    let ink = CardInk::of_card(store.section_theme(*section), theme);
    let linhas = opcoes
        .iter()
        .filter_map(|id| find(*id))
        .map(|c| (c.id, c.text.map_or("", tr).to_string(), false))
        .collect();
    let dentro = m::LIST_PAD_X + m::CARD_PAD_X;
    let caixa = Rect::new(panel.x + dentro, r.y, panel.w - dentro * 2.0, r.h);
    lista::stash(caixa, ink, linhas);
    lista::paint_pending(ctx.scene, ctx.text_system, hit_index, store);
}
