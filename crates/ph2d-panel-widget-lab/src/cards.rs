//! ⭐ **O laboratório em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`): converte-se só o
//! cromo. O corpo é o estudo dos sliders, um canvas de teste que desenha de propósito as VARIANTES do widget
//! antigo e a régua de larguras: pintá-lo na tinta do cartão apagaria o que ele mede. Chips e caixa viva
//! continuam os mesmos ids, no mesmo `Bench`.

use crate::WidgetLabPanel;
use crate::state::WidgetLabState;
use crate::study::{Bench, paint_study};
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::cards::CardDef;
use ph2d_editor_core::panel::cards::linhas_extra::{FloatHandles, paint_floating_list};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;

thread_local! {
    static FORCADO: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// O painel pinta-se em cartões? `PH2D_CARDS=0` = o corpo de antes.
#[must_use]
pub fn cards_on() -> bool {
    FORCADO
        .with(std::cell::Cell::get)
        .unwrap_or_else(|| std::env::var("PH2D_CARDS").as_deref() != Ok("0"))
}

/// Força o modo NESTA thread (os testes de um processo correm em paralelo).
#[doc(hidden)]
pub fn force_cards_on_this_thread(on: Option<bool>) {
    FORCADO.with(|f| f.set(on));
}

pub(crate) fn paint(state: &WidgetLabState, ctx: &mut PaintCtx, rect: Rect, live: (f32, bool)) {
    ctx.host
        .store_mut()
        .mark_card_list_close(ids::LAB_PANEL, crate::ids::LAB_CLOSE);
    let copia = state.clone();
    let cartoes = [CardDef {
        section: crate::ids::LAB_SEC_BENCH,
        title: tr("card.widget_lab.bench"),
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(move |c, top| {
            let mut b = Bench {
                scene: &mut *c.scene,
                text: &mut *c.text_system,
                hit: &mut *c.hit_index,
                theme: c.ink.theme,
                x: c.x,
                w: c.w,
                y: top,
            };
            top + paint_study(&mut b, &copia, live)
        })),
    }];
    paint_floating_list(
        ctx,
        ids::LAB_PANEL,
        WidgetLabPanel::TITLE.tr(),
        &cartoes,
        rect,
        FloatHandles {
            drag: crate::ids::LAB_DRAG_HANDLE,
            resize_br: Some(crate::ids::LAB_RESIZE_HANDLE),
            resize_bl: crate::ids::LAB_RESIZE_HANDLE_BL,
        },
    );
}
