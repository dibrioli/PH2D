//! ⭐ **O painel Equalize Sizes em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`): os
//! mesmos ids; `populate` e `event` intocados. Os três modos são uma escolha, e os liga/desliga caixas.

use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::ferramenta::{
    Numero, Verbos, caixa_em_celula, caixas_aos_pares, campo, cartao, opcoes, prepara_numeros,
};
use ph2d_editor_core::panel::TextKey;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_i18n::{tr, tr_with};
use ph2d_tool_equalize_sizes::ids as t;
use ph2d_tool_equalize_sizes::params::{EqualizeSizesUiSnapshot, TargetMode, UpscaleAlgorithm};

const PASSO_PX: f64 = 1.0;

pub(crate) fn prepare(store: &mut WidgetStore, snap: &EqualizeSizesUiSnapshot) {
    let fabrica = EqualizeSizesUiSnapshot::default();
    let numero = |chip, fab: u32, valor: u32| Numero {
        chip,
        step: PASSO_PX,
        scrub: PASSO_PX,
        fabrica: f64::from(fab),
        valor: f64::from(valor),
    };
    prepara_numeros(
        store,
        &[
            numero(t::EQS_FIXED_W, fabrica.fixed_w, snap.fixed_w),
            numero(t::EQS_FIXED_H, fabrica.fixed_h, snap.fixed_h),
            numero(
                t::EQS_GRID_OFFSET_NUM,
                fabrica.grid_offset,
                snap.grid_offset,
            ),
        ],
    );
}

fn tamanho_fixo(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let w = Cell::new(Widget::Field {
        id: t::EQS_FIXED_W,
        glyph: Glyph::Text(tr("panel.equalize_sizes.fixed.w")),
    })
    .label(tr("card.equalize.fixed_size"));
    let h = Cell::new(Widget::Field {
        id: t::EQS_FIXED_H,
        glyph: Glyph::Text(tr("panel.equalize_sizes.fixed.h")),
    });
    c.row(&[w, h], y)
}

fn da_grelha(c: &mut CardCtx<'_>, snap: &EqualizeSizesUiSnapshot, y: f32) -> f32 {
    let celula = tr_with(
        "panel.equalize_sizes.size.cell_from_grid_snap",
        &[("cell", &snap.grid_unit)],
    );
    let y = c.hint(&celula, y);
    let y = c.pair(
        campo(t::EQS_GRID_OFFSET_NUM, tr("card.equalize.offset")),
        caixa_em_celula(
            t::EQS_ARRANGE_ON_GRID,
            Rotulo::vista(TextKey::new("panel.equalize_sizes.size.arrange_on_grid")),
            snap.arrange_on_grid,
        ),
        y,
    );
    let max_off = (snap.grid_unit / 2).max(1);
    let final_dim = snap
        .grid_unit
        .saturating_sub(snap.grid_offset.min(max_off))
        .max(1);
    c.hint(
        &tr_with(
            "panel.equalize_sizes.size.final_size",
            &[("size", &final_dim)],
        ),
        y,
    )
}

fn cartao_do_alvo(snap: EqualizeSizesUiSnapshot) -> CardDef<'static> {
    let modos = [
        (t::EQS_MODE_MAX, tr("panel.equalize_sizes.size.max")),
        (t::EQS_MODE_FIXED, tr("panel.equalize_sizes.size.fixed")),
        (t::EQS_MODE_GRID, tr("panel.equalize_sizes.size.grid")),
    ];
    let aceso = match snap.target_mode {
        TargetMode::MaxOfSelection => 0,
        TargetMode::Fixed => 1,
        TargetMode::GridUnit => 2,
    };
    cartao(
        t::EQS_SEC_TARGET,
        tr("card.equalize.target"),
        Some(modos[aceso].1.to_string()),
        move |c, y| {
            let segs = opcoes(&modos, aceso);
            let y = c.full(
                Some(tr("card.equalize.size_from")),
                Widget::Choice(&segs),
                y,
            );
            match snap.target_mode {
                TargetMode::MaxOfSelection => y,
                TargetMode::Fixed => tamanho_fixo(c, y),
                TargetMode::GridUnit => da_grelha(c, &snap, y),
            }
        },
    )
}

fn cartao_do_resultado(snap: EqualizeSizesUiSnapshot) -> CardDef<'static> {
    cartao(
        t::EQS_SEC_RESULT,
        tr("card.equalize.result"),
        None,
        move |c, y| {
            let mut y = caixas_aos_pares(
                c,
                &[
                    (
                        t::EQS_UPSCALE_IF_SMALLER,
                        "panel.equalize_sizes.size.upscale_if_smaller",
                        snap.upscale_if_smaller,
                    ),
                    (
                        t::EQS_RASTERIZE_AFTER,
                        "panel.equalize_sizes.size.rasterize_after",
                        snap.rasterize_after,
                    ),
                ],
                y,
            );
            if snap.upscale_if_smaller {
                let aceso = match snap.upscale_algorithm {
                    UpscaleAlgorithm::Lanczos3 => 0,
                    UpscaleAlgorithm::Nearest => 1,
                    UpscaleAlgorithm::Epx => 2,
                };
                let segs = opcoes(
                    &[
                        (t::EQS_ALG_LANCZOS, tr("panel.equalize_sizes.size.lanczos")),
                        (t::EQS_ALG_NEAREST, tr("panel.equalize_sizes.size.nearest")),
                        (t::EQS_ALG_EPX, tr("panel.equalize_sizes.size.epx")),
                    ],
                    aceso,
                );
                y = c.full(
                    Some(tr("card.equalize.algorithm")),
                    Widget::Choice(&segs),
                    y,
                );
            }
            y
        },
    )
}

pub(crate) fn cards(snap: &EqualizeSizesUiSnapshot) -> Vec<CardDef<'static>> {
    vec![cartao_do_alvo(*snap), cartao_do_resultado(*snap)]
}

pub(crate) fn verbos() -> Verbos<'static> {
    Verbos {
        reset: (
            t::EQS_RESET,
            tr("panel.equalize_sizes.actions.reset_to_defaults"),
        ),
        cancel: (t::EQS_CANCEL, tr("panel.equalize_sizes.actions.cancel")),
        apply: (t::EQS_APPLY, tr("panel.equalize_sizes.actions.apply")),
    }
}
