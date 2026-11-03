//! ⭐⭐ **A galeria como LISTA DE CARTÕES no painel lateral** (doc `CartoesUI/08` §1): o
//! vocabulário do cartão primeiro ([`crate::cards_vocab`]), depois as onze secções de sempre, cada
//! uma o corpo de um cartão pintado com a tinta dele.

use crate::cards_vocab;
use crate::state::WidgetGalleryState;
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, paint_card_list_with};
use ph2d_editor_core::widget::card_ink::CardInk;
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

/// As onze secções de sempre, cada uma num cartão (o id de sempre: ordem e tema sobrevivem).
fn legacy() -> Vec<CardDef<'static>> {
    SHOWCASE_CARDS
        .iter()
        .map(|sc| {
            let sc = *sc;
            CardDef {
                section: sc.section,
                title: sc.title,
                actions: Vec::new(),
                add: None,
                summary: None,
                body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
                    // O corpo de sempre pinta com o tema da TINTA do cartão (o menu de tema dele).
                    let ink: CardInk = c.ink;
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
                })),
            }
        })
        .collect()
}

/// Pinta a lista no `ctx.slot` do dock (o encaixe da direita, em aba com o Inspector).
pub(crate) fn paint(state: &WidgetGalleryState, ctx: &mut PaintCtx) -> Rect {
    let mut cards = {
        let store = ctx.host.store_mut();
        cards_vocab::prepare(store);
        cards_vocab::vocabulary(state.choices, store)
    };
    cards.extend(legacy());
    let rect = paint_card_list_with(
        ctx,
        ids::GAL_PANEL,
        tr("panel.widget_gallery.title"),
        &cards,
        None,
    );
    let theme = ctx.host.theme();
    let (_, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_showcase_popovers(ctx.scene, ctx.text_system, theme, hit_index);
    rect
}
