//! ⭐⭐ **A galeria como LISTA DE CARTÕES no painel lateral** (doc `CartoesUI/08` §1): o
//! inspector de ferramentas fictícias primeiro ([`crate::cards_vocab`]), depois as onze secções de
//! sempre, cada uma o corpo de um cartão com nome de ferramenta, pintado com a tinta dele.

use crate::ids as g;
use crate::state::WidgetGalleryState;
use crate::{cards_avancado, cards_render, cards_vocab};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, paint_card_list_with};
use ph2d_editor_core::widget::card_ink::{self, CardInk};
use ph2d_editor_core::widget::showcase::{SHOWCASE_CARDS, paint_showcase_popovers};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;

thread_local! {
    static OVERRIDE: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// A galeria pinta-se em cartões? (`PH2D_CARDS=0` = a janela flutuante de hoje.)
#[must_use]
pub fn cards_on() -> bool {
    OVERRIDE
        .with(std::cell::Cell::get)
        .unwrap_or_else(|| std::env::var("PH2D_CARDS").as_deref() != Ok("0"))
}

/// Força o modo NESTA thread (os testes de um processo correm em paralelo: a variável de ambiente
/// seria partilhada entre eles). `None` volta a ler o ambiente.
#[doc(hidden)]
pub fn force_cards_on_this_thread(on: Option<bool>) {
    OVERRIDE.with(|o| o.set(on));
}

/// O nome de ferramenta de cada corpo de sempre (a tabela widget → ferramenta do doc 08 §1.2).
const TOOL_TITLES: [(NodeId, &str); 11] = [
    (
        ids::INSP_SECTION_INPUTS,
        "panel.widget_gallery.card.layer_info",
    ),
    (ids::INSP_SECTION_SLIDER, "panel.widget_gallery.card.motion"),
    (
        ids::INSP_SECTION_SWITCHES,
        "panel.widget_gallery.card.preferences",
    ),
    (ids::INSP_SECTION_LISTS, "panel.widget_gallery.card.browser"),
    (ids::INSP_SECTION_VECTOR, "panel.widget_gallery.card.pivot"),
    (ids::INSP_SECTION_STATUS, "panel.widget_gallery.card.render"),
    (ids::INSP_SECTION_COLOR, "panel.widget_gallery.card.tint"),
    (
        ids::INSP_SECTION_ACTIONS,
        "panel.widget_gallery.card.history",
    ),
    (
        ids::INSP_SECTION_IDENTITY,
        "panel.widget_gallery.card.collaborators",
    ),
    (ids::INSP_SECTION_CARD, "panel.widget_gallery.card.comments"),
    (ids::INSP_SECTION_W6, "panel.widget_gallery.card.advanced"),
];

fn tool_title(section: NodeId) -> &'static str {
    TOOL_TITLES
        .iter()
        .find(|(s, _)| *s == section)
        .map_or("", |(_, key)| tr(key))
}

/// As onze secções de sempre, cada uma num cartão (o id de sempre: ordem e tema sobrevivem). Render e
/// Advanced já falam o vocabulário (doc 08 §1.2); os corpos antigos só valem em `PH2D_CARDS=0`.
fn legacy(st: &WidgetGalleryState) -> Vec<CardDef<'static>> {
    SHOWCASE_CARDS
        .iter()
        .map(|sc| {
            let sc = *sc;
            if sc.section == g::CARD_RENDER {
                return cards_render::render(st.choices, st.render);
            }
            if sc.section == g::CARD_ADVANCED {
                return cards_avancado::advanced(st.choices);
            }
            CardDef {
                section: sc.section,
                title: tool_title(sc.section),
                actions: Vec::new(),
                add: None,
                summary: None,
                body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
                    // O corpo de sempre pinta com o tema da TINTA do cartão (o menu de tema dele)
                    // e publica-a: os campos e os chips assentam na superfície do cartão.
                    let ink: CardInk = c.ink;
                    card_ink::com(ink, || {
                        (sc.body)(
                            c.scene,
                            c.text_system,
                            ink.theme,
                            c.hit_index,
                            c.store,
                            c.x,
                            c.w,
                            y,
                        )
                    })
                })),
            }
        })
        .collect()
}

/// Pinta a lista no `ctx.slot` do dock (o encaixe da direita, em aba com o Inspector).
pub(crate) fn paint(state: &mut WidgetGalleryState, ctx: &mut PaintCtx) -> Rect {
    let theme = ctx.host.theme();
    let mut cards = {
        let store = ctx.host.store_mut();
        cards_vocab::prepare(store);
        crate::valores::sync_picked(state, store, theme);
        crate::valores::sync_link(state, store);
        crate::valores::sync_health(state, store);
        crate::cards_rive::sync_pegas(state, store);
        crate::estilos_ficticios::sync(state, store, theme);
        cards_render::avanca(state);
        cards_vocab::vocabulary(state, store)
    };
    cards.extend(legacy(state));
    let rect = paint_card_list_with(
        ctx,
        ids::GAL_PANEL,
        tr("panel.widget_gallery.title"),
        &cards,
        None,
    );
    crate::lado::paint(state, ctx, rect, theme);
    crate::lado::head_menus(ctx, rect, theme);
    let (_, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_showcase_popovers(ctx.scene, ctx.text_system, theme, hit_index);
    rect
}
