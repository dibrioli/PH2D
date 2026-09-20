//! **A fileira `Pigment` — uma lei, uma porta, três meios.**
//!
//! Até 2026-09-20 a mistura subtractiva era um controlo da aguada: a fileira vivia dentro do cartão
//! **Water** da secção Watercolor e o motor a gateava com `watercolor && pigment`. A ordem do dono
//! desse dia (*«ligue o digital»*) tirou a cerca do meio — a lei passou a valer para todo meio que
//! componha um dab pela porta `blend_over_pigment` —, e com ela a fileira deixou de pertencer a uma
//! secção só.
//!
//! ⛔⛔ **Por isso ela mora AQUI e não em dois sítios.** O valor que o slider mostra não é um campo:
//! ele é DERIVADO (`pigment ? pigment_mix : 0`, o par toggle+amount que a redesign de 2026-07-07
//! fundiu num controlo só), e *duas derivações do mesmo número divergem no dia em que alguém afina
//! uma delas*. Os dois hospedeiros — o cartão Water da aguada e o cartão **Mixing** que os outros
//! meios ganham — chamam [`paint_pigment_row`]; nenhum sabe a lei.
//!
//! ⚠️ **Quem responde «este meio tem a fileira?» é `PaintMedia::offers_pigment_mixing`**, e a lista
//! dela é MEDIDA (`diag_pigmento_por_meio`), não raciocinada: Digital · Watercolor · Impasto leem a
//! lei; o **Wet Paint não** (o depósito é do solver de fluido, que tem o Kubelka–Munk próprio e o
//! slider de pigmento dele). Um meio que lê a lei e não vê a fileira é um knob INALCANÇÁVEL; um que
//! a vê e não a lê é um knob MORTO — as duas leem-se igual numa tabela, e só a medição as separa.

use crate::card::{card_frame, card_row};
use crate::number_field;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_tool_painter::BrushSettings;

/// O número que o slider **mostra**: a quantidade quando a mistura está ligada, senão `0`.
///
/// ⚠️ Ele não é o `pigment_mix` cru. O controlo é o par *toggle + amount* fundido (redesign
/// 2026-07-07): `0` desliga a mistura **guardando** a última quantidade, para que subir o slider a
/// devolva sem perda. Quem escreve é o `set_brush_pigment_mixing` do lado da ferramenta; esta função
/// é a metade de LEITURA da mesma lei, e existe para as duas não divergirem.
pub(crate) fn pigment_amount(brush: &BrushSettings) -> f32 {
    if brush.pigment {
        brush.pigment_mix
    } else {
        0.0
    }
}

/// Pinta a fileira `Pigment` numa linha de cartão já enquadrada e devolve o `y` da linha seguinte.
pub(crate) fn paint_pigment_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    ix: f32,
    iw: f32,
    ry: f32,
    brush: &BrushSettings,
) -> f32 {
    card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        tr("panel.painter_layers.watercolor.pigment"),
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_MIX,
        pigment_amount(brush),
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    )
}

/// O cartão **Mixing** — o hospedeiro da fileira nos meios que não têm um cartão onde ela caiba.
///
/// ⚠️ **A aguada NÃO o usa:** lá a fileira vive no cartão *Water*, ao lado do Rewet e do Smudge, que
/// é onde o dono a aprendeu e onde ela pertence (as três são *«o que o traço faz com a tinta que já
/// está na tela»*). Pintar os dois seria pintar o mesmo id duas vezes no mesmo quadro.
pub(crate) fn paint_mixing_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let (ix, iw, ry, next_y) = card_frame(
        ctx,
        theme,
        x,
        content_w,
        y,
        tr("panel.painter_layers.brush.mixing"),
        1,
    );
    let _ = paint_pigment_row(ctx, theme, ix, iw, ry, brush);
    next_y
}
