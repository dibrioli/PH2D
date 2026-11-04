//! ⭐ **O painel Padding em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`): os mesmos
//! ids e o `panel_seam!` intocado. O campo é o chip ligado à barra, e é a barra que avisa a ferramenta.

use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::CardDef;
use ph2d_editor_core::panel::cards::ferramenta::{
    Numero, Verbos, aos_pares, caixa, campo, cartao, prepara_numeros,
};
use ph2d_i18n::tr;
use ph2d_tool_padding::ids as t;
use ph2d_tool_padding::params::PaddingUiSnapshot;

const PASSO_PX: f64 = 1.0;

pub(crate) fn prepare(store: &mut WidgetStore, snap: &PaddingUiSnapshot) {
    let fabrica = PaddingUiSnapshot::default();
    let numero = |chip, fab: i32, valor: i32| Numero {
        chip,
        step: PASSO_PX,
        scrub: PASSO_PX,
        fabrica: f64::from(fab),
        valor: f64::from(valor),
    };
    prepara_numeros(
        store,
        &[
            numero(t::PAD_TOP_NUM, fabrica.top, snap.top),
            numero(t::PAD_RIGHT_NUM, fabrica.right, snap.right),
            numero(t::PAD_BOTTOM_NUM, fabrica.bottom, snap.bottom),
            numero(t::PAD_LEFT_NUM, fabrica.left, snap.left),
        ],
    );
}

pub(crate) fn cards(snap: &PaddingUiSnapshot) -> Vec<CardDef<'static>> {
    let recentra = snap.recenter_pivot;
    let resumo = format!(
        "{} · {} · {} · {}",
        snap.top, snap.right, snap.bottom, snap.left
    );
    vec![cartao(
        crate::ids::PAD_SEC_EDGES,
        tr("card.padding.edges"),
        Some(resumo),
        move |c, y| {
            let cells = [
                campo(t::PAD_TOP_NUM, tr("card.padding.top")),
                campo(t::PAD_RIGHT_NUM, tr("card.padding.right")),
                campo(t::PAD_BOTTOM_NUM, tr("card.padding.bottom")),
                campo(t::PAD_LEFT_NUM, tr("card.padding.left")),
            ];
            let y = aos_pares(c, &cells, y);
            caixa(
                c,
                t::PAD_PIVOT_RECENTER,
                "card.padding.recenter_pivot",
                recentra,
                y,
            )
        },
    )]
}

pub(crate) fn verbos() -> Verbos<'static> {
    Verbos {
        reset: (t::PAD_RESET, tr("panel.padding.padding.reset_to_defaults")),
        cancel: (crate::ids::PAD_CANCEL, tr("panel.padding.padding.cancel")),
        apply: (t::PAD_APPLY, tr("panel.padding.padding.apply")),
    }
}
