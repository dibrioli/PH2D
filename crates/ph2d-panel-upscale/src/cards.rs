//! ⭐ **O painel Upscale em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`): os mesmos ids;
//! `populate` e `event` intocados. O campo é o chip ligado à barra, e é a barra que avisa a ferramenta.
//!
//! ⚠️ O campo mostra a ENTREGA (`effective_factor`), a mesma porta que o Apply usa: o EPX arredonda.

use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::ferramenta::{
    Numero, Verbos, campo, cartao, opcoes, prepara_numeros,
};
use ph2d_editor_core::panel::cards::{CardDef, Widget};
use ph2d_editor_core::widget::card_field;
use ph2d_i18n::{tr, tr_with};
use ph2d_tool_upscale::params::{
    DEFAULT_SCALE_FACTOR, UpscaleAlgorithm, UpscaleUiSnapshot, effective_factor,
    effective_output_size, scale_to_slider,
};
use ph2d_tool_upscale::tool::ids as t;

/// Um décimo por seta; o arrasto é mais fino, porque a faixa inteira são 15 unidades.
const PASSO: f64 = 0.1;
const ARRASTO: f64 = 0.02;

/// O trilho vivo: o do store enquanto a mão arrasta (a ferramenta chega um quadro depois), senão o
/// da ferramenta.
fn trilho(store: &WidgetStore, snap: &UpscaleUiSnapshot) -> f32 {
    if store
        .number_input_drag()
        .is_some_and(|d| d.id == t::UPS_SCALE_NUM)
    {
        store
            .slider(t::UPS_SCALE)
            .map_or_else(|| scale_to_slider(snap.scale_factor), |(_, v)| v)
    } else {
        scale_to_slider(snap.scale_factor)
    }
}

pub(crate) fn prepare(store: &mut WidgetStore, snap: &UpscaleUiSnapshot) {
    let valor = effective_factor(snap.algorithm, trilho(store, snap));
    prepara_numeros(
        store,
        &[Numero {
            chip: t::UPS_SCALE_NUM,
            step: PASSO,
            scrub: ARRASTO,
            fabrica: f64::from(DEFAULT_SCALE_FACTOR),
            valor: f64::from(valor),
        }],
    );
}

fn saida(snap: &UpscaleUiSnapshot, track: f32) -> String {
    if snap.source_w == 0 || snap.source_h == 0 {
        return tr("panel.upscale.scale.output_select_a_sprite").to_string();
    }
    let (ow, oh) = effective_output_size(snap.algorithm, track, snap.source_w, snap.source_h);
    tr_with(
        "panel.upscale.scale.output_size",
        &[
            ("ow", &ow),
            ("oh", &oh),
            ("iw", &snap.source_w),
            ("ih", &snap.source_h),
        ],
    )
}

pub(crate) fn cards(snap: &UpscaleUiSnapshot) -> Vec<CardDef<'static>> {
    let snap = *snap;
    let aceso = match snap.algorithm {
        UpscaleAlgorithm::Lanczos3 => 0,
        UpscaleAlgorithm::Nearest => 1,
        UpscaleAlgorithm::Epx => 2,
    };
    let resumo = format!("{:.2}", snap.effective_factor);
    vec![cartao(
        t::UPS_SEC_SCALE,
        tr("panel.upscale.scale.scale"),
        Some(resumo),
        move |c, y| {
            let segs = opcoes(
                &[
                    (t::UPS_ALGO_LANCZOS3, tr("panel.upscale.scale.lanczos3")),
                    (t::UPS_ALGO_NEAREST, tr("panel.upscale.scale.nearest")),
                    (t::UPS_ALGO_EPX, tr("panel.upscale.scale.epx")),
                ],
                aceso,
            );
            let y = c.full(
                Some(tr("panel.upscale.scale.algorithm")),
                Widget::Choice(&segs),
                y,
            );
            let track = trilho(c.store, &snap);
            let entrega = f64::from(effective_factor(snap.algorithm, track));
            let y = card_field::com_valores(&[(t::UPS_SCALE_NUM, entrega)], || {
                c.row(&[campo(t::UPS_SCALE_NUM, tr("card.upscale.scale"))], y)
            });
            c.hint(&saida(&snap, track), y)
        },
    )]
}

pub(crate) fn verbos() -> Verbos<'static> {
    Verbos {
        reset: (t::UPS_RESET, tr("panel.upscale.scale.reset_to_defaults")),
        cancel: (t::UPS_CANCEL, tr("panel.upscale.scale.cancel")),
        apply: (t::UPS_APPLY, tr("panel.upscale.scale.apply")),
    }
}
